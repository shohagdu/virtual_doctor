use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Doctor,
    Assistant,
    Admin,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Assistant => "assistant",
            Self::Admin => "admin",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "doctor" => Some(Self::Doctor),
            "assistant" => Some(Self::Assistant),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }
}

/// JWT access-token claims.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u64,
    pub clinic_id: u64,
    pub role: Role,
    pub exp: i64,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|h| h.to_string())
        .map_err(AppError::internal)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|h| {
        Argon2::default()
            .verify_password(password.as_bytes(), &h)
            .is_ok()
    })
}

pub fn issue_token(
    user_id: u64,
    clinic_id: u64,
    role: Role,
    secret: &str,
    ttl_minutes: i64,
) -> Result<String, AppError> {
    let claims = Claims {
        sub: user_id,
        clinic_id,
        role,
        exp: (Utc::now() + chrono::Duration::minutes(ttl_minutes)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::internal)
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_roundtrip() {
        let hash = hash_password("s3cret").unwrap();
        assert!(verify_password("s3cret", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn token_roundtrip() {
        let token = issue_token(7, 1, Role::Doctor, "test-secret", 5).unwrap();
        let claims = decode_token(&token, "test-secret").unwrap();
        assert_eq!(claims.sub, 7);
        assert_eq!(claims.role, Role::Doctor);
        assert!(decode_token(&token, "other-secret").is_err());
    }
}
