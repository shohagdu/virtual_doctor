use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use vd_common::AppError;
use vd_common::auth::{Role, decode_token};

use crate::state::AppState;

/// The authenticated user, taken from the `Authorization: Bearer` header.
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: u64,
    pub clinic_id: u64,
    #[allow(dead_code)] // per-route role checks arrive with the clinical endpoints
    pub role: Role,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;
        let claims = decode_token(token, &state.config.jwt_secret)?;
        Ok(Self {
            id: claims.sub,
            clinic_id: claims.clinic_id,
            role: claims.role,
        })
    }
}
