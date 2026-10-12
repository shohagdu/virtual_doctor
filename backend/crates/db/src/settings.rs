//! Clinic settings stored as JSON in `clinics.settings` (§12).

use sqlx::types::Json;
use vd_domain::settings::ClinicSettings;

use crate::{DbResult, MySqlPool};

/// The clinic's settings; anything never saved takes its default.
pub async fn get(pool: &MySqlPool, clinic_id: u64) -> DbResult<ClinicSettings> {
    let raw: Option<Option<Json<serde_json::Value>>> =
        sqlx::query_scalar("SELECT settings FROM clinics WHERE id = ?")
            .bind(clinic_id)
            .fetch_optional(pool)
            .await?;
    let Some(Json(value)) = raw.flatten() else {
        return Ok(ClinicSettings::default());
    };
    Ok(serde_json::from_value(value).unwrap_or_else(|e| {
        tracing::warn!(clinic_id, error = %e, "unreadable clinic settings; using defaults");
        ClinicSettings::default()
    }))
}

pub async fn save(pool: &MySqlPool, clinic_id: u64, settings: &ClinicSettings) -> DbResult<()> {
    sqlx::query("UPDATE clinics SET settings = ? WHERE id = ?")
        .bind(Json(settings))
        .bind(clinic_id)
        .execute(pool)
        .await?;
    Ok(())
}
