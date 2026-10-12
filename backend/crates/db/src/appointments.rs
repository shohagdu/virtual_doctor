//! Appointments (Patient_Management_Spec §5.6, PM-15).

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{MySql, Transaction};
use vd_common::AppError;
use vd_domain::appointment::{Appointment, AppointmentStatus};
use vd_domain::settings::ClinicSettings;
use vd_domain::visit::VisitType;
use vd_domain::{clinic_to_utc, utc_to_clinic};

use crate::queue::{PATIENT_COLUMNS, PatientRow};
use crate::{DbResult, MySqlPool};

fn db(e: sqlx::Error) -> AppError {
    AppError::internal(e)
}

#[derive(sqlx::FromRow)]
struct Row {
    id: u64,
    start_at: NaiveDateTime,
    visit_type: String,
    status: String,
    notes: Option<String>,
    visit_id: Option<u64>,
    sms_sent_at: Option<NaiveDateTime>,
    #[sqlx(flatten)]
    patient: PatientRow,
}

impl From<Row> for Appointment {
    fn from(r: Row) -> Self {
        let start_at = r.start_at.and_utc();
        let local = utc_to_clinic(start_at);
        Self {
            id: r.id,
            start_at,
            date: local.date(),
            time: local.time(),
            visit_type: VisitType::parse(&r.visit_type).unwrap_or_default(),
            status: AppointmentStatus::parse(&r.status).unwrap_or(AppointmentStatus::Booked),
            notes: r.notes,
            visit_id: r.visit_id,
            sms_sent_at: r.sms_sent_at.map(|t| t.and_utc()),
            patient: r.patient.into(),
        }
    }
}

/// An arrived appointment shows as seen once its visit is seen, so the status never
/// has to be copied between the two tables.
fn select(filter: &str) -> String {
    format!(
        "SELECT a.id, a.start_at, a.visit_type,
                CASE WHEN a.status = 'arrived' AND v.status = 'seen' THEN 'seen' ELSE a.status END AS status,
                a.notes, a.visit_id, a.sms_sent_at, {PATIENT_COLUMNS}
         FROM appointments a
         JOIN patients p ON p.id = a.patient_id
         LEFT JOIN visits v ON v.id = a.visit_id
         WHERE a.clinic_id = ? AND {filter}
         ORDER BY a.start_at, a.id"
    )
}

/// Appointments from `from` to `to` (clinic-local dates, inclusive).
pub async fn list(
    pool: &MySqlPool,
    clinic_id: u64,
    from: NaiveDate,
    to: NaiveDate,
) -> DbResult<Vec<Appointment>> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(select(
        "a.start_at >= ? AND a.start_at < ?",
    )))
    .bind(clinic_id)
    .bind(clinic_to_utc(from, NaiveTime::MIN).naive_utc())
    .bind(clinic_to_utc(to + Duration::days(1), NaiveTime::MIN).naive_utc())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<Option<Appointment>> {
    let row: Option<Row> = sqlx::query_as(sqlx::AssertSqlSafe(select("a.id = ?")))
        .bind(clinic_id)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(Into::into))
}

/// Upcoming and recent appointments of one patient (profile page).
pub async fn for_patient(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    from: NaiveDate,
) -> DbResult<Vec<Appointment>> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(select(
        "a.patient_id = ? AND a.start_at >= ?",
    )))
    .bind(clinic_id)
    .bind(patient_id)
    .bind(clinic_to_utc(from, NaiveTime::MIN).naive_utc())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// Start times on `date` already held by an active appointment.
pub async fn taken(pool: &MySqlPool, clinic_id: u64, date: NaiveDate) -> DbResult<Vec<NaiveTime>> {
    let starts: Vec<NaiveDateTime> = sqlx::query_scalar(
        "SELECT start_at FROM appointments
         WHERE clinic_id = ? AND start_at >= ? AND start_at < ? AND status IN ('booked', 'arrived')",
    )
    .bind(clinic_id)
    .bind(clinic_to_utc(date, NaiveTime::MIN).naive_utc())
    .bind(clinic_to_utc(date + Duration::days(1), NaiveTime::MIN).naive_utc())
    .fetch_all(pool)
    .await?;
    Ok(starts
        .into_iter()
        .map(|t| utc_to_clinic(t.and_utc()).time())
        .collect())
}

