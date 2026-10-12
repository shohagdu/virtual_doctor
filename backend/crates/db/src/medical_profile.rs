use sqlx::types::Json;
use vd_domain::medical_profile::MedicalProfile;

use crate::{DbResult, MySqlPool};

#[derive(sqlx::FromRow)]
struct Row {
    allergies: Option<Json<Vec<String>>>,
    chronic_conditions: Option<Json<Vec<String>>>,
    long_term_medicines: Option<Json<Vec<String>>>,
    vaccination: Option<Json<Vec<String>>>,
    past_history: Option<String>,
    family_history: Option<String>,
    social_history: Option<String>,
    gynae_obs_history: Option<String>,
    pregnancy_status: Option<String>,
}

fn list(v: Option<Json<Vec<String>>>) -> Vec<String> {
    v.map(|j| j.0).unwrap_or_default()
}

/// Returns `None` if the patient does not exist in this clinic.
pub async fn get(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
) -> DbResult<Option<MedicalProfile>> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT m.allergies, m.chronic_conditions, m.long_term_medicines, m.vaccination,
                m.past_history, m.family_history, m.social_history, m.gynae_obs_history,
                m.pregnancy_status
         FROM patients p
         LEFT JOIN patient_medical_profile m ON m.patient_id = p.id
         WHERE p.clinic_id = ? AND p.id = ? AND p.deleted_at IS NULL",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| MedicalProfile {
        allergies: list(r.allergies),
        chronic_conditions: list(r.chronic_conditions),
        long_term_medicines: list(r.long_term_medicines),
        vaccination: list(r.vaccination),
        past_history: r.past_history,
        family_history: r.family_history,
        social_history: r.social_history,
        gynae_obs_history: r.gynae_obs_history,
        pregnancy_status: r.pregnancy_status,
    }))
}

/// Upserts the profile. The caller must have checked that the patient is in the clinic.
pub async fn put(pool: &MySqlPool, patient_id: u64, m: &MedicalProfile) -> DbResult<()> {
    sqlx::query(
        "INSERT INTO patient_medical_profile (patient_id, allergies, chronic_conditions,
            long_term_medicines, vaccination, past_history, family_history, social_history,
            gynae_obs_history, pregnancy_status)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) AS new
         ON DUPLICATE KEY UPDATE
            allergies = new.allergies, chronic_conditions = new.chronic_conditions,
            long_term_medicines = new.long_term_medicines, vaccination = new.vaccination,
            past_history = new.past_history, family_history = new.family_history,
            social_history = new.social_history, gynae_obs_history = new.gynae_obs_history,
            pregnancy_status = new.pregnancy_status",
    )
    .bind(patient_id)
    .bind(Json(&m.allergies))
    .bind(Json(&m.chronic_conditions))
    .bind(Json(&m.long_term_medicines))
    .bind(Json(&m.vaccination))
    .bind(&m.past_history)
    .bind(&m.family_history)
    .bind(&m.social_history)
    .bind(&m.gynae_obs_history)
    .bind(&m.pregnancy_status)
    .execute(pool)
    .await?;
    Ok(())
}
