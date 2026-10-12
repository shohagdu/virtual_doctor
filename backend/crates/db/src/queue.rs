//! Today's queue (Patient_Management_Spec §5.1, PM-14).

use chrono::{NaiveDate, NaiveDateTime};
use sqlx::{MySql, Transaction};
use vd_common::AppError;
use vd_domain::patient::Sex;
use vd_domain::visit::{
    QueueCounters, QueueEntry, QueuePatient, Visit, VisitStatus, VisitType, reorder,
};

use crate::{DbResult, MySqlPool, visits};

fn db(e: sqlx::Error) -> AppError {
    AppError::internal(e)
}

#[derive(sqlx::FromRow)]
struct Row {
    id: u64,
    patient_id: u64,
    visit_no: u32,
    visit_date: NaiveDate,
    visit_type: String,
    status: String,
    queue_serial: Option<u32>,
    doctor_id: Option<u64>,
    created_at: NaiveDateTime,
    #[sqlx(flatten)]
    patient: PatientRow,
    has_allergy: bool,
    has_vitals: bool,
    bill_status: Option<String>,
}

/// Patient columns shown in lists (queue, appointments, follow-ups); select with
/// `PATIENT_COLUMNS` on a `patients p` join.
pub(crate) const PATIENT_COLUMNS: &str =
    "p.id AS patient_id, p.patient_code, p.title, p.name_en, p.name_bn, p.phone, p.dob, p.sex";

#[derive(sqlx::FromRow)]
pub(crate) struct PatientRow {
    patient_id: u64,
    patient_code: String,
    title: Option<String>,
    name_en: String,
    name_bn: Option<String>,
    phone: String,
    dob: NaiveDate,
    sex: String,
}

impl From<PatientRow> for QueuePatient {
    fn from(r: PatientRow) -> Self {
        Self {
            id: r.patient_id,
            patient_code: r.patient_code,
            title: r.title,
            name_en: r.name_en,
            name_bn: r.name_bn,
            phone: r.phone,
            dob: r.dob,
            sex: Sex::parse(&r.sex).unwrap_or(Sex::O),
        }
    }
}

/// All of today's visits in call order: active ones by queue position, then seen and
/// cancelled ones.
pub async fn today(pool: &MySqlPool, clinic_id: u64, day: NaiveDate) -> DbResult<Vec<QueueEntry>> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT v.id, v.patient_id, v.visit_no, v.visit_date, v.visit_type, v.status,
                v.queue_serial, v.doctor_id, v.created_at, {PATIENT_COLUMNS},
                COALESCE(JSON_LENGTH(m.allergies), 0) > 0 AS has_allergy,
                EXISTS (SELECT 1 FROM vitals vt WHERE vt.visit_id = v.id) AS has_vitals,
                b.status AS bill_status
         FROM visits v
         JOIN patients p ON p.id = v.patient_id
         LEFT JOIN patient_medical_profile m ON m.patient_id = v.patient_id
         LEFT JOIN bills b ON b.visit_id = v.id
         WHERE v.clinic_id = ? AND v.visit_date = ? AND v.deleted_at IS NULL
         ORDER BY FIELD(v.status, 'in_consultation', 'waiting', 'seen', 'cancelled'),
                  v.queue_position, v.id"
    )))
    .bind(clinic_id)
    .bind(day)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| QueueEntry {
            visit: Visit {
                id: r.id,
                patient_id: r.patient_id,
                visit_no: r.visit_no,
                visit_date: r.visit_date,
                visit_type: VisitType::parse(&r.visit_type).unwrap_or_default(),
                status: VisitStatus::parse(&r.status).unwrap_or(VisitStatus::Waiting),
                queue_serial: r.queue_serial,
                doctor_id: r.doctor_id,
                created_at: r.created_at.and_utc(),
            },
            patient: r.patient.into(),
            has_allergy: r.has_allergy,
            has_vitals: r.has_vitals,
            bill_status: r.bill_status,
        })
        .collect())
}

pub async fn counters(pool: &MySqlPool, clinic_id: u64, day: NaiveDate) -> DbResult<QueueCounters> {
    let (today, waiting, in_consultation, seen): (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT CAST(COUNT(*) AS SIGNED),
                CAST(COALESCE(SUM(status = 'waiting'), 0) AS SIGNED),
                CAST(COALESCE(SUM(status = 'in_consultation'), 0) AS SIGNED),
                CAST(COALESCE(SUM(status = 'seen'), 0) AS SIGNED)
         FROM visits
         WHERE clinic_id = ? AND visit_date = ? AND deleted_at IS NULL AND status <> 'cancelled'",
    )
    .bind(clinic_id)
    .bind(day)
    .fetch_one(pool)
    .await?;
    let followups_due: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM followups WHERE clinic_id = ? AND status = 'pending' AND due_date <= ?",
    )
    .bind(clinic_id)
    .bind(day)
    .fetch_one(pool)
    .await?;
    Ok(QueueCounters {
        today,
        waiting,
        in_consultation,
        seen,
        followups_due,
    })
}

