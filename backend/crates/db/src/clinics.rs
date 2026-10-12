use serde::{Deserialize, Serialize};

use crate::{DbResult, MySqlPool};

/// Clinic details printed on the prescription header (§12).
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Clinic {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
}

pub async fn get(pool: &MySqlPool, id: u64) -> DbResult<Clinic> {
    sqlx::query_as("SELECT name, address, phone FROM clinics WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn update(pool: &MySqlPool, id: u64, c: &Clinic) -> DbResult<()> {
    sqlx::query("UPDATE clinics SET name = ?, address = ?, phone = ? WHERE id = ?")
        .bind(&c.name)
        .bind(&c.address)
        .bind(&c.phone)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
