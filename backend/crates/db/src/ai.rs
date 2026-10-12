//! AI call log, suggestions, prompt versions and patient summaries (AI spec §10,
//! PM spec §7). Every Claude call is logged with its model, prompt version and tokens.

use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;
use serde_json::Value;
use sqlx::types::Json;
use vd_domain::report::{
    AnalysisView, NewSuggestion, PatientSummary, SavedSummary, Section, Suggestion,
};

use crate::{DbResult, MySqlPool};

pub struct NewCall<'a> {
    pub clinic_id: u64,
    pub visit_id: Option<u64>,
    pub patient_id: Option<u64>,
    pub report_id: Option<u64>,
    /// extraction, clinical, safety, summary, search or sms_draft.
    pub kind: &'a str,
    pub content: Option<&'a Value>,
    pub model: &'a str,
    pub prompt_version: &'a str,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub latency_ms: Option<u32>,
    pub error: Option<&'a str>,
    pub created_by: Option<u64>,
}

/// Logs one Claude call (done or failed). Returns its id.
pub async fn log_call(pool: &MySqlPool, c: &NewCall<'_>) -> DbResult<u64> {
    Ok(sqlx::query(
        "INSERT INTO ai_analyses (clinic_id, visit_id, patient_id, report_id, analysis_type,
             content, claude_model, prompt_version, input_tokens, output_tokens,
             cache_read_tokens, latency_ms, status, error, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(c.clinic_id)
    .bind(c.visit_id)
    .bind(c.patient_id)
    .bind(c.report_id)
    .bind(c.kind)
    .bind(c.content.map(Json))
    .bind(c.model.chars().take(50).collect::<String>())
    .bind(c.prompt_version)
    .bind(c.input_tokens)
    .bind(c.output_tokens)
    .bind(c.cache_read_tokens)
    .bind(c.latency_ms)
    .bind(if c.error.is_some() { "failed" } else { "done" })
    .bind(c.error)
    .bind(c.created_by)
    .execute(pool)
    .await?
    .last_insert_id())
}

pub async fn add_suggestions(
    pool: &MySqlPool,
    analysis_id: u64,
    items: &[NewSuggestion],
) -> DbResult<()> {
    let mut tx = pool.begin().await?;
    for s in items {
        sqlx::query(
            "INSERT INTO ai_suggestions (analysis_id, section, text_en, text_bn, likelihood, source_refs)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(analysis_id)
        .bind(s.section.as_str())
        .bind(&s.text_en)
        .bind(&s.text_bn)
        .bind(s.likelihood.as_deref().map(|l| l.chars().take(20).collect::<String>()))
        .bind(Json(&s.source_refs))
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

#[derive(sqlx::FromRow)]
struct SuggestionRow {
    id: u64,
    section: String,
    text_en: String,
    text_bn: Option<String>,
    likelihood: Option<String>,
    source_refs: Option<Json<Vec<String>>>,
    action: String,
    edited_text: Option<String>,
    reason: Option<String>,
}

pub async fn suggestions(pool: &MySqlPool, analysis_id: u64) -> DbResult<Vec<Suggestion>> {
    let rows: Vec<SuggestionRow> = sqlx::query_as(
        "SELECT id, section, text_en, text_bn, likelihood, source_refs, action, edited_text, reason
         FROM ai_suggestions WHERE analysis_id = ? ORDER BY id",
    )
    .bind(analysis_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(Suggestion {
                id: r.id,
                section: Section::parse(&r.section)?,
                text_en: r.text_en,
                text_bn: r.text_bn,
                likelihood: r.likelihood,
                source_refs: r.source_refs.map(|j| j.0).unwrap_or_default(),
                action: r.action,
                edited_text: r.edited_text,
                reason: r.reason,
            })
        })
        .collect())
}

/// The latest successful analysis of a kind for a visit, with its suggestions.
pub async fn latest_for_visit(
    pool: &MySqlPool,
    clinic_id: u64,
    visit_id: u64,
    kind: &str,
) -> DbResult<Option<AnalysisView>> {
    let row: Option<(u64, Json<Value>, String, String, NaiveDateTime)> = sqlx::query_as(
        "SELECT id, content, claude_model, prompt_version, created_at FROM ai_analyses
         WHERE clinic_id = ? AND visit_id = ? AND analysis_type = ? AND status = 'done'
           AND content IS NOT NULL
         ORDER BY id DESC LIMIT 1",
    )
    .bind(clinic_id)
    .bind(visit_id)
    .bind(kind)
    .fetch_optional(pool)
    .await?;
    let Some((id, content, claude_model, prompt_version, created_at)) = row else {
        return Ok(None);
    };
    Ok(Some(AnalysisView {
        id,
        content: content.0,
        claude_model,
        prompt_version,
        created_at: created_at.and_utc(),
        suggestions: suggestions(pool, id).await?,
    }))
}

/// Records the doctor's decision on a suggestion (FR-33, FR-37). False if the
/// suggestion is not in this clinic.
pub async fn act(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    action: &str,
    edited_text: Option<&str>,
    reason: Option<&str>,
    user_id: u64,
) -> DbResult<bool> {
    let res = sqlx::query(
        "UPDATE ai_suggestions s JOIN ai_analyses a ON a.id = s.analysis_id
         SET s.action = ?, s.edited_text = ?, s.reason = ?, s.acted_by = ?, s.acted_at = UTC_TIMESTAMP()
         WHERE s.id = ? AND a.clinic_id = ?",
    )
    .bind(action)
    .bind(edited_text)
    .bind(reason)
    .bind(user_id)
    .bind(id)
    .bind(clinic_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

// ---- Prompt versions ----

/// The active prompt text and version for `name`, if one was saved in the database.
pub async fn active_prompt(pool: &MySqlPool, name: &str) -> DbResult<Option<(String, String)>> {
    sqlx::query_as(
        "SELECT version, system_prompt FROM prompt_versions
         WHERE name = ? AND is_active = 1 ORDER BY id DESC LIMIT 1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

/// Records a built-in prompt so every stored result can be traced to its text. It
/// becomes active only if the name has no active prompt yet.
pub async fn register_prompt(
    pool: &MySqlPool,
    name: &str,
    version: &str,
    system: &str,
    schema: &Value,
) -> DbResult<()> {
    let has_active: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM prompt_versions WHERE name = ? AND is_active = 1)",
    )
    .bind(name)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        "INSERT IGNORE INTO prompt_versions (name, version, system_prompt, `schema`, is_active)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind(version)
    .bind(system)
    .bind(Json(schema))
    .bind(!has_active)
    .execute(pool)
    .await?;
    Ok(())
}

// ---- Patient summaries ----

#[allow(clippy::too_many_arguments)]
pub async fn save_summary(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    content: &PatientSummary,
    model: &str,
    prompt_version: &str,
    input_tokens: u32,
    output_tokens: u32,
    created_by: u64,
) -> DbResult<u64> {
    Ok(sqlx::query(
        "INSERT INTO ai_summaries (clinic_id, patient_id, content, claude_model, prompt_version,
             input_tokens, output_tokens, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .bind(Json(content))
    .bind(model.chars().take(50).collect::<String>())
    .bind(prompt_version)
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(created_by)
    .execute(pool)
    .await?
    .last_insert_id())
}

pub async fn latest_summary(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
) -> DbResult<Option<SavedSummary>> {
    let row: Option<(u64, Json<PatientSummary>, String, String, NaiveDateTime)> = sqlx::query_as(
        "SELECT s.id, s.content, s.claude_model, s.prompt_version, s.created_at
         FROM ai_summaries s JOIN patients p ON p.id = s.patient_id
         WHERE p.clinic_id = ? AND s.patient_id = ?
         ORDER BY s.id DESC LIMIT 1",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_optional(pool)
    .await?;
    Ok(
        row.map(|(id, c, claude_model, prompt_version, at)| SavedSummary {
            id,
            content: c.0,
            claude_model,
            prompt_version,
            created_at: at.and_utc(),
        }),
    )
}

// ---- Usage report (AI spec §8.9 "log tokens per call in your own DB") ----

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UsageRow {
    pub day: NaiveDate,
    pub analysis_type: String,
    pub claude_model: String,
    pub calls: i64,
    pub failed: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
}

pub async fn usage(
    pool: &MySqlPool,
    clinic_id: u64,
    from: NaiveDate,
    to: NaiveDate,
) -> DbResult<Vec<UsageRow>> {
    // Days in clinic time (UTC+6).
    sqlx::query_as(
        "SELECT DATE(created_at + INTERVAL 6 HOUR) AS day, CAST(analysis_type AS CHAR) AS analysis_type,
                claude_model,
                CAST(COUNT(*) AS SIGNED) AS calls,
                CAST(COALESCE(SUM(status = 'failed'), 0) AS SIGNED) AS failed,
                CAST(COALESCE(SUM(input_tokens), 0) AS SIGNED) AS input_tokens,
                CAST(COALESCE(SUM(output_tokens), 0) AS SIGNED) AS output_tokens,
                CAST(COALESCE(SUM(cache_read_tokens), 0) AS SIGNED) AS cache_read_tokens
         FROM ai_analyses
         WHERE clinic_id = ? AND created_at >= ? - INTERVAL 6 HOUR
           AND created_at < ? + INTERVAL 1 DAY - INTERVAL 6 HOUR
         GROUP BY day, analysis_type, claude_model
         ORDER BY day DESC, analysis_type",
    )
    .bind(clinic_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}
