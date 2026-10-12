use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::clinic_today;
use vd_domain::followup::{Bucket, BucketCounts, Followup, FollowupStatus};
use vd_domain::settings::MAX_SMS_CHARS;
use vd_domain::sms::{SmsKind, bd_msisdn, segments};
use vd_domain::text::check_len;

use crate::auth::AuthUser;
use crate::routes::notify;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    bucket: Bucket,
}

#[derive(Serialize)]
pub struct FollowupList {
    counts: BucketCounts,
    items: Vec<Followup>,
}

/// "Follow-ups Due" (§5.7): Today / Overdue / This week, plus recently closed ones.
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<FollowupList>> {
    let today = clinic_today();
    let items = vd_db::followups::list(
        &state.pool,
        user.clinic_id,
        q.bucket,
        today,
        user.role == Role::Doctor,
    )
    .await
    .map_err(AppError::internal)?;
    let counts = vd_db::followups::counts(&state.pool, user.clinic_id, today)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(FollowupList { counts, items }))
}

#[derive(Deserialize)]
pub struct Update {
    status: FollowupStatus,
}

/// Mark as Came / Missed, or back to pending.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<Update>,
) -> AppResult<StatusCode> {
    if !vd_db::followups::set_status(&state.pool, user.clinic_id, id, req.status)
        .await
        .map_err(AppError::internal)?
    {
        return Err(AppError::NotFound);
    }
    user.audit(&state, "followup", id, req.status.as_str())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct SmsDraft {
    body: String,
    segments: usize,
    /// SMS is switched on in Settings.
    enabled: bool,
    /// The patient's phone is a mobile number that can receive SMS.
    phone_ok: bool,
}

/// The reminder text from the clinic's template, for review before sending (§7.3:
/// the assistant reviews before sending).
pub async fn sms_draft(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<SmsDraft>> {
    let (patient, due) = vd_db::followups::for_reminder(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let settings = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    let body = notify::compose(
        &state,
        user.clinic_id,
        &settings,
        &settings.sms.templates.followup_reminder,
        &patient,
        due,
        None,
    )
    .await?;
    Ok(Json(SmsDraft {
        segments: segments(&body),
        body,
        enabled: settings.sms.enabled,
        phone_ok: bd_msisdn(&patient.phone).is_some(),
    }))
}

#[derive(Deserialize)]
pub struct SendSms {
    body: String,
}

/// Sends the (possibly edited) reminder.
pub async fn send_sms(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<SendSms>,
) -> AppResult<StatusCode> {
    let settings = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    if !settings.sms.enabled {
        return Err(AppError::Conflict("SMS is turned off in Settings".into()));
    }
    let body = req.body.split_whitespace().collect::<Vec<_>>().join(" ");
    if body.is_empty() {
        return Err(AppError::Validation("the message is empty".into()));
    }
    check_len(&body, "message", MAX_SMS_CHARS)?;
    let (patient, _) = vd_db::followups::for_reminder(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let sms_id = notify::queue(
        &state,
        notify::Outgoing {
            clinic_id: user.clinic_id,
            patient: &patient,
            body: &body,
            kind: SmsKind::FollowupReminder,
            ref_id: id,
            created_by: Some(user.id),
            send_after: Utc::now(),
        },
    )
    .await?;
    user.audit(&state, "sms", sms_id, "queue").await?;
    Ok(StatusCode::ACCEPTED)
}

/// Sends a failed SMS again (§13 "SMS failed: retry").
pub async fn retry_sms(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<StatusCode> {
    if !vd_db::sms::retry(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
    {
        return Err(AppError::Conflict(
            "only a failed SMS can be retried".into(),
        ));
    }
    user.audit(&state, "sms", id, "retry").await?;
    Ok(StatusCode::ACCEPTED)
}
