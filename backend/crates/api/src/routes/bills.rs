use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::billing::{Bill, BillInput, CollectionRow, CollectionTotals, totals};

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Serialize)]
pub struct VisitBill {
    bill: Option<Bill>,
    /// The fee for this visit type from Settings, to pre-fill a new bill.
    suggested_fee: u32,
}

/// The visit's bill, if any (§5.8). Doctor, assistant and admin all collect fees.
pub async fn show(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
) -> AppResult<Json<VisitBill>> {
    let visit = vd_db::visits::get(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let settings = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    let bill = vd_db::bills::for_visit(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(VisitBill {
        bill,
        suggested_fee: settings.fees.for_type(visit.visit_type),
    }))
}

/// Records the fee and payment; saving again updates it (e.g. a due paid later).
pub async fn save(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
    Json(req): Json<BillInput>,
) -> AppResult<Json<Bill>> {
    let amounts = req.validate()?;
    let (bill, created) =
        vd_db::bills::save(&state.pool, user.clinic_id, visit_id, &amounts, user.id).await?;
    user.audit(
        &state,
        "bill",
        bill.id,
        if created { "create" } else { "update" },
    )
    .await?;
    Ok(Json(bill))
}

#[derive(Deserialize)]
pub struct Range {
    from: NaiveDate,
    to: NaiveDate,
}

#[derive(Serialize)]
pub struct Collection {
    from: NaiveDate,
    to: NaiveDate,
    totals: CollectionTotals,
    rows: Vec<CollectionRow>,
}

const MAX_RANGE_DAYS: i64 = 92;

/// Collection report for the doctor / admin (§5.8), by visit date.
pub async fn collection(
    State(state): State<AppState>,
    user: AuthUser,
    Query(r): Query<Range>,
) -> AppResult<Json<Collection>> {
    user.require(&[Role::Doctor, Role::Admin])?;
    if r.to < r.from || r.to - r.from > Duration::days(MAX_RANGE_DAYS) {
        return Err(AppError::Validation(format!(
            "date range: up to {MAX_RANGE_DAYS} days"
        )));
    }
    let rows = vd_db::bills::collection(&state.pool, user.clinic_id, r.from, r.to)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(Collection {
        from: r.from,
        to: r.to,
        totals: totals(&rows),
        rows,
    }))
}
