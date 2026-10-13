use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::clinic_today;
use vd_domain::medical_profile::MedicalProfile;
use vd_domain::patient::{NewPatientInput, Patient};

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    q: String,
    sex: Option<String>,
    #[serde(default)]
    sort: vd_domain::patient::PatientSort,
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Serialize)]
pub struct Page<T> {
    items: Vec<T>,
    total: i64,
    page: u32,
    per_page: u32,
}

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Page<vd_domain::patient::PatientListItem>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);
    let filter = vd_db::patients::SearchFilter {
        q: &q.q,
        sex: q.sex.as_deref().and_then(vd_domain::patient::Sex::parse),
        sort: q.sort,
        page,
        per_page,
    };
    let (items, total) = vd_db::patients::search(&state.pool, user.clinic_id, &filter)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(Page {
        items,
        total,
        page,
        per_page,
    }))
}

pub async fn show(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<Patient>> {
    let patient = vd_db::patients::get(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    vd_db::audit::log(
        &state.pool,
        user.clinic_id,
        Some(user.id),
        "patient",
        Some(id),
        "view",
    )
    .await
    .map_err(AppError::internal)?;
    Ok(Json(patient))
}

#[derive(Deserialize)]
pub struct CreateQuery {
    #[serde(default)]
    force: bool,
}

#[derive(Serialize)]
pub struct CreateResponse {
    patient: Option<Patient>,
    duplicates: Vec<Patient>,
}

/// Registers a patient. Without `?force=true`, possible duplicates are returned
/// instead of saving, so staff can pick "Use existing" or "Create new" (§13).
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Query(cq): Query<CreateQuery>,
    Json(input): Json<NewPatientInput>,
) -> AppResult<(StatusCode, Json<CreateResponse>)> {
    let new = input.validate(clinic_today())?;

    if !cq.force {
        let duplicates =
            vd_db::patients::find_duplicates(&state.pool, user.clinic_id, &new.phone, &new.name_en)
                .await
                .map_err(AppError::internal)?;
        if !duplicates.is_empty() {
            return Ok((
                StatusCode::OK,
                Json(CreateResponse {
                    patient: None,
                    duplicates,
                }),
            ));
        }
    }

    let patient = vd_db::patients::create(&state.pool, user.clinic_id, user.id, &new)
        .await
        .map_err(AppError::internal)?;
    vd_db::audit::log(
        &state.pool,
        user.clinic_id,
        Some(user.id),
        "patient",
        Some(patient.id),
        "create",
    )
    .await
    .map_err(AppError::internal)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateResponse {
            patient: Some(patient),
            duplicates: vec![],
        }),
    ))
}

/// Edits demographics (any role, §4).
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(input): Json<NewPatientInput>,
) -> AppResult<Json<Patient>> {
    let new = input.validate(clinic_today())?;
    let patient = vd_db::patients::update(&state.pool, user.clinic_id, id, &new)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    user.audit(&state, "patient", id, "update").await?;
    Ok(Json(patient))
}

/// Allergies, chronic conditions and histories. Clinical data, so not for Admin (§4).
pub async fn medical_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<MedicalProfile>> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let profile = vd_db::medical_profile::get(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    user.audit(&state, "medical_profile", id, "view").await?;
    Ok(Json(profile))
}

pub async fn put_medical_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(input): Json<MedicalProfile>,
) -> AppResult<Json<MedicalProfile>> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let profile = input.validate()?;
    vd_db::patients::get(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    vd_db::medical_profile::put(&state.pool, id, &profile)
        .await
        .map_err(AppError::internal)?;
    user.audit(&state, "medical_profile", id, "update").await?;
    Ok(Json(profile))
}
