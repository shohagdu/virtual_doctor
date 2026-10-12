use chrono::{NaiveDate, NaiveDateTime};
use sqlx::types::Json;
use sqlx::{MySql, Transaction};
use vd_common::AppError;
use vd_domain::prescription::{
    BaseVersion, MedicineLine, Prescription, PrescriptionBody, SavePlan, plan_save,
};

use crate::MySqlPool;

#[derive(sqlx::FromRow)]
struct Row {
    id: u64,
    visit_id: u64,
    version: u32,
    revision: u32,
    locked: bool,
    complaints: Option<Json<Vec<String>>>,
    examination: Option<Json<Vec<String>>>,
    investigations: Option<Json<Vec<String>>>,
    diagnosis: Option<Json<Vec<String>>>,
    treatment_plan: Option<Json<Vec<String>>>,
    advice: Option<Json<Vec<String>>>,
    next_plan: Option<Json<Vec<String>>>,
    operation_notes: Option<String>,
    next_visit_date: Option<NaiveDate>,
    updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub(crate) struct MedRow {
    pub prescription_id: u64,
    medicine_id: Option<u64>,
    brand: String,
    generic: Option<String>,
    strength: Option<String>,
    form: Option<String>,
    dose_pattern: Option<String>,
    timing: Option<String>,
    duration: Option<String>,
    notes: Option<String>,
}

pub(crate) const MED_COLUMNS: &str = "prescription_id, medicine_id, brand, generic, strength, \
                                        form, dose_pattern, timing, duration, notes";

impl From<MedRow> for MedicineLine {
    fn from(m: MedRow) -> Self {
        Self {
            medicine_id: m.medicine_id,
            brand: m.brand,
            generic: m.generic,
            strength: m.strength,
            form: m.form,
            dose_pattern: m.dose_pattern,
            timing: m.timing,
            duration: m.duration,
            notes: m.notes,
        }
    }
}

fn list(v: Option<Json<Vec<String>>>) -> Vec<String> {
    v.map(|j| j.0).unwrap_or_default()
}

fn db(e: sqlx::Error) -> AppError {
    AppError::internal(e)
}

/// Latest version for a visit, or an empty draft if nothing was saved yet.
pub async fn latest(pool: &MySqlPool, visit_id: u64) -> Result<Prescription, AppError> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, visit_id, version, revision, locked, complaints, examination, investigations,
                diagnosis, treatment_plan, advice, next_plan, operation_notes, next_visit_date,
                updated_at
         FROM prescriptions WHERE visit_id = ? ORDER BY version DESC LIMIT 1",
    )
    .bind(visit_id)
    .fetch_optional(pool)
    .await
    .map_err(db)?;

    let Some(r) = row else {
        return Ok(Prescription {
            id: None,
            visit_id,
            version: 0,
            revision: 0,
            locked: false,
            body: PrescriptionBody::default(),
            updated_at: None,
        });
    };

    let meds: Vec<MedRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {MED_COLUMNS} FROM prescription_medicines
         WHERE prescription_id = ? ORDER BY sort_order, id"
    )))
    .bind(r.id)
    .fetch_all(pool)
    .await
    .map_err(db)?;

    Ok(Prescription {
        id: Some(r.id),
        visit_id: r.visit_id,
        version: r.version,
        revision: r.revision,
        locked: r.locked,
        body: PrescriptionBody {
            complaints: list(r.complaints),
            examination: list(r.examination),
            investigations: list(r.investigations),
            diagnosis: list(r.diagnosis),
            treatment_plan: list(r.treatment_plan),
            advice: list(r.advice),
            next_plan: list(r.next_plan),
            operation_notes: r.operation_notes,
            next_visit_date: r.next_visit_date,
            medicines: meds.into_iter().map(Into::into).collect(),
        },
        updated_at: Some(r.updated_at.and_utc()),
    })
}

/// (id+revision, version, locked) of the latest version, row-locked for this transaction.
async fn latest_for_update(
    tx: &mut Transaction<'_, MySql>,
    visit_id: u64,
) -> Result<Option<(BaseVersion, u32, bool)>, AppError> {
    let row: Option<(u64, u32, u32, bool)> = sqlx::query_as(
        "SELECT id, revision, version, locked FROM prescriptions
         WHERE visit_id = ? ORDER BY version DESC LIMIT 1 FOR UPDATE",
    )
    .bind(visit_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db)?;
    Ok(row.map(|(id, revision, version, locked)| (BaseVersion { id, revision }, version, locked)))
}

