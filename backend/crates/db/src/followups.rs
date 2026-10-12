//! Follow-ups due (Patient_Management_Spec §5.7, PM-16). Records are created from
//! the prescription's Next Visit date (see `prescriptions::save`).

use chrono::{NaiveDate, NaiveDateTime};
use sqlx::types::Json;
use vd_domain::followup::{Bucket, BucketCounts, Followup, FollowupStatus, SmsState};
use vd_domain::visit::QueuePatient;

use crate::queue::{PATIENT_COLUMNS, PatientRow};
use crate::{DbResult, MySqlPool};

#[derive(sqlx::FromRow)]
struct Row {
    id: u64,
    due_date: NaiveDate,
    status: String,
    reminder_sent_at: Option<NaiveDateTime>,
    visit_id: Option<u64>,
    visit_date: Option<NaiveDate>,
    diagnosis: Option<Json<Vec<String>>>,
    chronic: Option<Json<Vec<String>>>,
    sms_id: Option<u64>,
    sms_status: Option<String>,
    sms_error: Option<String>,
    sms_at: Option<NaiveDateTime>,
    #[sqlx(flatten)]
    patient: PatientRow,
}

const MAX_ROWS: u32 = 500;
/// Lower bound for "overdue" (NaiveDate::MIN is outside MySQL's DATE range).
const EARLIEST: NaiveDate = NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid date");

/// One tab of the follow-up list. `clinical` adds the diagnosis (doctor only, §4).
pub async fn list(
    pool: &MySqlPool,
    clinic_id: u64,
    bucket: Bucket,
    today: NaiveDate,
    clinical: bool,
) -> DbResult<Vec<Followup>> {
    let f = bucket.filter(today);
    let order = if f.pending {
        "f.due_date, f.id"
    } else {
        "f.due_date DESC, f.id DESC"
    };
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT f.id, f.due_date, f.status, f.reminder_sent_at, rx.visit_id, v.visit_date,
                rx.diagnosis, m.chronic_conditions AS chronic,
                s.id AS sms_id, s.status AS sms_status, s.error AS sms_error, s.created_at AS sms_at,
                {PATIENT_COLUMNS}
         FROM followups f
         JOIN patients p ON p.id = f.patient_id AND p.deleted_at IS NULL
         LEFT JOIN prescriptions rx ON rx.id = f.prescription_id
         LEFT JOIN visits v ON v.id = rx.visit_id
         LEFT JOIN patient_medical_profile m ON m.patient_id = f.patient_id
         LEFT JOIN sms_messages s ON s.id = (
             SELECT MAX(id) FROM sms_messages
             WHERE kind = 'followup_reminder' AND ref_id = f.id AND status <> 'cancelled')
         WHERE f.clinic_id = ? AND f.due_date <= ? AND f.due_date >= ?
           AND (f.status = 'pending') = ?
         ORDER BY {order}
         LIMIT {MAX_ROWS}"
    )))
    .bind(clinic_id)
    .bind(f.to)
    .bind(f.from.unwrap_or(EARLIEST))
    .bind(f.pending)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Followup {
            id: r.id,
            due_date: r.due_date,
            status: FollowupStatus::parse(&r.status).unwrap_or(FollowupStatus::Pending),
            reminder_sent_at: r.reminder_sent_at.map(|t| t.and_utc()),
            visit_id: r.visit_id,
            visit_date: r.visit_date,
            diagnosis: clinical.then(|| r.diagnosis.map(|j| j.0).unwrap_or_default()),
            chronic_conditions: r.chronic.map(|j| j.0).unwrap_or_default(),
            patient: r.patient.into(),
            last_sms: match (r.sms_id, r.sms_status, r.sms_at) {
                (Some(id), Some(status), Some(at)) => Some(SmsState {
                    id,
                    status,
                    error: r.sms_error,
                    created_at: at.and_utc(),
                }),
                _ => None,
            },
        })
        .collect())
}

pub async fn counts(pool: &MySqlPool, clinic_id: u64, today: NaiveDate) -> DbResult<BucketCounts> {
    let week_end = Bucket::Week.filter(today).to;
    let (today_n, overdue, week): (i64, i64, i64) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(due_date = ?), 0) AS SIGNED),
                CAST(COALESCE(SUM(due_date < ?), 0) AS SIGNED),
                CAST(COALESCE(SUM(due_date >= ? AND due_date <= ?), 0) AS SIGNED)
         FROM followups f
         JOIN patients p ON p.id = f.patient_id AND p.deleted_at IS NULL
         WHERE f.clinic_id = ? AND f.status = 'pending' AND f.due_date <= ?",
    )
    .bind(today)
    .bind(today)
    .bind(today)
    .bind(week_end)
    .bind(clinic_id)
    .bind(week_end)
    .fetch_one(pool)
    .await?;
    Ok(BucketCounts {
        today: today_n,
        overdue,
        week,
    })
}

/// Marks a follow-up came / missed (or back to pending). False if not found.
pub async fn set_status(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    status: FollowupStatus,
) -> DbResult<bool> {
    let res = sqlx::query("UPDATE followups SET status = ? WHERE clinic_id = ? AND id = ?")
        .bind(status.as_str())
        .bind(clinic_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() == 1)
}

/// The patient and due date of one follow-up, for its reminder SMS.
pub async fn for_reminder(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
) -> DbResult<Option<(QueuePatient, NaiveDate)>> {
    #[derive(sqlx::FromRow)]
    struct ReminderRow {
        due_date: NaiveDate,
        #[sqlx(flatten)]
        patient: PatientRow,
    }
    let row: Option<ReminderRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT f.due_date, {PATIENT_COLUMNS}
         FROM followups f JOIN patients p ON p.id = f.patient_id AND p.deleted_at IS NULL
         WHERE f.clinic_id = ? AND f.id = ?"
    )))
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.patient.into(), r.due_date)))
}

/// A patient who comes in has kept any follow-up due within the next week or
/// already overdue (§5.7 "mark as Came", done automatically when they arrive).
pub async fn mark_came_on_visit(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    today: NaiveDate,
) -> DbResult<()> {
    sqlx::query(
        "UPDATE followups SET status = 'came'
         WHERE clinic_id = ? AND patient_id = ? AND status = 'pending'
           AND due_date <= ? + INTERVAL 7 DAY",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .bind(today)
    .execute(pool)
    .await?;
    Ok(())
}
