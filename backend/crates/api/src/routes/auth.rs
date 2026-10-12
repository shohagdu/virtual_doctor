use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use vd_common::auth::{Role, issue_token, verify_password};
use vd_common::{AppError, AppResult};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserDto {
    id: u64,
    name: String,
    email: String,
    role: Role,
}

#[derive(Serialize)]
pub struct LoginResponse {
    access_token: String,
    user: UserDto,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let user = vd_db::users::find_by_email(&state.pool, req.email.trim())
        .await
        .map_err(AppError::internal)?
        .filter(|u| u.is_active && verify_password(&req.password, &u.password_hash))
        .ok_or(AppError::Unauthorized)?;
    let role = Role::parse(&user.role).ok_or_else(|| AppError::internal("bad role in db"))?;

    let access_token = issue_token(
        user.id,
        user.clinic_id,
        role,
        &state.config.jwt_secret,
        state.config.jwt_ttl_minutes,
    )?;
    vd_db::audit::log(
        &state.pool,
        user.clinic_id,
        Some(user.id),
        "user",
        Some(user.id),
        "login",
    )
    .await
    .map_err(AppError::internal)?;

    Ok(Json(LoginResponse {
        access_token,
        user: UserDto {
            id: user.id,
            name: user.name,
            email: user.email,
            role,
        },
    }))
}
