mod auth;
mod health;
mod me;
mod medicines;
mod patients;
mod queue;
mod visits;

use axum::Router;
use axum::routing::{get, patch, post};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/login", post(auth::login))
        .route("/me", get(me::show).patch(me::update))
        .route("/clinic", patch(me::update_clinic))
        .route("/patients", get(patients::list).post(patients::create))
        .route(
            "/patients/{id}",
            get(patients::show).patch(patients::update),
        )
        .route(
            "/patients/{id}/medical-profile",
            get(patients::medical_profile).put(patients::put_medical_profile),
        )
        .route(
            "/patients/{id}/visits",
            get(visits::timeline).post(visits::start),
        )
        .route("/patients/{id}/vitals/trend", get(visits::vitals_trend))
        .route("/queue/today", get(queue::today).post(queue::add))
        .route("/queue/next", post(queue::next))
        .route("/queue/{id}", patch(queue::update))
        .route("/visits/{id}", get(visits::show))
        .route(
            "/visits/{id}/vitals",
            get(visits::vitals).put(visits::save_vitals),
        )
        .route("/visits/{id}/allergy-check", post(visits::allergy_check))
        .route(
            "/visits/{id}/prescription",
            get(visits::prescription).put(visits::save_prescription),
        )
        .route(
            "/visits/{id}/prescription/lock",
            post(visits::lock_prescription),
        )
        .route("/medicines/search", get(medicines::search));

    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", api)
        .with_state(state)
}