const MAX_DAYS_AHEAD: i64 = 180;

/// Checks a requested time against the calendar and the chamber schedule.
pub fn check_time(
    settings: &ClinicSettings,
    date: NaiveDate,
    time: NaiveTime,
    now: NaiveDateTime,
) -> Result<(), AppError> {
    let bad = |m: String| Err(AppError::Validation(m));
    if date < now.date() || (date == now.date() && time < now.time()) {
        return bad("that time has already passed".into());
    }
    if date > now.date() + Duration::days(MAX_DAYS_AHEAD) {
        return bad(format!(
            "appointments can be booked up to {MAX_DAYS_AHEAD} days ahead"
        ));
    }
    if !settings.schedule.is_empty() && !settings.slots(date).contains(&time) {
        return bad(format!(
            "{} is not a slot in the chamber schedule for {}",
            time.format("%H:%M"),
            date.format("%d/%m/%Y")
        ));
    }
    Ok(())
}

/// Within a transaction that holds the clinic lock: the slot must be free, the day
/// under its limit, and the patient not already booked that day.
async fn check_free(
    tx: &mut Transaction<'_, MySql>,
    clinic_id: u64,
    settings: &ClinicSettings,
    patient_id: u64,
    date: NaiveDate,
    time: NaiveTime,
    except: Option<u64>,
) -> Result<(), AppError> {
    let day_start = clinic_to_utc(date, NaiveTime::MIN).naive_utc();
    let day_end = clinic_to_utc(date + Duration::days(1), NaiveTime::MIN).naive_utc();
    let start = clinic_to_utc(date, time).naive_utc();
    let (in_slot, on_day, mine): (i64, i64, i64) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(start_at = ?), 0) AS SIGNED),
                CAST(COUNT(*) AS SIGNED),
                CAST(COALESCE(SUM(patient_id = ?), 0) AS SIGNED)
         FROM appointments
         WHERE clinic_id = ? AND start_at >= ? AND start_at < ?
           AND status IN ('booked', 'arrived') AND id <> ?",
    )
    .bind(start)
    .bind(patient_id)
    .bind(clinic_id)
    .bind(day_start)
    .bind(day_end)
    .bind(except.unwrap_or(0))
    .fetch_one(&mut **tx)
    .await
    .map_err(db)?;
    if mine > 0 {
        return Err(AppError::Conflict(
            "this patient already has an appointment that day".into(),
        ));
    }
    if in_slot > 0 {
        return Err(AppError::Conflict(format!(
            "{} is already booked",
            time.format("%H:%M")
        )));
    }
    if let Some(limit) = settings.daily_limit
        && on_day >= limit as i64
    {
        return Err(AppError::Conflict(format!(
            "the daily limit of {limit} appointments is reached"
        )));
    }
    Ok(())
}

/// Serialises bookings for one clinic, so two people cannot take the same slot.
async fn lock_clinic(tx: &mut Transaction<'_, MySql>, clinic_id: u64) -> Result<(), AppError> {
    sqlx::query("SELECT id FROM clinics WHERE id = ? FOR UPDATE")
        .bind(clinic_id)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    Ok(())
}

pub struct Booking {
    pub patient_id: u64,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub visit_type: VisitType,
    pub notes: Option<String>,
    pub source: &'static str,
    pub created_by: u64,
}

