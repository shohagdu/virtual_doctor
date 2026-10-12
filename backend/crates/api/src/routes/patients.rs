use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use vd_common::{AppError, AppResult};
use vd_domain::patient::{NewPatientInput, Patient};

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    q: String,
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
) -> AppResult<Json<Page<Patient>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);
    let (items, total) = vd_db::patients::search(&state.pool, user.clinic_id, &q.q, page, per_page)
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
    let new = input.validate(Utc::now().date_naive())?;

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
