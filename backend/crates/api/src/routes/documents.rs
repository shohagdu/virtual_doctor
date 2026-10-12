//! Medical certificates, sick leave and referral letters (roadmap 2.9). Doctor only.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::document::{Document, DocumentBody, DocumentKind};

use crate::auth::AuthUser;
use crate::state::AppState;

pub async fn for_visit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
) -> AppResult<Json<Vec<Document>>> {
    user.require(&[Role::Doctor])?;
    let items = vd_db::documents::for_visit(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(items))
}

#[derive(Deserialize)]
pub struct NewDocument {
    kind: DocumentKind,
    body: DocumentBody,
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
    Json(req): Json<NewDocument>,
) -> AppResult<(StatusCode, Json<Document>)> {
    user.require(&[Role::Doctor])?;
    let visit = vd_db::visits::get(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let body = req.body.validate(req.kind)?;
    let id = vd_db::documents::create(
        &state.pool,
        user.clinic_id,
        visit.patient_id,
        visit_id,
        req.kind,
        &body,
        user.id,
    )
    .await
    .map_err(AppError::internal)?;
    user.audit(&state, "document", id, req.kind.as_str())
        .await?;
    let doc = vd_db::documents::get(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    Ok((StatusCode::CREATED, Json(doc)))
}
