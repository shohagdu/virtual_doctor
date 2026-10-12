use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_db::medicines::MedicineSuggestion;

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    q: String,
}

pub async fn search(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<Vec<MedicineSuggestion>>> {
    user.require(&[Role::Doctor])?;
    let items = vd_db::medicines::search(&state.pool, user.clinic_id, &q.q, 15)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(items))
}

#[derive(serde::Serialize)]
pub struct Catalogue {
    clinic_rows: i64,
    shared_rows: i64,
}

/// How many catalogue medicines the clinic has (its own and shared).
pub async fn catalogue(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Catalogue>> {
    user.require(&[Role::Doctor, Role::Admin])?;
    let (clinic_rows, shared_rows) = vd_db::medicines::catalogue_size(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(Catalogue {
        clinic_rows,
        shared_rows,
    }))
}

#[derive(serde::Serialize)]
pub struct Imported {
    added: usize,
    updated: usize,
    problems: Vec<vd_domain::catalogue::ImportProblem>,
    /// Problems beyond the first ones listed.
    more_problems: usize,
}

/// Imports the clinic's medicine list from CSV (roadmap 2.2): header row naming
/// brand, generic, strength, form, company, price. Rows are matched on brand and
/// strength, so importing again updates them.
pub async fn import(
    State(state): State<AppState>,
    user: AuthUser,
    body: String,
) -> AppResult<Json<Imported>> {
    user.require(&[Role::Doctor, Role::Admin])?;
    let (rows, problems) = vd_domain::catalogue::parse(&body).map_err(AppError::Validation)?;
    let (added, updated) = vd_db::medicines::import(&state.pool, user.clinic_id, &rows)
        .await
        .map_err(AppError::internal)?;
    user.audit(&state, "medicine_catalogue", user.clinic_id, "import")
        .await?;
    let more_problems = problems.len().saturating_sub(20);
    Ok(Json(Imported {
        added,
        updated,
        problems: problems.into_iter().take(20).collect(),
        more_problems,
    }))
}
