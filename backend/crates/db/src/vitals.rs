//! Vitals per visit (PM-13).

use chrono::{NaiveDate, NaiveDateTime};
use vd_domain::vitals::{Vitals, VitalsRecord};

use crate::{DbResult, MySqlPool};

#[derive(sqlx::FromRow)]
struct Row {
    visit_id: u64,
    visit_date: NaiveDate,
    bp_sys: Option<u16>,
    bp_dia: Option<u16>,
    pulse: Option<u16>,
    temp: Option<f64>,
    weight: Option<f64>,
    height: Option<f64>,
    bmi: Option<f64>,
    spo2: Option<u8>,
    rbs: Option<f64>,
    created_at: NaiveDateTime,
}

impl From<Row> for VitalsRecord {
    fn from(r: Row) -> Self {
        Self {
            visit_id: r.visit_id,
            visit_date: r.visit_date,
            vitals: Vitals {
                bp_sys: r.bp_sys,
                bp_dia: r.bp_dia,
                pulse: r.pulse,
                temp: r.temp,
                weight: r.weight,
                height: r.height,
                bmi: r.bmi,
                spo2: r.spo2,
                rbs: r.rbs,
            },
            recorded_at: r.created_at.and_utc(),
        }
    }
}

// DECIMAL columns are read as DOUBLE.
const SELECT: &str = "SELECT vt.visit_id, v.visit_date, vt.bp_sys, vt.bp_dia, vt.pulse,
        CAST(vt.temp AS DOUBLE) AS temp, CAST(vt.weight AS DOUBLE) AS weight,
        CAST(vt.height AS DOUBLE) AS height, CAST(vt.bmi AS DOUBLE) AS bmi, vt.spo2,
        CAST(vt.rbs AS DOUBLE) AS rbs, vt.created_at
     FROM vitals vt JOIN visits v ON v.id = vt.visit_id";

/// The latest vitals recorded for a visit.
pub async fn for_visit(pool: &MySqlPool, visit_id: u64) -> DbResult<Option<VitalsRecord>> {
    let row: Option<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT} WHERE vt.visit_id = ? ORDER BY vt.id DESC LIMIT 1"
    )))
    .bind(visit_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

/// Records vitals for a visit. A visit keeps one set: re-entering corrects it.
pub async fn save(
    pool: &MySqlPool,
    visit_id: u64,
    v: &Vitals,
    user_id: u64,
) -> DbResult<Option<VitalsRecord>> {
    let mut tx = pool.begin().await?;
    let existing: Option<u64> = sqlx::query_scalar(
        "SELECT id FROM vitals WHERE visit_id = ? ORDER BY id DESC LIMIT 1 FOR UPDATE",
    )
    .bind(visit_id)
    .fetch_optional(&mut *tx)
    .await?;
    // Both statements end with the row to write: the existing id, or the visit for a new row.
    let sql = match existing {
        Some(_) => {
            "UPDATE vitals SET bp_sys = ?, bp_dia = ?, pulse = ?, temp = ?, weight = ?,
                height = ?, bmi = ?, spo2 = ?, rbs = ?, recorded_by = ?, created_at = NOW()
             WHERE id = ?"
        }
        None => {
            "INSERT INTO vitals (bp_sys, bp_dia, pulse, temp, weight, height, bmi, spo2, rbs,
                recorded_by, visit_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        }
    };
    sqlx::query(sql)
        .bind(v.bp_sys)
        .bind(v.bp_dia)
        .bind(v.pulse)
        .bind(v.temp)
        .bind(v.weight)
        .bind(v.height)
        .bind(v.bmi)
        .bind(v.spo2)
        .bind(v.rbs)
        .bind(user_id)
        .bind(existing.unwrap_or(visit_id))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    for_visit(pool, visit_id).await
}

/// All of a patient's vitals, oldest first, one record per visit (trend charts).
pub async fn trend(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
) -> DbResult<Vec<VitalsRecord>> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT}
         WHERE v.clinic_id = ? AND v.patient_id = ? AND v.deleted_at IS NULL
           AND vt.id = (SELECT MAX(id) FROM vitals WHERE visit_id = v.id)
         ORDER BY v.visit_date, v.id"
    )))
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}
