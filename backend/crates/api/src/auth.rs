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
    pub role: Role,
}

impl AuthUser {
    /// Role check per route (Patient_Management_Spec §4).
    pub fn require(&self, allowed: &[Role]) -> Result<(), AppError> {
        if allowed.contains(&self.role) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    /// Writes an audit trail entry (PM-21, §11).
    pub async fn audit(
        &self,
        state: &AppState,
        entity: &str,
        id: u64,
        action: &str,
    ) -> Result<(), AppError> {
        vd_db::audit::log(
            &state.pool,
            self.clinic_id,
            Some(self.id),
            entity,
            Some(id),
            action,
        )
        .await
        .map_err(AppError::internal)
    }
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
