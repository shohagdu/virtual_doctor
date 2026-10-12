mod admin;
mod ai;
mod analysis;
mod appointments;
mod auth;
mod bills;
mod documents;
mod followups;
mod health;
mod me;
mod medicines;
mod notify;
mod patients;
mod queue;
mod reports;
mod settings;
mod summary;
mod templates;
mod visits;

use axum::Router;
use axum::routing::{delete, get, patch, post, put};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/login", post(auth::login))
        .route("/me", get(me::show).patch(me::update))
        .route("/clinic", patch(me::update_clinic))
        .route("/settings", get(settings::show).put(settings::update))
        .route("/patients", get(patients::list).post(patients::create))
        .route("/patients/smart-search", get(summary::smart_search))
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
        .route("/patients/{id}/reports", get(reports::for_patient))
        .route("/patients/{id}/lab-trends", get(reports::trends))
        .route("/patients/{id}/ai-summary", post(summary::generate))
        .route("/patients/{id}/ai-summary/latest", get(summary::latest))
        .route(
            "/visits/{id}/reports",
            get(reports::for_visit).post(reports::upload),
        )
        .route("/reports/{id}", get(reports::show).delete(reports::delete))
        .route("/reports/{id}/file", get(reports::file))
        .route("/reports/{id}/extract", post(reports::extract))
        .route("/reports/{id}/results", put(reports::save_results))
        .route("/visits/{id}/ai-analysis", post(analysis::run))
        .route("/visits/{id}/ai-analysis/latest", get(analysis::latest))
        .route("/ai-suggestions/{id}/action", post(analysis::act))
        .route(
            "/visits/{id}/prescription/safety-check",
            post(analysis::safety),
        )
        .route("/ai/status", get(ai::status))
        .route("/admin/ai-usage", get(ai::usage))
        .route("/followups/{id}/sms/ai-draft", post(summary::sms_draft))
        .route(
            "/patients/{id}/appointments",
            get(appointments::for_patient),
        )
        .route(
            "/appointments",
            get(appointments::list).post(appointments::create),
        )
        .route("/appointments/slots", get(appointments::slots))
        .route("/appointments/{id}", patch(appointments::update))
        .route("/followups", get(followups::list))
        .route("/followups/{id}", patch(followups::update))
        .route(
            "/followups/{id}/sms",
            get(followups::sms_draft).post(followups::send_sms),
        )
        .route("/sms/{id}/retry", post(followups::retry_sms))
        .route("/reports/collection", get(bills::collection))
        .route("/queue/today", get(queue::today).post(queue::add))
        .route("/queue/next", post(queue::next))
        .route("/queue/{id}", patch(queue::update))
        .route("/visits/{id}", get(visits::show))
        .route(
            "/visits/{id}/vitals",
            get(visits::vitals).put(visits::save_vitals),
        )
        .route("/visits/{id}/allergy-check", post(visits::allergy_check))
        .route("/visits/{id}/bill", get(bills::show).put(bills::save))
        .route(
            "/visits/{id}/prescription",
            get(visits::prescription).put(visits::save_prescription),
        )
        .route(
            "/visits/{id}/prescription/lock",
            post(visits::lock_prescription),
        )
        .route("/medicines/search", get(medicines::search))
        .route("/medicines/catalogue", get(medicines::catalogue))
        .route("/medicines/import", post(medicines::import))
        .route("/rx-templates", get(templates::list).post(templates::save))
        .route("/rx-templates/{id}", delete(templates::delete))
        .route(
            "/medicine-favorites",
            get(templates::favorites).post(templates::add_favorite),
        )
        .route(
            "/medicine-favorites/{id}",
            delete(templates::delete_favorite),
        )
        .route(
            "/visits/{id}/documents",
            get(documents::for_visit).post(documents::create),
        )
        .route("/patients/{id}/bills", get(bills::for_patient))
        .route("/admin/patients/merge", post(admin::merge))
        .route("/admin/patients/merge-preview", get(admin::preview));

    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", api)
        .with_state(state)
}

pub use ai::register_prompts;
