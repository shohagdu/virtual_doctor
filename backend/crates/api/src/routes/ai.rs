//! Shared plumbing for the AI features: availability checks, prompt lookup, call
//! logging and the server-sent-event progress stream (AI spec §8–9, PM spec §7).

use std::convert::Infallible;
use std::time::Duration;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use chrono::{Duration as Days, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use vd_ai::prompts::Prompt;
use vd_ai::{AiError, ToolAnswer, ToolCall};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::clinic_today;
use vd_domain::settings::ClinicSettings;

use crate::auth::AuthUser;
use crate::state::AppState;

/// The clinic's settings, if AI may be used: switched on for the clinic and an API
/// key configured on the server.
pub async fn ready(state: &AppState, clinic_id: u64) -> AppResult<ClinicSettings> {
    let settings = vd_db::settings::get(&state.pool, clinic_id)
        .await
        .map_err(AppError::internal)?;
    if !settings.ai.enabled {
        return Err(AppError::Conflict(
            "AI features are turned off in Settings".into(),
        ));
    }
    if !state.claude.is_configured() {
        return Err(AppError::Conflict(
            "AI is not set up on this server (no Claude API key) — enter values manually".into(),
        ));
    }
    Ok(settings)
}

/// The system prompt and version to use: an active override saved in the database,
/// else the built-in one.
pub async fn prompt(state: &AppState, p: &Prompt) -> AppResult<(String, String)> {
    let saved = vd_db::ai::active_prompt(&state.pool, p.name)
        .await
        .map_err(AppError::internal)?;
    Ok(saved.unwrap_or_else(|| (p.version.to_string(), p.system.to_string())))
}

/// Records the built-in prompts at startup so results can be traced to their text.
pub async fn register_prompts(pool: &vd_db::MySqlPool) {
    for p in vd_ai::prompts::ALL {
        if let Err(e) =
            vd_db::ai::register_prompt(pool, p.name, p.version, p.system, &(p.schema)()).await
        {
            tracing::warn!(prompt = p.name, error = %e, "could not record prompt version");
        }
    }
}

/// What a call was about, for the log.
#[derive(Clone, Copy, Default)]
pub struct Subject {
    pub visit_id: Option<u64>,
    pub patient_id: Option<u64>,
    pub report_id: Option<u64>,
}

/// Runs a forced tool call (optionally streamed with progress) and logs it, done or
/// failed. Returns the answer and the log id.
#[allow(clippy::too_many_arguments)]
pub async fn call(
    state: &AppState,
    user: &AuthUser,
    p: &Prompt,
    kind: &str,
    subject: Subject,
    tier: vd_ai::Tier,
    max_tokens: u32,
    content: Value,
    progress: Option<mpsc::Sender<usize>>,
) -> AppResult<(ToolAnswer, u64)> {
    let (version, system) = prompt(state, p).await?;
    let schema = (p.schema)();
    let call = ToolCall {
        tier,
        max_tokens,
        system: &system,
        tool_name: p.tool_name,
        tool_description: p.tool_description,
        schema: &schema,
        content,
    };
    let result = match progress {
        Some(tx) => state.claude.tool_streamed(&call, tx).await,
        None => state.claude.tool(&call).await,
    };
    let model = state.claude.model(tier).to_string();
    let log = |answer: Option<&ToolAnswer>, error: Option<String>| {
        let model = answer.map(|a| a.model.clone()).unwrap_or(model.clone());
        let usage = answer.map(|a| a.usage).unwrap_or_default();
        let latency = answer.map(|a| a.latency_ms);
        let content = answer.map(|a| a.input.clone());
        let version = version.clone();
        async move {
            vd_db::ai::log_call(
                &state.pool,
                &vd_db::ai::NewCall {
                    clinic_id: user.clinic_id,
                    visit_id: subject.visit_id,
                    patient_id: subject.patient_id,
                    report_id: subject.report_id,
                    kind,
                    content: content.as_ref(),
                    model: &model,
                    prompt_version: &version,
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    cache_read_tokens: usage.cache_read_input_tokens,
                    latency_ms: latency,
                    error: error.as_deref(),
                    created_by: Some(user.id),
                },
            )
            .await
            .map_err(AppError::internal)
        }
    };
    match result {
        Ok(answer) => {
            let id = log(Some(&answer), None).await?;
            Ok((answer, id))
        }
        Err(e) => {
            log(None, Some(e.to_string())).await?;
            Err(ai_error(e))
        }
    }
}

/// Claude failures as client errors the screen can explain (AI spec §8.11).
pub fn ai_error(e: AiError) -> AppError {
    tracing::warn!(error = %e, "Claude call failed");
    match e {
        AiError::NotConfigured => AppError::Conflict("AI is not set up on this server".into()),
        AiError::BadOutput => {
            AppError::Conflict("the AI answer could not be read — please try again".into())
        }
        AiError::Api { status: 429, .. } | AiError::Api { status: 529, .. } => {
            AppError::Conflict("the AI service is busy — please try again in a minute".into())
        }
        AiError::Api { status, .. } if status >= 500 => {
            AppError::Conflict("the AI service is unavailable — please try again".into())
        }
        AiError::Api { .. } => AppError::Conflict("the AI request was refused".into()),
        AiError::Http(_) => {
            AppError::Conflict("could not reach the AI service — check the connection".into())
        }
    }
}

/// Runs `work` and reports it to the browser as server-sent events: `progress`
/// (characters received so far) while Claude writes, then `done` with the result
/// or `error` with a message (AI spec §9.1).
pub fn sse<F, Fut, T>(work: F) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
where
    F: FnOnce(mpsc::Sender<usize>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = AppResult<T>> + Send + 'static,
    T: Serialize + Send + 'static,
{
    let (events, rx) = mpsc::channel::<Result<Event, Infallible>>(32);
    let (progress, mut chars) = mpsc::channel::<usize>(64);
    tokio::spawn(async move {
        let forward = {
            let events = events.clone();
            tokio::spawn(async move {
                while let Some(n) = chars.recv().await {
                    let _ = events
                        .send(Ok(Event::default().event("progress").data(n.to_string())))
                        .await;
                }
            })
        };
        let result = work(progress).await;
        let _ = forward.await;
        let event = match result {
            Ok(v) => Event::default()
                .event("done")
                .data(serde_json::to_string(&v).unwrap_or_else(|_| "null".into())),
            Err(e) => {
                let message = match &e {
                    AppError::Internal(_) => {
                        tracing::error!(error = %e, "AI request failed");
                        "something went wrong".to_string()
                    }
                    other => other.to_string(),
                };
                Event::default()
                    .event("error")
                    .data(serde_json::json!({ "error": message }).to_string())
            }
        };
        let _ = events.send(Ok(event)).await;
    });
    Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::new().interval(Duration::from_secs(10)))
}

/// Age in whole years and days (rule engine ranges use days).
pub fn age(dob: NaiveDate) -> (i64, i64) {
    let days = (clinic_today() - dob).num_days().max(0);
    (days / 365, days)
}

#[derive(Serialize)]
pub struct Status {
    configured: bool,
    enabled: bool,
}

/// Whether AI features can be offered (the screens hide them otherwise).
pub async fn status(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Status>> {
    let settings = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(Status {
        configured: state.claude.is_configured(),
        enabled: settings.ai.enabled,
    }))
}

#[derive(Deserialize)]
pub struct Range {
    from: NaiveDate,
    to: NaiveDate,
}

#[derive(Serialize)]
pub struct Usage {
    from: NaiveDate,
    to: NaiveDate,
    rows: Vec<vd_db::ai::UsageRow>,
}

/// Token usage per day, task and model (AI spec §8.9; roadmap 3.9). Prices change,
/// so cost is worked out from these counts against the current price list.
pub async fn usage(
    State(state): State<AppState>,
    user: AuthUser,
    Query(r): Query<Range>,
) -> AppResult<Json<Usage>> {
    user.require(&[Role::Doctor, Role::Admin])?;
    if r.to < r.from || r.to - r.from > Days::days(366) {
        return Err(AppError::Validation("date range: up to one year".into()));
    }
    let rows = vd_db::ai::usage(&state.pool, user.clinic_id, r.from, r.to)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(Usage {
        from: r.from,
        to: r.to,
        rows,
    }))
}