/// The patient's waiting or in-consultation visit today, if any. Used so a patient is
/// never queued twice on the same day.
pub async fn open_visit(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    day: NaiveDate,
) -> DbResult<Option<Visit>> {
    let id: Option<u64> = sqlx::query_scalar(
        "SELECT id FROM visits
         WHERE clinic_id = ? AND patient_id = ? AND visit_date = ? AND deleted_at IS NULL
           AND status IN ('waiting', 'in_consultation')
         ORDER BY id DESC LIMIT 1",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .bind(day)
    .fetch_optional(pool)
    .await?;
    match id {
        Some(id) => visits::get(pool, clinic_id, id).await,
        None => Ok(None),
    }
}

async fn status_for_update(
    tx: &mut Transaction<'_, MySql>,
    clinic_id: u64,
    id: u64,
) -> Result<VisitStatus, AppError> {
    let status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM visits WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db)?;
    status
        .and_then(|s| VisitStatus::parse(&s))
        .ok_or(AppError::NotFound)
}

async fn write_status(
    tx: &mut Transaction<'_, MySql>,
    id: u64,
    to: VisitStatus,
    doctor_id: Option<u64>,
) -> Result<(), AppError> {
    // The doctor who starts the consultation is recorded on the visit.
    sqlx::query("UPDATE visits SET status = ?, doctor_id = COALESCE(?, doctor_id) WHERE id = ?")
        .bind(to.as_str())
        .bind(doctor_id)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    Ok(())
}

/// Changes a visit's queue status, enforcing the allowed transitions.
pub async fn set_status(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    to: VisitStatus,
    doctor_id: Option<u64>,
) -> Result<Visit, AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    let from = status_for_update(&mut tx, clinic_id, id).await?;
    if !from.can_become(to) {
        return Err(AppError::Conflict(format!(
            "a {} visit cannot be marked {}",
            from.as_str(),
            to.as_str()
        )));
    }
    let doctor_id = if to == VisitStatus::InConsultation {
        doctor_id
    } else {
        None
    };
    write_status(&mut tx, id, to, doctor_id).await?;
    tx.commit().await.map_err(db)?;
    visits::get(pool, clinic_id, id)
        .await
        .map_err(db)?
        .ok_or(AppError::NotFound)
}

/// Moves a visit to `position` (0-based) in the day's call order.
pub async fn move_to(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    position: usize,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    let day: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT visit_date FROM visits WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?;
    let day = day.ok_or(AppError::NotFound)?;
    // Only the waiting part of the queue is reordered; who is next is all that changes.
    let order: Vec<u64> = sqlx::query_scalar(
        "SELECT id FROM visits
         WHERE clinic_id = ? AND visit_date = ? AND deleted_at IS NULL AND status = 'waiting'
         ORDER BY queue_position, id FOR UPDATE",
    )
    .bind(clinic_id)
    .bind(day)
    .fetch_all(&mut *tx)
    .await
    .map_err(db)?;
    let order = reorder(order, id, position)
        .ok_or_else(|| AppError::Conflict("only waiting patients can be reordered".into()))?;
    let base: u64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(MIN(queue_position), 1) AS UNSIGNED) FROM visits
         WHERE clinic_id = ? AND visit_date = ? AND status = 'waiting' AND deleted_at IS NULL",
    )
    .bind(clinic_id)
    .bind(day)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    for (i, vid) in order.iter().enumerate() {
        sqlx::query("UPDATE visits SET queue_position = ? WHERE id = ?")
            .bind(base + i as u64)
            .bind(vid)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
    }
    tx.commit().await.map_err(db)?;
    Ok(())
}

/// "Save & Next Patient" (§5.5): marks `current` as seen and calls the first waiting
/// patient of the day into consultation. Returns the called visit, or `None` if nobody
/// is waiting.
pub async fn call_next(
    pool: &MySqlPool,
    clinic_id: u64,
    day: NaiveDate,
    current: Option<u64>,
    doctor_id: u64,
) -> Result<Option<Visit>, AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    if let Some(cur) = current {
        let from = status_for_update(&mut tx, clinic_id, cur).await?;
        if from.can_become(VisitStatus::Seen) {
            write_status(&mut tx, cur, VisitStatus::Seen, None).await?;
        }
    }
    let next: Option<u64> = sqlx::query_scalar(
        "SELECT id FROM visits
         WHERE clinic_id = ? AND visit_date = ? AND deleted_at IS NULL AND status = 'waiting'
         ORDER BY queue_position, id LIMIT 1 FOR UPDATE",
    )
    .bind(clinic_id)
    .bind(day)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?;
    if let Some(id) = next {
        write_status(&mut tx, id, VisitStatus::InConsultation, Some(doctor_id)).await?;
    }
    tx.commit().await.map_err(db)?;
    match next {
        Some(id) => visits::get(pool, clinic_id, id).await.map_err(db),
        None => Ok(None),
    }
}