pub struct SaveContext {
    pub clinic_id: u64,
    pub patient_id: u64,
    pub visit_id: u64,
    pub user_id: u64,
}

/// Saves a prescription: updates the open draft, or starts a new version when the latest
/// one is locked (PM-11). Keeps the visit's follow-up in step with Next Visit (PM-16).
pub async fn save(
    pool: &MySqlPool,
    ctx: &SaveContext,
    base: Option<BaseVersion>,
    body: &PrescriptionBody,
) -> Result<Prescription, AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    let plan = plan_save(latest_for_update(&mut tx, ctx.visit_id).await?, base)?;

    let rx_id = match plan {
        SavePlan::Insert { version } => sqlx::query(
            "INSERT INTO prescriptions (visit_id, version, revision, complaints, examination,
                investigations, diagnosis, treatment_plan, advice, next_plan, operation_notes,
                next_visit_date, created_by)
             VALUES (?, ?, 0, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(ctx.visit_id)
        .bind(version)
        .bind(Json(&body.complaints))
        .bind(Json(&body.examination))
        .bind(Json(&body.investigations))
        .bind(Json(&body.diagnosis))
        .bind(Json(&body.treatment_plan))
        .bind(Json(&body.advice))
        .bind(Json(&body.next_plan))
        .bind(&body.operation_notes)
        .bind(body.next_visit_date)
        .bind(ctx.user_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?
        .last_insert_id(),
        SavePlan::Update { id, revision } => {
            sqlx::query(
                "UPDATE prescriptions SET revision = ?, complaints = ?, examination = ?,
                    investigations = ?, diagnosis = ?, treatment_plan = ?, advice = ?,
                    next_plan = ?, operation_notes = ?, next_visit_date = ?
                 WHERE id = ?",
            )
            .bind(revision)
            .bind(Json(&body.complaints))
            .bind(Json(&body.examination))
            .bind(Json(&body.investigations))
            .bind(Json(&body.diagnosis))
            .bind(Json(&body.treatment_plan))
            .bind(Json(&body.advice))
            .bind(Json(&body.next_plan))
            .bind(&body.operation_notes)
            .bind(body.next_visit_date)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
            sqlx::query("DELETE FROM prescription_medicines WHERE prescription_id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            id
        }
    };

    for (i, m) in body.medicines.iter().enumerate() {
        sqlx::query(
            "INSERT INTO prescription_medicines (prescription_id, sort_order, medicine_id, brand,
                generic, strength, form, dose_pattern, timing, duration, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(rx_id)
        .bind(i as u32)
        .bind(m.medicine_id)
        .bind(&m.brand)
        .bind(&m.generic)
        .bind(&m.strength)
        .bind(&m.form)
        .bind(&m.dose_pattern)
        .bind(&m.timing)
        .bind(&m.duration)
        .bind(&m.notes)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    }

    // One pending follow-up per visit, matching the latest Next Visit date.
    sqlx::query(
        "DELETE FROM followups WHERE status = 'pending'
           AND prescription_id IN (SELECT id FROM prescriptions WHERE visit_id = ?)",
    )
    .bind(ctx.visit_id)
    .execute(&mut *tx)
    .await
    .map_err(db)?;
    if let Some(due) = body.next_visit_date {
        sqlx::query(
            "INSERT INTO followups (clinic_id, patient_id, prescription_id, due_date) VALUES (?, ?, ?, ?)",
        )
        .bind(ctx.clinic_id)
        .bind(ctx.patient_id)
        .bind(rx_id)
        .bind(due)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    }

    // Writing a prescription means the doctor is seeing the patient now.
    sqlx::query("UPDATE visits SET status = 'in_consultation' WHERE id = ? AND status = 'waiting'")
        .bind(ctx.visit_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;

    tx.commit().await.map_err(db)?;
    latest(pool, ctx.visit_id).await
}

/// Finalises the latest version (on print / PDF). Locked versions are never edited again;
/// the next save starts a new version. Marks the visit as seen.
pub async fn lock(
    pool: &MySqlPool,
    visit_id: u64,
    base: BaseVersion,
) -> Result<Prescription, AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    match latest_for_update(&mut tx, visit_id).await? {
        Some((current, _, _)) if current == base => {}
        _ => {
            return Err(AppError::Conflict(
                "this prescription was changed by someone else — reload before printing".into(),
            ));
        }
    }
    sqlx::query("UPDATE prescriptions SET locked = 1 WHERE id = ?")
        .bind(base.id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    sqlx::query("UPDATE visits SET status = 'seen' WHERE id = ? AND status <> 'cancelled'")
        .bind(visit_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;
    latest(pool, visit_id).await
}
