use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime};
use sqlx::types::Json;
use vd_domain::prescription::MedicineLine;
use vd_domain::visit::{PrescriptionSummary, Visit, VisitStatus, VisitSummary, VisitType};

use crate::prescriptions::{MED_COLUMNS, MedRow};
use crate::{DbResult, MySqlPool};

const COLUMNS: &str =
    "id, patient_id, visit_no, visit_date, visit_type, status, queue_serial, doctor_id, created_at";

#[derive(sqlx::FromRow)]
struct VisitRow {
    id: u64,
    patient_id: u64,
    visit_no: u32,
    visit_date: NaiveDate,
    visit_type: String,
    status: String,
    queue_serial: Option<u32>,
    doctor_id: Option<u64>,
    created_at: NaiveDateTime,
}

impl From<VisitRow> for Visit {
    fn from(r: VisitRow) -> Self {
        Self {
            id: r.id,
            patient_id: r.patient_id,
            visit_no: r.visit_no,
            visit_date: r.visit_date,
            visit_type: VisitType::parse(&r.visit_type).unwrap_or_default(),
            status: VisitStatus::parse(&r.status).unwrap_or(VisitStatus::Waiting),
            queue_serial: r.queue_serial,
            doctor_id: r.doctor_id,
            created_at: r.created_at.and_utc(),
        }
    }
}

pub async fn get(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<Option<Visit>> {
    let row: Option<VisitRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM visits WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL"
    )))
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

/// Starts a visit: next visit number for the patient and next queue serial for the day
/// (PM-06). Returns `None` if the patient is not in this clinic.
pub async fn create(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    visit_type: VisitType,
    status: VisitStatus,
    doctor_id: Option<u64>,
    today: NaiveDate,
) -> DbResult<Option<Visit>> {
    let mut tx = pool.begin().await?;

    // Locking the patient row serialises visit numbering for that patient.
    let exists: Option<u64> = sqlx::query_scalar(
        "SELECT id FROM patients WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_optional(&mut *tx)
    .await?;
    if exists.is_none() {
        return Ok(None);
    }

    let visit_no: u64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(MAX(visit_no), 0) + 1 AS UNSIGNED) FROM visits WHERE patient_id = ?",
    )
    .bind(patient_id)
    .fetch_one(&mut *tx)
    .await?;
    let (serial, position): (u64, u64) = sqlx::query_as(
        "SELECT CAST(COALESCE(MAX(queue_serial), 0) + 1 AS UNSIGNED),
                CAST(COALESCE(MAX(queue_position), 0) + 1 AS UNSIGNED)
         FROM visits WHERE clinic_id = ? AND visit_date = ?",
    )
    .bind(clinic_id)
    .bind(today)
    .fetch_one(&mut *tx)
    .await?;

    let res = sqlx::query(
        "INSERT INTO visits (clinic_id, patient_id, visit_no, visit_date, visit_type, queue_serial,
            queue_position, status, doctor_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .bind(visit_no)
    .bind(today)
    .bind(visit_type.as_str())
    .bind(serial)
    .bind(position)
    .bind(status.as_str())
    .bind(doctor_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    get(pool, clinic_id, res.last_insert_id()).await
}

/// Date of the patient's most recent visit before `visit_id`.
pub async fn previous_visit_date(
    pool: &MySqlPool,
    patient_id: u64,
    visit_id: u64,
) -> DbResult<Option<NaiveDate>> {
    sqlx::query_scalar(
        "SELECT MAX(visit_date) FROM visits
         WHERE patient_id = ? AND id < ? AND deleted_at IS NULL AND status <> 'cancelled'",
    )
    .bind(patient_id)
    .bind(visit_id)
    .fetch_one(pool)
    .await
}

#[derive(sqlx::FromRow)]
struct LatestRx {
    id: u64,
    visit_id: u64,
    version: u32,
    locked: bool,
    complaints: Option<Json<Vec<String>>>,
    diagnosis: Option<Json<Vec<String>>>,
    next_visit_date: Option<NaiveDate>,
}

// Latest prescription version of each visit of one patient.
const LATEST_RX: &str = "SELECT p.id FROM prescriptions p
     JOIN visits v ON v.id = p.visit_id
     WHERE v.patient_id = ? AND p.version = (
        SELECT MAX(p2.version) FROM prescriptions p2 WHERE p2.visit_id = p.visit_id)";

/// The patient's visit timeline, newest first. With `clinical = false` (limited history
/// access, §4) prescription details are left out.
pub async fn timeline(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    clinical: bool,
) -> DbResult<Vec<VisitSummary>> {
    let rows: Vec<VisitRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM visits
         WHERE clinic_id = ? AND patient_id = ? AND deleted_at IS NULL
         ORDER BY visit_date DESC, id DESC"
    )))
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_all(pool)
    .await?;

    let mut rx_by_visit: HashMap<u64, PrescriptionSummary> = HashMap::new();
    if clinical && !rows.is_empty() {
        let latest: Vec<LatestRx> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT id, visit_id, version, locked, complaints, diagnosis, next_visit_date
             FROM prescriptions WHERE id IN ({LATEST_RX})"
        )))
        .bind(patient_id)
        .fetch_all(pool)
        .await?;

        let meds: Vec<MedRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {MED_COLUMNS} FROM prescription_medicines
             WHERE prescription_id IN ({LATEST_RX}) ORDER BY prescription_id, sort_order, id"
        )))
        .bind(patient_id)
        .fetch_all(pool)
        .await?;
        let mut meds_by_rx: HashMap<u64, Vec<MedicineLine>> = HashMap::new();
        for m in meds {
            meds_by_rx
                .entry(m.prescription_id)
                .or_default()
                .push(m.into());
        }

        for r in latest {
            rx_by_visit.insert(
                r.visit_id,
                PrescriptionSummary {
                    version: r.version,
                    locked: r.locked,
                    complaints: r.complaints.map(|j| j.0).unwrap_or_default(),
                    diagnosis: r.diagnosis.map(|j| j.0).unwrap_or_default(),
                    medicines: meds_by_rx.remove(&r.id).unwrap_or_default(),
                    next_visit_date: r.next_visit_date,
                },
            );
        }
    }

    Ok(rows
        .into_iter()
        .map(|r| {
            let prescription = rx_by_visit.remove(&r.id);
            VisitSummary {
                visit: r.into(),
                prescription,
            }
        })
        .collect())
}