/// Books an appointment. The time must already have passed `check_time`.
pub async fn book(
    pool: &MySqlPool,
    clinic_id: u64,
    settings: &ClinicSettings,
    b: &Booking,
) -> Result<u64, AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    lock_clinic(&mut tx, clinic_id).await?;
    let patient: Option<u64> = sqlx::query_scalar(
        "SELECT id FROM patients WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(clinic_id)
    .bind(b.patient_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?;
    if patient.is_none() {
        return Err(AppError::NotFound);
    }
    check_free(
        &mut tx,
        clinic_id,
        settings,
        b.patient_id,
        b.date,
        b.time,
        None,
    )
    .await?;
    let id = sqlx::query(
        "INSERT INTO appointments (clinic_id, patient_id, start_at, visit_type, source, notes, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(b.patient_id)
    .bind(clinic_to_utc(b.date, b.time).naive_utc())
    .bind(b.visit_type.as_str())
    .bind(b.source)
    .bind(&b.notes)
    .bind(b.created_by)
    .execute(&mut *tx)
    .await
    .map_err(db)?
    .last_insert_id();
    tx.commit().await.map_err(db)?;
    Ok(id)
}

/// Current (derived) status and patient of an appointment, locked for update.
async fn lock_appointment(
    tx: &mut Transaction<'_, MySql>,
    clinic_id: u64,
    id: u64,
) -> Result<(AppointmentStatus, u64, NaiveDateTime), AppError> {
    let row: Option<(String, u64, NaiveDateTime)> = sqlx::query_as(
        "SELECT CASE WHEN a.status = 'arrived' AND v.status = 'seen' THEN 'seen' ELSE a.status END,
                a.patient_id, a.start_at
         FROM appointments a LEFT JOIN visits v ON v.id = a.visit_id
         WHERE a.clinic_id = ? AND a.id = ? FOR UPDATE",
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db)?;
    let (status, patient_id, start) = row.ok_or(AppError::NotFound)?;
    let status = AppointmentStatus::parse(&status).ok_or(AppError::NotFound)?;
    Ok((status, patient_id, start))
}

/// Moves a booked appointment to another time (PM-15 reschedule).
pub async fn reschedule(
    pool: &MySqlPool,
    clinic_id: u64,
    settings: &ClinicSettings,
    id: u64,
    date: NaiveDate,
    time: NaiveTime,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    lock_clinic(&mut tx, clinic_id).await?;
    let (status, patient_id, _) = lock_appointment(&mut tx, clinic_id, id).await?;
    if status != AppointmentStatus::Booked {
        return Err(AppError::Conflict(
            "only a booked appointment can be moved".into(),
        ));
    }
    check_free(
        &mut tx,
        clinic_id,
        settings,
        patient_id,
        date,
        time,
        Some(id),
    )
    .await?;
    sqlx::query("UPDATE appointments SET start_at = ?, sms_sent_at = NULL WHERE id = ?")
        .bind(clinic_to_utc(date, time).naive_utc())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)
}

/// Changes the status (no-show, cancel, restore). Arrival goes through `arrive`.
/// A restored booking must still find its slot free.
pub async fn set_status(
    pool: &MySqlPool,
    clinic_id: u64,
    settings: &ClinicSettings,
    id: u64,
    to: AppointmentStatus,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    lock_clinic(&mut tx, clinic_id).await?;
    let (from, patient_id, start) = lock_appointment(&mut tx, clinic_id, id).await?;
    if !from.can_become(to) || to == AppointmentStatus::Arrived {
        return Err(AppError::Conflict(format!(
            "this appointment is {}; it cannot be marked {}",
            from.as_str(),
            to.as_str()
        )));
    }
    if to == AppointmentStatus::Booked {
        let local = utc_to_clinic(start.and_utc());
        check_free(
            &mut tx,
            clinic_id,
            settings,
            patient_id,
            local.date(),
            local.time(),
            Some(id),
        )
        .await?;
    }
    sqlx::query("UPDATE appointments SET status = ? WHERE id = ?")
        .bind(to.as_str())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)
}

/// Marks a booked appointment arrived and links it to the patient's visit in
/// today's queue. False if it was no longer booked.
pub async fn arrive(pool: &MySqlPool, id: u64, visit_id: u64) -> DbResult<bool> {
    let res = sqlx::query(
        "UPDATE appointments SET status = 'arrived', visit_id = ? WHERE id = ? AND status = 'booked'",
    )
    .bind(visit_id)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}
