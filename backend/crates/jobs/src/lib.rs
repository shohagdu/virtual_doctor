//! Background worker over the MySQL `jobs` table (SMS reminders, AI batch jobs, retries).

mod sms;

use std::sync::Arc;
use std::time::Duration;

use sqlx::types::Json;
use vd_common::config::SmsConfig;
use vd_db::MySqlPool;

use crate::sms::Gateway;

const POLL_INTERVAL: Duration = Duration::from_secs(15);
const MAX_ATTEMPTS: u32 = 5;

#[derive(sqlx::FromRow)]
struct Job {
    id: u64,
    #[sqlx(rename = "type")]
    kind: String,
    payload: Json<serde_json::Value>,
    attempts: u32,
}

/// What job handlers need besides the database.
struct Ctx {
    sms: Gateway,
}

pub fn spawn(pool: MySqlPool, sms: &SmsConfig) -> tokio::task::JoinHandle<()> {
    let ctx = Arc::new(Ctx {
        sms: Gateway::from_config(sms),
    });
    tokio::spawn(async move {
        loop {
            match tick(&pool, &ctx).await {
                Ok(true) => continue, // more work may be waiting
                Ok(false) => {}
                Err(e) => tracing::warn!(error = %e, "job worker tick failed"),
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
}

/// Claims and runs one due job. Returns whether a job was found.
async fn tick(pool: &MySqlPool, ctx: &Ctx) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let job: Option<Job> = sqlx::query_as(
        "SELECT id, type, payload, attempts FROM jobs
         WHERE status = 'pending' AND run_at <= UTC_TIMESTAMP()
         ORDER BY run_at LIMIT 1 FOR UPDATE SKIP LOCKED",
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(job) = job else { return Ok(false) };
    sqlx::query("UPDATE jobs SET status = 'running' WHERE id = ?")
        .bind(job.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let attempts = job.attempts + 1;
    let result = run(pool, ctx, &job, attempts >= MAX_ATTEMPTS).await;

    let (status, error) = match result {
        Ok(()) => ("done", None),
        Err(e) if attempts >= MAX_ATTEMPTS => ("failed", Some(e)),
        Err(e) => ("pending", Some(e)),
    };
    // Exponential backoff: 2, 4, 8, 16 minutes.
    sqlx::query(
        "UPDATE jobs SET status = ?, attempts = ?, last_error = ?,
             run_at = UTC_TIMESTAMP() + INTERVAL POW(2, ?) MINUTE
         WHERE id = ?",
    )
    .bind(status)
    .bind(attempts)
    .bind(error)
    .bind(attempts)
    .bind(job.id)
    .execute(pool)
    .await?;
    Ok(true)
}

/// Runs one job. `last_try` tells the handler this is the final attempt.
async fn run(pool: &MySqlPool, ctx: &Ctx, job: &Job, last_try: bool) -> Result<(), String> {
    match job.kind.as_str() {
        "sms_send" => {
            let id = job.payload.0["sms_id"]
                .as_u64()
                .ok_or("sms_send job without sms_id")?;
            send_sms(pool, ctx, id, last_try).await
        }
        other => Err(format!("no handler for job type '{other}'")),
    }
}

/// Sends a stored message unless it was cancelled or already sent.
async fn send_sms(pool: &MySqlPool, ctx: &Ctx, id: u64, last_try: bool) -> Result<(), String> {
    let db = |e: sqlx::Error| e.to_string();
    let Some(m) = vd_db::sms::outgoing(pool, id).await.map_err(db)? else {
        return Ok(());
    };
    if m.status != "queued" {
        return Ok(());
    }
    match ctx.sms.send(&m.phone, &m.body).await {
        Ok(provider_ref) => {
            vd_db::sms::mark_sent(pool, id, &provider_ref)
                .await
                .map_err(db)?;
            tracing::info!(sms_id = id, kind = %m.kind, ref_id = ?m.ref_id, "SMS sent");
            Ok(())
        }
        Err(e) => {
            vd_db::sms::mark_error(pool, id, &e, last_try)
                .await
                .map_err(db)?;
            Err(e)
        }
    }
}
