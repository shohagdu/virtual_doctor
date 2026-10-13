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

// ---- Medicines page: manage the catalogue (doctor, admin) ----

use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use vd_domain::medicine::{FORMS, Medicine, MedicineInput, to_csv};

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    q: String,
    form: Option<String>,
    company: Option<String>,
    /// "active", "inactive" or empty for both.
    #[serde(default)]
    status: String,
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(serde::Serialize)]
pub struct MedicinePage {
    items: Vec<Medicine>,
    total: i64,
    page: u32,
    per_page: u32,
    /// Forms in use plus the common ones, for the form picker and filter.
    forms: Vec<String>,
    /// Companies from Settings plus any already used, for the company picker and filter.
    companies: Vec<String>,
}

const MANAGERS: &[Role] = &[Role::Doctor, Role::Admin];

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<MedicinePage>> {
    user.require(MANAGERS)?;
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(25).clamp(5, 200);
    let form = q.form.as_deref().map(str::trim).filter(|f| !f.is_empty());
    let company = q
        .company
        .as_deref()
        .map(str::trim)
        .filter(|f| !f.is_empty());
    let active = match q.status.as_str() {
        "active" => Some(true),
        "inactive" => Some(false),
        _ => None,
    };
    let (items, total) = vd_db::medicines::list(
        &state.pool,
        user.clinic_id,
        &vd_db::medicines::ListFilter {
            q: &q.q,
            form,
            company,
            active,
            page,
            per_page,
        },
    )
    .await
    .map_err(AppError::internal)?;
    let mut forms: Vec<String> = FORMS.iter().map(|f| f.to_string()).collect();
    for f in vd_db::medicines::forms(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?
    {
        if !forms.iter().any(|x| x.eq_ignore_ascii_case(&f)) {
            forms.push(f);
        }
    }
    let mut companies = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?
        .rx_lists
        .companies;
    for c in vd_db::medicines::companies(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?
    {
        if !companies.iter().any(|x| x.eq_ignore_ascii_case(&c)) {
            companies.push(c);
        }
    }
    Ok(Json(MedicinePage {
        items,
        total,
        page,
        per_page,
        forms,
        companies,
    }))
}

async fn one(state: &AppState, user: &AuthUser, id: u64) -> AppResult<Json<Medicine>> {
    let m = vd_db::medicines::get(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(m))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<MedicineInput>,
) -> AppResult<(StatusCode, Json<Medicine>)> {
    user.require(MANAGERS)?;
    let m = req.validate()?;
    let id = vd_db::medicines::create(&state.pool, user.clinic_id, &m).await?;
    user.audit(&state, "medicine", id, "create").await?;
    Ok((StatusCode::CREATED, one(&state, &user, id).await?))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<MedicineInput>,
) -> AppResult<Json<Medicine>> {
    user.require(MANAGERS)?;
    let m = req.validate()?;
    vd_db::medicines::update(&state.pool, user.clinic_id, id, &m).await?;
    user.audit(&state, "medicine", id, "update").await?;
    one(&state, &user, id).await
}

#[derive(Deserialize)]
pub struct Activate {
    is_active: bool,
}

pub async fn set_active(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<Activate>,
) -> AppResult<Json<Medicine>> {
    user.require(MANAGERS)?;
    vd_db::medicines::set_active(&state.pool, user.clinic_id, id, req.is_active).await?;
    user.audit(
        &state,
        "medicine",
        id,
        if req.is_active {
            "activate"
        } else {
            "deactivate"
        },
    )
    .await?;
    one(&state, &user, id).await
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<StatusCode> {
    user.require(MANAGERS)?;
    vd_db::medicines::delete(&state.pool, user.clinic_id, id).await?;
    user.audit(&state, "medicine", id, "delete").await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The whole catalogue as CSV, in the import format.
pub async fn export(State(state): State<AppState>, user: AuthUser) -> AppResult<impl IntoResponse> {
    user.require(MANAGERS)?;
    let rows = vd_db::medicines::all(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"medicines.csv\"",
            ),
        ],
        format!("\u{feff}{}", to_csv(&rows)),
    ))
}
