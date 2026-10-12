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
