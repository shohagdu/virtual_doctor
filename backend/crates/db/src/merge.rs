//! Merging duplicate patient records (PM-19, §13 "Merge patients"). Everything
//! belonging to the merged-away record moves to the kept one; nothing is deleted.

use vd_common::AppError;
use vd_domain::medical_profile::merge as merge_profiles;

use crate::{MySqlPool, medical_profile};

fn db(e: sqlx::Error) -> AppError {
    AppError::internal(e)
}

/// Tables whose rows point at a patient directly.
const PATIENT_TABLES: [&str; 7] = [
    "visits",
    "followups",
    "appointments",
    "reports",
    "ai_summaries",
    "sms_messages",
    "documents",
];

/// Moves all of `merge_id`'s history to `keep_id`, combines the medical profiles,
/// renumbers the kept patient's visits by date, and soft-deletes `merge_id` with a
/// pointer to `keep_id`.
pub async fn merge(
    pool: &MySqlPool,
    clinic_id: u64,
    keep_id: u64,
    merge_id: u64,
) -> Result<(), AppError> {
    if keep_id == merge_id {
        return Err(AppError::Validation("choose two different patients".into()));
    }
    // Read profiles first (outside the transaction; they are re-written inside it).
    let keep_profile = medical_profile::get(pool, clinic_id, keep_id)
        .await
        .map_err(db)?
        .ok_or(AppError::NotFound)?;
    let other_profile = medical_profile::get(pool, clinic_id, merge_id)
        .await
        .map_err(db)?
        .ok_or(AppError::NotFound)?;

    let mut tx = pool.begin().await.map_err(db)?;
    let found: Vec<u64> = sqlx::query_scalar(
        "SELECT id FROM patients WHERE clinic_id = ? AND id IN (?, ?) AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(clinic_id)
    .bind(keep_id)
    .bind(merge_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db)?;
    if found.len() != 2 {
        return Err(AppError::NotFound);
    }

    for table in PATIENT_TABLES {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE {table} SET patient_id = ? WHERE patient_id = ?"
        )))
        .bind(keep_id)
        .bind(merge_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    }
    sqlx::query("UPDATE ai_analyses SET patient_id = ? WHERE patient_id = ?")
        .bind(keep_id)
        .bind(merge_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;

    // Visit numbers follow the combined history in date order.
    let ids: Vec<u64> =
        sqlx::query_scalar("SELECT id FROM visits WHERE patient_id = ? ORDER BY visit_date, id")
            .bind(keep_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(db)?;
    for (i, id) in ids.iter().enumerate() {
        sqlx::query("UPDATE visits SET visit_no = ? WHERE id = ?")
            .bind(i as u32 + 1)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }

    sqlx::query("UPDATE patients SET deleted_at = UTC_TIMESTAMP(), merged_into = ? WHERE id = ?")
        .bind(keep_id)
        .bind(merge_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;

    let combined = merge_profiles(keep_profile, other_profile).validate()?;
    medical_profile::put(pool, keep_id, &combined)
        .await
        .map_err(db)?;
    sqlx::query("DELETE FROM patient_medical_profile WHERE patient_id = ?")
        .bind(merge_id)
        .execute(pool)
        .await
        .map_err(db)?;
    Ok(())
}

/// Counts shown in the merge preview, so the admin sees what moves.
#[derive(Debug, Default, serde::Serialize)]
pub struct Counts {
    pub visits: i64,
    pub reports: i64,
    pub appointments: i64,
    pub bills: i64,
}

pub async fn counts(pool: &MySqlPool, patient_id: u64) -> Result<Counts, sqlx::Error> {
    let (visits, reports, appointments, bills): (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT CAST(COUNT(*) AS SIGNED) FROM visits WHERE patient_id = ? AND deleted_at IS NULL),
            (SELECT CAST(COUNT(*) AS SIGNED) FROM reports WHERE patient_id = ? AND deleted_at IS NULL),
            (SELECT CAST(COUNT(*) AS SIGNED) FROM appointments WHERE patient_id = ?),
            (SELECT CAST(COUNT(*) AS SIGNED) FROM bills b JOIN visits v ON v.id = b.visit_id WHERE v.patient_id = ?)",
    )
    .bind(patient_id)
    .bind(patient_id)
    .bind(patient_id)
    .bind(patient_id)
    .fetch_one(pool)
    .await?;
    Ok(Counts {
        visits,
        reports,
        appointments,
        bills,
    })
}
