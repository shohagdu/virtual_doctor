//! Admin: merge duplicate patients (PM-19, §13 — preview both histories first; the
//! merge is logged).

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::patient::Patient;
use vd_domain::visit::VisitSummary;

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct Pair {
    keep: u64,
    merge: u64,
}

#[derive(Serialize)]
pub struct Side {
    patient: Patient,
    counts: vd_db::merge::Counts,
    visits: Vec<VisitSummary>,
}

#[derive(Serialize)]
pub struct Preview {
    keep: Side,
    merge: Side,
}

async fn side(state: &AppState, clinic_id: u64, id: u64) -> AppResult<Side> {
    let patient = vd_db::patients::get(&state.pool, clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let counts = vd_db::merge::counts(&state.pool, id)
        .await
        .map_err(AppError::internal)?;
    // Admins see dates and visit types, not clinical details (§4).
    let visits = vd_db::visits::timeline(&state.pool, clinic_id, id, false)
        .await
        .map_err(AppError::internal)?;
    Ok(Side {
        patient,
        counts,
        visits,
    })
}

/// Both records side by side before merging.
pub async fn preview(
    State(state): State<AppState>,
    user: AuthUser,
    Query(p): Query<Pair>,
) -> AppResult<Json<Preview>> {
    user.require(&[Role::Admin])?;
    if p.keep == p.merge {
        return Err(AppError::Validation("choose two different patients".into()));
    }
    Ok(Json(Preview {
        keep: side(&state, user.clinic_id, p.keep).await?,
        merge: side(&state, user.clinic_id, p.merge).await?,
    }))
}

/// Merges `merge` into `keep`; all history moves, nothing is deleted.
pub async fn merge(
    State(state): State<AppState>,
    user: AuthUser,
    Json(p): Json<Pair>,
) -> AppResult<StatusCode> {
    user.require(&[Role::Admin])?;
    vd_db::merge::merge(&state.pool, user.clinic_id, p.keep, p.merge).await?;
    user.audit(&state, "patient", p.merge, "merged_into")
        .await?;
    user.audit(&state, "patient", p.keep, "merged_from").await?;
    Ok(StatusCode::NO_CONTENT)
}
