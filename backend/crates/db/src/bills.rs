//! Consultation fees and payments (Patient_Management_Spec §5.8, PM-18).

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use vd_common::AppError;
use vd_domain::billing::{Amounts, Bill, BillStatus, CollectionRow, PaymentMethod, round2};
use vd_domain::visit::VisitType;

use crate::{DbResult, MySqlPool};

fn db(e: sqlx::Error) -> AppError {
    AppError::internal(e)
}

const COLUMNS: &str = "b.id, b.visit_id, b.receipt_no,
    CAST(b.fee AS DOUBLE) AS fee, CAST(b.discount AS DOUBLE) AS discount, b.discount_reason,
    CAST(b.paid AS DOUBLE) AS paid, b.method, b.status, b.created_at, b.updated_at";

#[derive(sqlx::FromRow)]
struct Row {
    id: u64,
    visit_id: u64,
    receipt_no: Option<String>,
    fee: f64,
    discount: f64,
    discount_reason: Option<String>,
    paid: f64,
    method: String,
    status: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<Row> for Bill {
    fn from(r: Row) -> Self {
        let total = round2(r.fee - r.discount);
        Self {
            id: r.id,
            visit_id: r.visit_id,
            receipt_no: r.receipt_no.unwrap_or_default(),
            fee: r.fee,
            discount: r.discount,
            discount_reason: r.discount_reason,
            total,
            paid: r.paid,
            due: round2((total - r.paid).max(0.0)),
            method: PaymentMethod::parse(&r.method).unwrap_or_default(),
            status: if r.status == "paid" {
                BillStatus::Paid
            } else {
                BillStatus::Due
            },
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        }
    }
}

pub async fn for_visit(pool: &MySqlPool, clinic_id: u64, visit_id: u64) -> DbResult<Option<Bill>> {
    let row: Option<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM bills b WHERE b.clinic_id = ? AND b.visit_id = ?"
    )))
    .bind(clinic_id)
    .bind(visit_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

/// Creates the visit's bill (with the next receipt number) or updates it, e.g. when
/// a due amount is paid later. Returns the bill and whether it was new.
pub async fn save(
    pool: &MySqlPool,
    clinic_id: u64,
    visit_id: u64,
    a: &Amounts,
    user_id: u64,
) -> Result<(Bill, bool), AppError> {
    let mut tx = pool.begin().await.map_err(db)?;
    let visit_date: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT visit_date FROM visits WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(clinic_id)
    .bind(visit_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?;
    let visit_date = visit_date.ok_or(AppError::NotFound)?;

    let existing: Option<u64> = sqlx::query_scalar("SELECT id FROM bills WHERE visit_id = ?")
        .bind(visit_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db)?;
    let created = existing.is_none();
    match existing {
        Some(id) => {
            sqlx::query(
                "UPDATE bills SET fee = ?, discount = ?, discount_reason = ?, paid = ?, method = ?,
                     status = ?, updated_by = ?
                 WHERE id = ?",
            )
            .bind(a.fee)
            .bind(a.discount)
            .bind(&a.discount_reason)
            .bind(a.paid)
            .bind(a.method.as_str())
            .bind(a.status.as_str())
            .bind(user_id)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        }
        None => {
            let year = visit_date.year();
            sqlx::query(
                "INSERT INTO clinic_counters (clinic_id, name, year, value) VALUES (?, 'receipt', ?, 1)
                 ON DUPLICATE KEY UPDATE value = value + 1",
            )
            .bind(clinic_id)
            .bind(year)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
            let seq: u64 = sqlx::query_scalar(
                "SELECT value FROM clinic_counters WHERE clinic_id = ? AND name = 'receipt' AND year = ?",
            )
            .bind(clinic_id)
            .bind(year)
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
            sqlx::query(
                "INSERT INTO bills (clinic_id, visit_id, receipt_no, fee, discount, discount_reason,
                     paid, method, status, created_by)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(clinic_id)
            .bind(visit_id)
            .bind(format!("R-{year}-{seq:06}"))
            .bind(a.fee)
            .bind(a.discount)
            .bind(&a.discount_reason)
            .bind(a.paid)
            .bind(a.method.as_str())
            .bind(a.status.as_str())
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        }
    }
    tx.commit().await.map_err(db)?;
    let bill = for_visit(pool, clinic_id, visit_id)
        .await
        .map_err(db)?
        .ok_or(AppError::NotFound)?;
    Ok((bill, created))
}

#[derive(sqlx::FromRow)]
struct CollectionDbRow {
    bill_id: u64,
    visit_id: u64,
    visit_date: NaiveDate,
    receipt_no: Option<String>,
    patient_id: u64,
    patient_code: String,
    title: Option<String>,
    name_en: String,
    visit_type: String,
    fee: f64,
    discount: f64,
    paid: f64,
    method: String,
}

impl From<CollectionDbRow> for CollectionRow {
    fn from(r: CollectionDbRow) -> Self {
        let total = round2(r.fee - r.discount);
        CollectionRow {
            bill_id: r.bill_id,
            visit_id: r.visit_id,
            visit_date: r.visit_date,
            receipt_no: r.receipt_no.unwrap_or_default(),
            patient_id: r.patient_id,
            patient_code: r.patient_code,
            patient_name: [r.title.as_deref(), Some(&r.name_en)]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" "),
            visit_type: VisitType::parse(&r.visit_type).unwrap_or_default(),
            fee: r.fee,
            discount: r.discount,
            total,
            paid: r.paid,
            due: round2((total - r.paid).max(0.0)),
            method: PaymentMethod::parse(&r.method).unwrap_or_default(),
        }
    }
}

/// Bills for visits from `from` to `to` (inclusive), for the collection report.
pub async fn collection(
    pool: &MySqlPool,
    clinic_id: u64,
    from: NaiveDate,
    to: NaiveDate,
) -> DbResult<Vec<CollectionRow>> {
    let rows: Vec<CollectionDbRow> = sqlx::query_as(
        "SELECT b.id AS bill_id, v.id AS visit_id, v.visit_date, b.receipt_no,
                p.id AS patient_id, p.patient_code, p.title, p.name_en, v.visit_type,
                CAST(b.fee AS DOUBLE) AS fee, CAST(b.discount AS DOUBLE) AS discount,
                CAST(b.paid AS DOUBLE) AS paid, b.method
         FROM bills b
         JOIN visits v ON v.id = b.visit_id AND v.deleted_at IS NULL
         JOIN patients p ON p.id = v.patient_id
         WHERE b.clinic_id = ? AND v.visit_date BETWEEN ? AND ?
         ORDER BY v.visit_date, b.receipt_no",
    )
    .bind(clinic_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// A patient's bills, newest first (profile Billing tab, §5.4).
pub async fn for_patient(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
) -> DbResult<Vec<CollectionRow>> {
    let rows: Vec<CollectionDbRow> = sqlx::query_as(
        "SELECT b.id AS bill_id, v.id AS visit_id, v.visit_date, b.receipt_no,
                p.id AS patient_id, p.patient_code, p.title, p.name_en, v.visit_type,
                CAST(b.fee AS DOUBLE) AS fee, CAST(b.discount AS DOUBLE) AS discount,
                CAST(b.paid AS DOUBLE) AS paid, b.method
         FROM bills b
         JOIN visits v ON v.id = b.visit_id AND v.deleted_at IS NULL
         JOIN patients p ON p.id = v.patient_id
         WHERE b.clinic_id = ? AND v.patient_id = ?
         ORDER BY v.visit_date DESC, b.id DESC",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}
