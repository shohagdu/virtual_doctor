use crate::{DbResult, MySqlPool};

#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: u64,
    pub clinic_id: u64,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
}

pub async fn find_by_email(pool: &MySqlPool, email: &str) -> DbResult<Option<UserRow>> {
    sqlx::query_as(
        "SELECT id, clinic_id, name, email, password_hash, role, is_active FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

/// Returns the first clinic's id, creating a default clinic on a fresh install.
pub async fn ensure_default_clinic(pool: &MySqlPool, name: &str) -> DbResult<u64> {
    if let Some(id) = sqlx::query_scalar::<_, u64>("SELECT id FROM clinics ORDER BY id LIMIT 1")
        .fetch_optional(pool)
        .await?
    {
        return Ok(id);
    }
    let res = sqlx::query("INSERT INTO clinics (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(res.last_insert_id())
}

pub async fn create(
    pool: &MySqlPool,
    clinic_id: u64,
    name: &str,
    email: &str,
    password_hash: &str,
    role: &str,
) -> DbResult<u64> {
    let res = sqlx::query(
        "INSERT INTO users (clinic_id, name, email, password_hash, role) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(name)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(res.last_insert_id())
}
