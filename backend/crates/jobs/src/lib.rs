//! Background worker over the MySQL `jobs` table (SMS reminders, AI batch jobs, retries).

use std::time::Duration;
use vd_db::MySqlPool;

const POLL_INTERVAL: Duration = Duration::from_secs(15);
const MAX_ATTEMPTS: u32 = 5;

#[derive(sqlx::FromRow)]
struct Job {
    id: u64,
    #[sqlx(rename = "type")]
    kind: String,
    attempts: u32,
}

pub fn spawn(pool: MySqlPool) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match tick(&pool).await {
                Ok(true) => continue, // more work may be waiting
                Ok(false) => {}
                Err(e) => tracing::warn!(error = %e, "job worker tick failed"),
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
}

/// Claims and runs one due job. Returns whether a job was found.
async fn tick(pool: &MySqlPool) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let job: Option<Job> = sqlx::query_as(
        "SELECT id, type, attempts FROM jobs
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

    let result = run(&job.kind).await;

    let attempts = job.attempts + 1;
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

#[allow(clippy::match_single_binding)] // arms are added as job types are implemented
async fn run(kind: &str) -> Result<(), String> {
    match kind {
        // Handlers (sms_reminder, ai_batch, …) are added with their features.
        other => Err(format!("no handler for job type '{other}'")),
    }
}
