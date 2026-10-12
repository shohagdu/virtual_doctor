use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use vd_common::{AppError, AppResult};
use vd_domain::appointment::{Appointment, AppointmentStatus, Slot};
use vd_domain::clinic_today;
use vd_domain::text::opt;
use vd_domain::visit::{VisitStatus, VisitType};

use crate::auth::AuthUser;
use crate::routes::notify::{self, clinic_now};
use crate::state::AppState;

const MAX_RANGE_DAYS: i64 = 62;

fn parse_time(s: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(s.trim(), "%H:%M")
        .map_err(|_| AppError::Validation("time must be HH:MM".into()))
}

async fn load(state: &AppState, clinic_id: u64, id: u64) -> AppResult<Appointment> {
    vd_db::appointments::get(&state.pool, clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)
}

#[derive(Deserialize)]
pub struct Range {
    from: NaiveDate,
    to: NaiveDate,
}

/// Calendar and list view (§5.6): appointments between two clinic-local dates.
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Query(r): Query<Range>,
) -> AppResult<Json<Vec<Appointment>>> {
    if r.to < r.from || r.to - r.from > Duration::days(MAX_RANGE_DAYS) {
        return Err(AppError::Validation(format!(
            "date range: up to {MAX_RANGE_DAYS} days"
        )));
    }
    let items = vd_db::appointments::list(&state.pool, user.clinic_id, r.from, r.to)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(items))
}

/// A patient's appointments from today on (profile page).
pub async fn for_patient(
    State(state): State<AppState>,
    user: AuthUser,
    Path(patient_id): Path<u64>,
) -> AppResult<Json<Vec<Appointment>>> {
    let items =
        vd_db::appointments::for_patient(&state.pool, user.clinic_id, patient_id, clinic_today())
            .await
            .map_err(AppError::internal)?;
    Ok(Json(items))
}

#[derive(Deserialize)]
pub struct SlotsQuery {
    date: NaiveDate,
}

#[derive(Serialize)]
pub struct DaySlots {
    date: NaiveDate,
    /// No chamber schedule is set, so any time may be booked.
    free_time: bool,
    slots: Vec<Slot>,
    booked: usize,
    daily_limit: Option<u16>,
}

/// Bookable times on a day, from the chamber schedule (§5.6).
pub async fn slots(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<SlotsQuery>,
) -> AppResult<Json<DaySlots>> {
    let settings = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    let taken = vd_db::appointments::taken(&state.pool, user.clinic_id, q.date)
        .await
        .map_err(AppError::internal)?;
    let now = clinic_now();
    let slots = settings
        .slots(q.date)
        .into_iter()
        .map(|time| Slot {
            time,
            taken: taken.contains(&time),
            past: q.date < now.date() || (q.date == now.date() && time < now.time()),
        })
        .collect();
    Ok(Json(DaySlots {
        date: q.date,
        free_time: settings.schedule.is_empty(),
        slots,
        booked: taken.len(),
        daily_limit: settings.daily_limit,
    }))
}

#[derive(Deserialize)]
pub struct Book {
    patient_id: u64,
    date: NaiveDate,
    time: String,
    #[serde(default)]
    visit_type: VisitType,
    notes: Option<String>,
}

/// Books an appointment (PM-15) and queues the confirmation / reminder SMS.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<Book>,
) -> AppResult<(StatusCode, Json<Appointment>)> {
    let time = parse_time(&req.time)?;
    let settings = vd_db::settings::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    vd_db::appointments::check_time(&settings, req.date, time, clinic_now())?;
    let id = vd_db::appointments::book(
        &state.pool,
        user.clinic_id,
        &settings,
        &vd_db::appointments::Booking {
            patient_id: req.patient_id,
            date: req.date,
            time,
            visit_type: req.visit_type,
            notes: opt(req.notes, "notes", 255)?,
            source: "desk",
            created_by: user.id,
        },
    )
    .await?;
    user.audit(&state, "appointment", id, "create").await?;
    let a = load(&state, user.clinic_id, id).await?;
    notify::appointment_booked(&state, user.clinic_id, user.id, &a).await;
    Ok((StatusCode::CREATED, Json(a)))
}

#[derive(Deserialize)]
pub struct Update {
    status: Option<AppointmentStatus>,
    /// Reschedule: both date and time.
    date: Option<NaiveDate>,
    time: Option<String>,
}

/// Arrive (moves the patient to today's queue), no-show, cancel, restore, or
/// reschedule.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<Update>,
) -> AppResult<Json<Appointment>> {
    let pool = &state.pool;
    let clinic_id = user.clinic_id;
    let settings = vd_db::settings::get(pool, clinic_id)
        .await
        .map_err(AppError::internal)?;

    match (req.date, req.time.as_deref()) {
        (Some(date), Some(time)) => {
            let time = parse_time(time)?;
            vd_db::appointments::check_time(&settings, date, time, clinic_now())?;
            vd_db::appointments::reschedule(pool, clinic_id, &settings, id, date, time).await?;
            user.audit(&state, "appointment", id, "reschedule").await?;
            let a = load(&state, clinic_id, id).await?;
            notify::appointment_booked(&state, clinic_id, user.id, &a).await;
        }
        (None, None) => {}
        _ => {
            return Err(AppError::Validation(
                "give both date and time to reschedule".into(),
            ));
        }
    }

    match req.status {
        Some(AppointmentStatus::Arrived) => arrive(&state, &user, id).await?,
        Some(to) => {
            vd_db::appointments::set_status(pool, clinic_id, &settings, id, to).await?;
            user.audit(&state, "appointment", id, to.as_str()).await?;
            if to == AppointmentStatus::Booked {
                let a = load(&state, clinic_id, id).await?;
                notify::appointment_booked(&state, clinic_id, user.id, &a).await;
            } else {
                notify::appointment_withdrawn(&state, id).await;
            }
        }
        None => {}
    }
    Ok(Json(load(&state, clinic_id, id).await?))
}

/// Booked → Arrived: the patient joins today's queue (or keeps the place they
/// already have) and the appointment is linked to that visit.
async fn arrive(state: &AppState, user: &AuthUser, id: u64) -> AppResult<()> {
    let a = load(state, user.clinic_id, id).await?;
    if a.status != AppointmentStatus::Booked {
        return Err(AppError::Conflict(format!(
            "this appointment is {}; it cannot be marked arrived",
            a.status.as_str()
        )));
    }
    let today = clinic_today();
    if a.date != today {
        return Err(AppError::Conflict(
            "only today's appointments can be marked arrived".into(),
        ));
    }
    let open = vd_db::queue::open_visit(&state.pool, user.clinic_id, a.patient.id, today)
        .await
        .map_err(AppError::internal)?;
    let visit = match open {
        Some(v) => v,
        None => {
            let v = vd_db::visits::create(
                &state.pool,
                user.clinic_id,
                a.patient.id,
                a.visit_type,
                VisitStatus::Waiting,
                None,
                today,
            )
            .await
            .map_err(AppError::internal)?
            .ok_or(AppError::NotFound)?;
            user.audit(state, "visit", v.id, "create").await?;
            v
        }
    };
    if !vd_db::appointments::arrive(&state.pool, id, visit.id)
        .await
        .map_err(AppError::internal)?
    {
        return Err(AppError::Conflict(
            "this appointment was changed by someone else — reload".into(),
        ));
    }
    user.audit(state, "appointment", id, "arrived").await?;
    notify::appointment_withdrawn(state, id).await;
    Ok(())
}
