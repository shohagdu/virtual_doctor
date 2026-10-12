use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_db::clinics::Clinic;
use vd_db::users::Profile;
use vd_domain::text::{check_len, opt};

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Serialize)]
pub struct MeResponse {
    user: Profile,
    clinic: Clinic,
}

pub async fn show(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<MeResponse>> {
    let profile = vd_db::users::profile(&state.pool, user.id)
        .await
        .map_err(AppError::internal)?;
    let clinic = vd_db::clinics::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(MeResponse {
        user: profile,
        clinic,
    }))
}

#[derive(Deserialize)]
pub struct UpdateMe {
    name: String,
    phone: Option<String>,
    degrees: Option<String>,
    bmdc_reg_no: Option<String>,
}

fn required(v: String, field: &str, max: usize) -> AppResult<String> {
    let v = v.trim().to_string();
    if v.is_empty() {
        return Err(AppError::Validation(format!("{field} is required")));
    }
    check_len(&v, field, max)?;
    Ok(v)
}

/// Updates the signed-in user's name and the doctor details printed on prescriptions.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UpdateMe>,
) -> AppResult<Json<MeResponse>> {
    let name = required(req.name, "name", 150)?;
    let phone = opt(req.phone, "phone", 15)?;
    let degrees = opt(req.degrees, "degrees", 255)?;
    let bmdc = opt(req.bmdc_reg_no, "bmdc_reg_no", 30)?;
    vd_db::users::update_profile(
        &state.pool,
        user.id,
        &name,
        phone.as_deref(),
        degrees.as_deref(),
        bmdc.as_deref(),
    )
    .await
    .map_err(AppError::internal)?;
    user.audit(&state, "user", user.id, "update").await?;
    show(State(state), user).await
}

/// Clinic name/address/phone for the prescription header (doctor or admin).
pub async fn update_clinic(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<Clinic>,
) -> AppResult<Json<MeResponse>> {
    user.require(&[Role::Doctor, Role::Admin])?;
    let clinic = Clinic {
        name: required(req.name, "clinic name", 150)?,
        address: opt(req.address, "address", 255)?,
        phone: opt(req.phone, "phone", 15)?,
    };
    vd_db::clinics::update(&state.pool, user.clinic_id, &clinic)
        .await
        .map_err(AppError::internal)?;
    user.audit(&state, "clinic", user.clinic_id, "update")
        .await?;
    show(State(state), user).await
}
