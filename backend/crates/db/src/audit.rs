use crate::{DbResult, MySqlPool};

/// Appends an audit trail entry (PM-21).
pub async fn log(
    pool: &MySqlPool,
    clinic_id: u64,
    user_id: Option<u64>,
    entity: &str,
    entity_id: Option<u64>,
    action: &str,
) -> DbResult<()> {
    sqlx::query(
        "INSERT INTO audit_logs (clinic_id, user_id, entity, entity_id, action) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(user_id)
    .bind(entity)
    .bind(entity_id)
    .bind(action)
    .execute(pool)
    .await?;
    Ok(())
}
