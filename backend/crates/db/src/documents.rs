//! Certificates and letters written for a visit (roadmap 2.9).

use chrono::NaiveDateTime;
use sqlx::types::Json;
use vd_domain::document::{Document, DocumentBody, DocumentKind};

use crate::{DbResult, MySqlPool};

type Row = (u64, u64, u64, String, Json<DocumentBody>, NaiveDateTime);

fn into(r: Row) -> Option<Document> {
    let (id, visit_id, patient_id, kind, body, at) = r;
    Some(Document {
        id,
        visit_id,
        patient_id,
        kind: DocumentKind::parse(&kind)?,
        body: body.0,
        created_at: at.and_utc(),
    })
}

pub async fn for_visit(pool: &MySqlPool, clinic_id: u64, visit_id: u64) -> DbResult<Vec<Document>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, visit_id, patient_id, kind, body, created_at FROM documents
         WHERE clinic_id = ? AND visit_id = ? ORDER BY id DESC",
    )
    .bind(clinic_id)
    .bind(visit_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(into).collect())
}

pub async fn get(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<Option<Document>> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, visit_id, patient_id, kind, body, created_at FROM documents WHERE clinic_id = ? AND id = ?",
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(into))
}

pub async fn create(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    visit_id: u64,
    kind: DocumentKind,
    body: &DocumentBody,
    user_id: u64,
) -> DbResult<u64> {
    Ok(sqlx::query(
        "INSERT INTO documents (clinic_id, patient_id, visit_id, kind, body, created_by) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .bind(visit_id)
    .bind(kind.as_str())
    .bind(Json(body))
    .bind(user_id)
    .execute(pool)
    .await?
    .last_insert_id())
}
