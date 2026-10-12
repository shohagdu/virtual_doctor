mod auth;
mod health;
mod patients;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/login", post(auth::login))
        .route("/patients", get(patients::list).post(patients::create))
        .route("/patients/{id}", get(patients::show));

    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", api)
        .with_state(state)
}
