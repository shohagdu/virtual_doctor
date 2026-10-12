use axum::Json;
use axum::extract::State;
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::settings::ClinicSettings;

use crate::auth::AuthUser;
use crate::state::AppState;

/// Fees, chamber schedule and SMS settings (§12). Everyone reads them (fees are
/// needed at the front desk); doctor and admin change them.
pub async fn show(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ClinicSettings>> {
    let s = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(s))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Json(mut req): Json<ClinicSettings>,
) -> AppResult<Json<ClinicSettings>> {
    user.require(&[Role::Doctor, Role::Admin])?;
    req.validate()?;
    vd_db::settings::save(&state.pool, user.clinic_id, &req)
        .await
        .map_err(AppError::internal)?;
    user.audit(&state, "clinic_settings", user.clinic_id, "update")
        .await?;
    Ok(Json(req))
}
