//! Prescription templates (roadmap 2.1): a saved set of sections and medicines that
//! fills a new prescription in one click.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::types::Json;
use vd_domain::prescription::PrescriptionBody;

use crate::{DbResult, MySqlPool};

#[derive(Debug, Serialize)]
pub struct Template {
    pub id: u64,
    pub name: String,
    pub body: PrescriptionBody,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

type Row = (u64, String, Json<PrescriptionBody>, NaiveDateTime);

pub async fn list(pool: &MySqlPool, clinic_id: u64) -> DbResult<Vec<Template>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, name, body, updated_at FROM rx_templates WHERE clinic_id = ? ORDER BY name",
    )
    .bind(clinic_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, body, at)| Template {
            id,
            name,
            body: body.0,
            updated_at: at.and_utc(),
        })
        .collect())
}

/// Saves a template by name; a template with the same name is replaced.
pub async fn upsert(
    pool: &MySqlPool,
    clinic_id: u64,
    name: &str,
    body: &PrescriptionBody,
    user_id: u64,
) -> DbResult<u64> {
    sqlx::query(
        "INSERT INTO rx_templates (clinic_id, name, body, created_by) VALUES (?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE body = VALUES(body), id = LAST_INSERT_ID(id)",
    )
    .bind(clinic_id)
    .bind(name)
    .bind(Json(body))
    .bind(user_id)
    .execute(pool)
    .await
    .map(|r| r.last_insert_id())
}

pub async fn delete(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<bool> {
    let res = sqlx::query("DELETE FROM rx_templates WHERE clinic_id = ? AND id = ?")
        .bind(clinic_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() == 1)
}
