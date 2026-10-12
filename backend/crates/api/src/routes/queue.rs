use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::clinic_today;
use vd_domain::visit::{QueueCounters, QueueEntry, Visit, VisitStatus, VisitType};

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Serialize)]
pub struct Today {
    counters: QueueCounters,
    items: Vec<QueueEntry>,
}

/// Today's queue with the dashboard counters (§5.1).
pub async fn today(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Today>> {
    let day = clinic_today();
    let items = vd_db::queue::today(&state.pool, user.clinic_id, day)
        .await
        .map_err(AppError::internal)?;
    let counters = vd_db::queue::counters(&state.pool, user.clinic_id, day)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(Today { counters, items }))
}

#[derive(Deserialize)]
pub struct AddToQueue {
    patient_id: u64,
    #[serde(default)]
    visit_type: VisitType,
}

/// Adds a patient to today's queue as waiting. A patient already waiting or in
/// consultation today is not added again.
pub async fn add(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<AddToQueue>,
) -> AppResult<(StatusCode, Json<Visit>)> {
    let day = clinic_today();
    if vd_db::queue::open_visit(&state.pool, user.clinic_id, req.patient_id, day)
        .await
        .map_err(AppError::internal)?
        .is_some()
    {
        return Err(AppError::Conflict(
            "this patient is already in today's queue".into(),
        ));
    }
    let visit = vd_db::visits::create(
        &state.pool,
        user.clinic_id,
        req.patient_id,
        req.visit_type,
        VisitStatus::Waiting,
        None,
        day,
    )
    .await
    .map_err(AppError::internal)?
    .ok_or(AppError::NotFound)?;
    user.audit(&state, "visit", visit.id, "create").await?;
    Ok((StatusCode::CREATED, Json(visit)))
}

#[derive(Deserialize)]
pub struct UpdateQueue {
    status: Option<VisitStatus>,
    /// New 0-based place among the waiting patients.
    position: Option<usize>,
}

/// Changes a queue entry's status and/or its place in the waiting order.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<UpdateQueue>,
) -> AppResult<Json<Visit>> {
    if let Some(to) = req.status {
        if to.doctor_only() {
            user.require(&[Role::Doctor])?;
        }
        let doctor = (user.role == Role::Doctor).then_some(user.id);
        vd_db::queue::set_status(&state.pool, user.clinic_id, id, to, doctor).await?;
        user.audit(&state, "visit", id, to.as_str()).await?;
    }
    if let Some(position) = req.position {
        vd_db::queue::move_to(&state.pool, user.clinic_id, id, position).await?;
        user.audit(&state, "visit", id, "reorder").await?;
    }
    let visit = vd_db::visits::get(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(visit))
}

#[derive(Deserialize)]
pub struct CallNext {
    current_visit_id: Option<u64>,
}

#[derive(Serialize)]
pub struct Next {
    next: Option<Visit>,
}

/// Save & Next Patient (§5.5): finishes the current visit and calls the next one in.
pub async fn next(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CallNext>,
) -> AppResult<Json<Next>> {
    user.require(&[Role::Doctor])?;
    let next = vd_db::queue::call_next(
        &state.pool,
        user.clinic_id,
        clinic_today(),
        req.current_visit_id,
        user.id,
    )
    .await?;
    if let Some(cur) = req.current_visit_id {
        user.audit(&state, "visit", cur, "seen").await?;
    }
    if let Some(v) = &next {
        user.audit(&state, "visit", v.id, "in_consultation").await?;
    }
    Ok(Json(Next { next }))
}
