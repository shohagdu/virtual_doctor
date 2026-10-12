//! Prescription templates and favourite medicines (roadmap 2.1, 2.3). Doctor only.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_db::favorites::Favorite;
use vd_db::templates::Template;
use vd_domain::clinic_today;
use vd_domain::prescription::{MedicineLine, PrescriptionBody};
use vd_domain::text::check_len;

use crate::auth::AuthUser;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Vec<Template>>> {
    user.require(&[Role::Doctor])?;
    let items = vd_db::templates::list(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(items))
}

#[derive(Deserialize)]
pub struct SaveTemplate {
    name: String,
    body: PrescriptionBody,
}

/// Saves the current prescription's sections as a named template (replacing one
/// with the same name). The Next Visit date is never part of a template.
pub async fn save(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SaveTemplate>,
) -> AppResult<(StatusCode, Json<Vec<Template>>)> {
    user.require(&[Role::Doctor])?;
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("give the template a name".into()));
    }
    check_len(&name, "template name", 100)?;
    let mut body = req.body;
    body.next_visit_date = None;
    let body = body.validate(clinic_today())?;
    let id = vd_db::templates::upsert(&state.pool, user.clinic_id, &name, &body, user.id)
        .await
        .map_err(AppError::internal)?;
    user.audit(&state, "rx_template", id, "save").await?;
    list(State(state), user)
        .await
        .map(|j| (StatusCode::CREATED, j))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<StatusCode> {
    user.require(&[Role::Doctor])?;
    if !vd_db::templates::delete(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
    {
        return Err(AppError::NotFound);
    }
    user.audit(&state, "rx_template", id, "delete").await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn favorites(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<Favorite>>> {
    user.require(&[Role::Doctor])?;
    let items = vd_db::favorites::list(&state.pool, user.id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(items))
}

/// Saves a medicine line (brand, strength and usual dose) as a favourite.
pub async fn add_favorite(
    State(state): State<AppState>,
    user: AuthUser,
    Json(line): Json<MedicineLine>,
) -> AppResult<(StatusCode, Json<Vec<Favorite>>)> {
    user.require(&[Role::Doctor])?;
    // Validated like a prescription line, through a one-line body.
    let body = PrescriptionBody {
        medicines: vec![line],
        ..Default::default()
    }
    .validate(clinic_today())?;
    let Some(line) = body.medicines.into_iter().next() else {
        return Err(AppError::Validation(
            "the medicine needs a brand name".into(),
        ));
    };
    vd_db::favorites::add(&state.pool, user.id, &line)
        .await
        .map_err(AppError::internal)?;
    favorites(State(state), user)
        .await
        .map(|j| (StatusCode::CREATED, j))
}

pub async fn delete_favorite(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<StatusCode> {
    user.require(&[Role::Doctor])?;
    if !vd_db::favorites::delete(&state.pool, user.id, id)
        .await
        .map_err(AppError::internal)?
    {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
