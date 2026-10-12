//! A doctor's favourite medicines with their usual dose (roadmap 2.3).

use serde::Serialize;
use sqlx::types::Json;
use vd_domain::prescription::MedicineLine;

use crate::{DbResult, MySqlPool};

#[derive(Debug, Serialize)]
pub struct Favorite {
    pub id: u64,
    pub line: MedicineLine,
}

pub async fn list(pool: &MySqlPool, user_id: u64) -> DbResult<Vec<Favorite>> {
    let rows: Vec<(u64, Json<MedicineLine>)> = sqlx::query_as(
        "SELECT id, line FROM medicine_favorites WHERE user_id = ? ORDER BY brand, id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, line)| Favorite { id, line: line.0 })
        .collect())
}

/// Adds a favourite, replacing one with the same brand and strength.
pub async fn add(pool: &MySqlPool, user_id: u64, line: &MedicineLine) -> DbResult<u64> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "DELETE FROM medicine_favorites WHERE user_id = ? AND brand = ?
           AND COALESCE(JSON_UNQUOTE(JSON_EXTRACT(line, '$.strength')), '') = ?",
    )
    .bind(user_id)
    .bind(&line.brand)
    .bind(line.strength.as_deref().unwrap_or_default())
    .execute(&mut *tx)
    .await?;
    let id = sqlx::query("INSERT INTO medicine_favorites (user_id, line, brand) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(Json(line))
        .bind(&line.brand)
        .execute(&mut *tx)
        .await?
        .last_insert_id();
    tx.commit().await?;
    Ok(id)
}

pub async fn delete(pool: &MySqlPool, user_id: u64, id: u64) -> DbResult<bool> {
    let res = sqlx::query("DELETE FROM medicine_favorites WHERE user_id = ? AND id = ?")
        .bind(user_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() == 1)
}
