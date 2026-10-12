use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_db::prescriptions::SaveContext;
use vd_domain::clinic_today;
use vd_domain::medical_profile::MedicalProfile;
use vd_domain::patient::Patient;
use vd_domain::prescription::{BaseVersion, Prescription, SavePrescription};
use vd_domain::visit::{Visit, VisitStatus, VisitSummary, VisitType};

use crate::auth::AuthUser;
use crate::state::AppState;

/// Visit timeline. Doctors see clinical details; assistants get the limited view (§4).
pub async fn timeline(
    State(state): State<AppState>,
    user: AuthUser,
    Path(patient_id): Path<u64>,
) -> AppResult<Json<Vec<VisitSummary>>> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    vd_db::patients::get(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let clinical = user.role == Role::Doctor;
    let items = vd_db::visits::timeline(&state.pool, user.clinic_id, patient_id, clinical)
        .await
        .map_err(AppError::internal)?;
    user.audit(&state, "patient", patient_id, "view_history")
        .await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
pub struct StartVisit {
    #[serde(default)]
    visit_type: VisitType,
}

/// Starts a visit (PM-06). A doctor starting it goes straight into consultation;
/// an assistant or admin adds the patient to today's queue.
pub async fn start(
    State(state): State<AppState>,
    user: AuthUser,
    Path(patient_id): Path<u64>,
    Json(req): Json<StartVisit>,
) -> AppResult<(StatusCode, Json<Visit>)> {
    let (status, doctor_id) = match user.role {
        Role::Doctor => (VisitStatus::InConsultation, Some(user.id)),
        _ => (VisitStatus::Waiting, None),
    };
    let visit = vd_db::visits::create(
        &state.pool,
        user.clinic_id,
        patient_id,
        req.visit_type,
        status,
        doctor_id,
        clinic_today(),
    )
    .await
    .map_err(AppError::internal)?
    .ok_or(AppError::NotFound)?;
    user.audit(&state, "visit", visit.id, "create").await?;
    Ok((StatusCode::CREATED, Json(visit)))
}

/// Everything the prescription screen's patient bar needs (PM-07, §5.5).
#[derive(Serialize)]
pub struct VisitContext {
    visit: Visit,
    patient: Patient,
    profile: MedicalProfile,
    previous_visit_date: Option<NaiveDate>,
}

async fn load_visit(state: &AppState, user: &AuthUser, id: u64) -> AppResult<Visit> {
    vd_db::visits::get(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)
}

pub async fn show(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<VisitContext>> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let visit = load_visit(&state, &user, id).await?;
    let patient = vd_db::patients::get(&state.pool, user.clinic_id, visit.patient_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let profile = vd_db::medical_profile::get(&state.pool, user.clinic_id, visit.patient_id)
        .await
        .map_err(AppError::internal)?
        .unwrap_or_default();
    let previous_visit_date =
        vd_db::visits::previous_visit_date(&state.pool, visit.patient_id, visit.id)
            .await
            .map_err(AppError::internal)?;
    user.audit(&state, "visit", id, "view").await?;
    Ok(Json(VisitContext {
        visit,
        patient,
        profile,
        previous_visit_date,
    }))
}

pub async fn prescription(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<Prescription>> {
    user.require(&[Role::Doctor])?;
    load_visit(&state, &user, id).await?;
    let rx = vd_db::prescriptions::latest(&state.pool, id).await?;
    user.audit(&state, "prescription", id, "view").await?;
    Ok(Json(rx))
}

pub async fn save_prescription(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<SavePrescription>,
) -> AppResult<Json<Prescription>> {
    user.require(&[Role::Doctor])?;
    let visit = load_visit(&state, &user, id).await?;
    if visit.status == VisitStatus::Cancelled {
        return Err(AppError::Conflict("this visit was cancelled".into()));
    }
    let body = req.body.validate(clinic_today())?;
    let ctx = SaveContext {
        clinic_id: user.clinic_id,
        patient_id: visit.patient_id,
        visit_id: id,
        user_id: user.id,
    };
    let rx = vd_db::prescriptions::save(&state.pool, &ctx, req.base, &body).await?;
    user.audit(&state, "prescription", id, "save").await?;
    Ok(Json(rx))
}

pub async fn lock_prescription(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(base): Json<BaseVersion>,
) -> AppResult<Json<Prescription>> {
    user.require(&[Role::Doctor])?;
    load_visit(&state, &user, id).await?;
    let rx = vd_db::prescriptions::lock(&state.pool, id, base).await?;
    user.audit(&state, "prescription", id, "lock").await?;
    Ok(Json(rx))
}
