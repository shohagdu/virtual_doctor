//! Outgoing SMS (PM-17). Messages are stored here and sent by the jobs worker
//! (`sms_send` jobs), so a slow or failing gateway never blocks the screen.

use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::types::Json;
use vd_domain::sms::SmsKind;

use crate::{DbResult, MySqlPool};

pub struct NewSms<'a> {
    pub clinic_id: u64,
    pub patient_id: Option<u64>,
    /// Already normalised (8801XXXXXXXXX).
    pub phone: &'a str,
    pub body: &'a str,
    pub kind: SmsKind,
    pub ref_id: Option<u64>,
    pub created_by: Option<u64>,
    pub send_after: DateTime<Utc>,
}

/// Stores a message and schedules its sending. Returns the message id.
pub async fn queue(pool: &MySqlPool, m: &NewSms<'_>) -> DbResult<u64> {
    let mut tx = pool.begin().await?;
    let id = sqlx::query(
        "INSERT INTO sms_messages (clinic_id, patient_id, phone, body, kind, ref_id, send_after, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(m.clinic_id)
    .bind(m.patient_id)
    .bind(m.phone)
    .bind(m.body)
    .bind(m.kind.as_str())
    .bind(m.ref_id)
    .bind(m.send_after.naive_utc())
    .bind(m.created_by)
    .execute(&mut *tx)
    .await?
    .last_insert_id();
    schedule(&mut tx, id, m.send_after.naive_utc()).await?;
    tx.commit().await?;
    Ok(id)
}

async fn schedule(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    sms_id: u64,
    run_at: NaiveDateTime,
) -> DbResult<()> {
    sqlx::query("INSERT INTO jobs (type, payload, run_at) VALUES ('sms_send', ?, ?)")
        .bind(Json(serde_json::json!({ "sms_id": sms_id })))
        .bind(run_at)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Withdraws messages not yet sent, e.g. the reminder for an appointment that was
/// cancelled or moved.
pub async fn cancel_queued(pool: &MySqlPool, kind: SmsKind, ref_id: u64) -> DbResult<()> {
    sqlx::query(
        "UPDATE sms_messages SET status = 'cancelled'
         WHERE kind = ? AND ref_id = ? AND status = 'queued'",
    )
    .bind(kind.as_str())
    .bind(ref_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Sends a failed message again. Returns false if it is not a failed message of
/// this clinic.
pub async fn retry(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<bool> {
    let mut tx = pool.begin().await?;
    let res = sqlx::query(
        "UPDATE sms_messages SET status = 'queued', attempts = 0, error = NULL
         WHERE clinic_id = ? AND id = ? AND status = 'failed'",
    )
    .bind(clinic_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Ok(false);
    }
    schedule(&mut tx, id, Utc::now().naive_utc()).await?;
    tx.commit().await?;
    Ok(true)
}

/// What the worker needs to send one message.
#[derive(sqlx::FromRow)]
pub struct Outgoing {
    pub phone: String,
    pub body: String,
    pub kind: String,
    pub ref_id: Option<u64>,
    pub status: String,
}

pub async fn outgoing(pool: &MySqlPool, id: u64) -> DbResult<Option<Outgoing>> {
    sqlx::query_as("SELECT phone, body, kind, ref_id, status FROM sms_messages WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Records a successful send, and stamps the appointment or follow-up it was for.
pub async fn mark_sent(pool: &MySqlPool, id: u64, provider_ref: &str) -> DbResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE sms_messages SET status = 'sent', sent_at = UTC_TIMESTAMP(), provider_ref = ?,
             attempts = attempts + 1, error = NULL
         WHERE id = ?",
    )
    .bind(provider_ref)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE appointments a JOIN sms_messages s ON s.ref_id = a.id
         SET a.sms_sent_at = s.sent_at
         WHERE s.id = ? AND s.kind IN ('appointment_confirm', 'appointment_reminder')",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE followups f JOIN sms_messages s ON s.ref_id = f.id
         SET f.reminder_sent_at = s.sent_at
         WHERE s.id = ? AND s.kind = 'followup_reminder'",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

/// Records a failed attempt. `give_up` marks the message failed (shown in the
/// follow-up list with a retry button); otherwise it stays queued for the next try.
pub async fn mark_error(pool: &MySqlPool, id: u64, error: &str, give_up: bool) -> DbResult<()> {
    let error: String = error.chars().take(255).collect();
    sqlx::query(
        "UPDATE sms_messages SET attempts = attempts + 1, error = ?,
             status = IF(?, 'failed', status)
         WHERE id = ? AND status = 'queued'",
    )
    .bind(error)
    .bind(give_up)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
