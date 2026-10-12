//! Investigation reports and their results (AI spec §10).

use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;
use sqlx::types::Json;
use vd_domain::lab::{self, RangeSource, Status, SystemRange};
use vd_domain::patient::Sex;
use vd_domain::report::{CleanResult, Confidence, Report, ReportResult, ReportStatus};

use crate::{DbResult, MySqlPool};

#[derive(sqlx::FromRow)]
struct Row {
    id: u64,
    patient_id: u64,
    visit_id: Option<u64>,
    report_type: Option<String>,
    lab_name: Option<String>,
    report_date: Option<NaiveDate>,
    mime_type: String,
    original_name: Option<String>,
    status: String,
    impression: Option<String>,
    name_mismatch: bool,
    unreadable: Option<Json<Vec<String>>>,
    error: Option<String>,
    created_at: NaiveDateTime,
}

const COLUMNS: &str = "id, patient_id, visit_id, report_type, lab_name, report_date, mime_type,
    original_name, status, impression, name_mismatch,
    JSON_EXTRACT(extraction_raw, '$.unreadable_parts') AS unreadable, error, created_at";

#[derive(sqlx::FromRow)]
struct ResultRow {
    id: u64,
    report_id: u64,
    test_code: String,
    test_name: String,
    value_text: Option<String>,
    value_num: Option<f64>,
    unit: Option<String>,
    ref_range_text: Option<String>,
    lab_flag: Option<String>,
    range_source: Option<String>,
    status: Option<String>,
    confidence: Option<String>,
    edited_by_doctor: bool,
}

const RESULT_COLUMNS: &str = "id, report_id, test_code, test_name, value_text,
    CAST(value_num AS DOUBLE) AS value_num, unit, ref_range_text, lab_flag, range_source,
    status, confidence, edited_by_doctor";

impl From<ResultRow> for ReportResult {
    fn from(r: ResultRow) -> Self {
        Self {
            id: r.id,
            test_code: r.test_code,
            test_name: r.test_name,
            value_text: r.value_text.unwrap_or_default(),
            value_num: r.value_num,
            unit: r.unit,
            ref_range_text: r.ref_range_text,
            lab_flag: r.lab_flag,
            range_source: match r.range_source.as_deref() {
                Some("lab") => Some(RangeSource::Lab),
                Some("system") => Some(RangeSource::System),
                _ => None,
            },
            status: r.status.as_deref().and_then(Status::parse),
            confidence: r.confidence.as_deref().and_then(Confidence::parse),
            edited_by_doctor: r.edited_by_doctor,
        }
    }
}

async fn with_results(pool: &MySqlPool, rows: Vec<Row>) -> DbResult<Vec<Report>> {
    if rows.is_empty() {
        return Ok(vec![]);
    }
    let ids = rows
        .iter()
        .map(|r| r.id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let results: Vec<ResultRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {RESULT_COLUMNS} FROM report_results WHERE report_id IN ({ids})
         ORDER BY report_id, sort_order, id"
    )))
    .fetch_all(pool)
    .await?;
    let mut out: Vec<Report> = rows
        .into_iter()
        .map(|r| Report {
            id: r.id,
            patient_id: r.patient_id,
            visit_id: r.visit_id,
            report_type: r.report_type,
            lab_name: r.lab_name,
            report_date: r.report_date,
            mime_type: r.mime_type,
            original_name: r.original_name,
            status: ReportStatus::parse(&r.status).unwrap_or(ReportStatus::Uploaded),
            impression: r.impression,
            name_mismatch: r.name_mismatch,
            unreadable_parts: r.unreadable.map(|j| j.0).unwrap_or_default(),
            error: r.error,
            created_at: r.created_at.and_utc(),
            results: vec![],
        })
        .collect();
    for res in results {
        if let Some(rep) = out.iter_mut().find(|r| r.id == res.report_id) {
            rep.results.push(res.into());
        }
    }
    Ok(out)
}

pub async fn get(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<Option<Report>> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM reports WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL"
    )))
    .bind(clinic_id)
    .bind(id)
    .fetch_all(pool)
    .await?;
    Ok(with_results(pool, rows).await?.pop())
}

pub async fn for_visit(pool: &MySqlPool, clinic_id: u64, visit_id: u64) -> DbResult<Vec<Report>> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM reports
         WHERE clinic_id = ? AND visit_id = ? AND deleted_at IS NULL ORDER BY id"
    )))
    .bind(clinic_id)
    .bind(visit_id)
    .fetch_all(pool)
    .await?;
    with_results(pool, rows).await
}

/// All of a patient's reports, newest first (FR-38).
pub async fn for_patient(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
) -> DbResult<Vec<Report>> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM reports
         WHERE clinic_id = ? AND patient_id = ? AND deleted_at IS NULL
         ORDER BY COALESCE(report_date, DATE(created_at)) DESC, id DESC"
    )))
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_all(pool)
    .await?;
    with_results(pool, rows).await
}

/// A report of this patient with the same file contents (no repeat AI cost).
pub async fn find_by_hash(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    hash: &str,
) -> DbResult<Option<u64>> {
    sqlx::query_scalar(
        "SELECT id FROM reports
         WHERE clinic_id = ? AND file_hash = ? AND patient_id = ? AND deleted_at IS NULL
         ORDER BY id DESC LIMIT 1",
    )
    .bind(clinic_id)
    .bind(hash)
    .bind(patient_id)
    .fetch_optional(pool)
    .await
}

pub struct NewReport<'a> {
    pub clinic_id: u64,
    pub patient_id: u64,
    pub visit_id: u64,
    pub file_path: &'a str,
    pub file_hash: &'a str,
    pub mime_type: &'a str,
    pub original_name: Option<&'a str>,
    pub uploaded_by: u64,
}

pub async fn create(pool: &MySqlPool, r: &NewReport<'_>) -> DbResult<u64> {
    Ok(sqlx::query(
        "INSERT INTO reports (clinic_id, patient_id, visit_id, file_path, file_hash, mime_type,
             original_name, uploaded_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(r.clinic_id)
    .bind(r.patient_id)
    .bind(r.visit_id)
    .bind(r.file_path)
    .bind(r.file_hash)
    .bind(r.mime_type)
    .bind(r.original_name)
    .bind(r.uploaded_by)
    .execute(pool)
    .await?
    .last_insert_id())
}

/// Stored file details for download and extraction.
pub async fn file(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
) -> DbResult<Option<(String, String, Option<String>)>> {
    sqlx::query_as(
        "SELECT file_path, mime_type, original_name FROM reports
         WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn set_status(
    pool: &MySqlPool,
    id: u64,
    status: ReportStatus,
    error: Option<&str>,
) -> DbResult<()> {
    sqlx::query("UPDATE reports SET status = ?, error = ? WHERE id = ?")
        .bind(status.as_str())
        .bind(error.map(|e| e.chars().take(255).collect::<String>()))
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub struct Header<'a> {
    pub report_type: Option<&'a str>,
    pub lab_name: Option<&'a str>,
    pub report_date: Option<NaiveDate>,
    pub impression: Option<&'a str>,
}

/// low, high, critical low, critical high.
type Limits = (Option<f64>, Option<f64>, Option<f64>, Option<f64>);

/// The system range for a test, patient sex and age: the clinic's own row first,
/// then the default; a sex-specific row before an "ANY" one.
pub async fn system_range(
    pool: &MySqlPool,
    clinic_id: u64,
    code: &str,
    sex: Sex,
    age_days: i64,
) -> DbResult<Option<SystemRange>> {
    let sex = match sex {
        Sex::M => "M",
        Sex::F => "F",
        Sex::O => "ANY",
    };
    let row: Option<Limits> = sqlx::query_as(
        "SELECT CAST(low AS DOUBLE), CAST(high AS DOUBLE),
                CAST(critical_low AS DOUBLE), CAST(critical_high AS DOUBLE)
         FROM reference_ranges
         WHERE test_code = ? AND sex IN (?, 'ANY') AND ? BETWEEN age_min_days AND age_max_days
           AND (clinic_id = ? OR clinic_id IS NULL)
         ORDER BY clinic_id IS NULL, sex = 'ANY'
         LIMIT 1",
    )
    .bind(code)
    .bind(sex)
    .bind(age_days.max(0))
    .bind(clinic_id)
    .fetch_optional(pool)
    .await?;
    Ok(
        row.map(|(low, high, critical_low, critical_high)| SystemRange {
            low,
            high,
            critical_low,
            critical_high,
        }),
    )
}

/// Grades results (rule engine) and replaces the report's rows; sets the header and
/// the new status in one transaction.
#[allow(clippy::too_many_arguments)]
pub async fn save_results(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    header: &Header<'_>,
    results: &[CleanResult],
    sex: Sex,
    age_days: i64,
    status: ReportStatus,
) -> DbResult<()> {
    // Ranges are looked up before the transaction; they are reference data.
    let mut graded = Vec::with_capacity(results.len());
    for r in results {
        let (code, _) = lab::normalise(&r.test_name, r.unit.as_deref());
        let range = system_range(pool, clinic_id, &code, sex, age_days).await?;
        let a = lab::assess(
            &code,
            &r.value_text,
            r.unit.as_deref(),
            r.ref_range_text.as_deref(),
            r.lab_flag.as_deref(),
            range.as_ref(),
        );
        let num = lab::read_value(&r.value_text).numeric();
        graded.push((code, a, num));
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE reports SET report_type = ?, lab_name = ?, report_date = ?, impression = ?,
             status = ?, error = NULL
         WHERE id = ?",
    )
    .bind(header.report_type)
    .bind(header.lab_name)
    .bind(header.report_date)
    .bind(header.impression)
    .bind(status.as_str())
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM report_results WHERE report_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (i, (r, (code, a, num))) in results.iter().zip(&graded).enumerate() {
        sqlx::query(
            "INSERT INTO report_results (report_id, sort_order, test_code, test_name, value_text,
                 value_num, unit, ref_range_text, lab_flag, range_source, status, confidence,
                 edited_by_doctor)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(i as u32)
        .bind(code)
        .bind(&r.test_name)
        .bind(&r.value_text)
        // DECIMAL(12,4): keep huge or tiny numbers out rather than fail the insert.
        .bind(num.filter(|x| x.abs() < 1e8))
        .bind(&r.unit)
        .bind(&r.ref_range_text)
        .bind(&r.lab_flag)
        .bind(a.source.map(RangeSource::as_str))
        .bind(a.status.map(Status::as_str))
        .bind(r.confidence.map(Confidence::as_str))
        .bind(r.edited)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

/// Keeps Claude's raw extraction and the name check with the report.
pub async fn set_extraction(
    pool: &MySqlPool,
    id: u64,
    raw: &serde_json::Value,
    name_mismatch: bool,
) -> DbResult<()> {
    sqlx::query("UPDATE reports SET extraction_raw = ?, name_mismatch = ? WHERE id = ?")
        .bind(Json(raw))
        .bind(name_mismatch)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<bool> {
    let res = sqlx::query(
        "UPDATE reports SET deleted_at = UTC_TIMESTAMP() WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(clinic_id)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TrendTest {
    pub test_code: String,
    pub test_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TrendPoint {
    pub report_id: u64,
    pub date: NaiveDate,
    pub value: f64,
    pub value_text: Option<String>,
    pub unit: Option<String>,
    pub status: Option<String>,
}

/// Tests with numeric results for the patient, most measured first (FR-39).
pub async fn trend_tests(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
) -> DbResult<Vec<TrendTest>> {
    sqlx::query_as(
        "SELECT rr.test_code, MAX(rr.test_name) AS test_name, CAST(COUNT(*) AS SIGNED) AS count
         FROM report_results rr JOIN reports r ON r.id = rr.report_id
         WHERE r.clinic_id = ? AND r.patient_id = ? AND r.deleted_at IS NULL
           AND rr.value_num IS NOT NULL
         GROUP BY rr.test_code
         ORDER BY count DESC, test_name",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_all(pool)
    .await
}

pub async fn trend(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    code: &str,
) -> DbResult<Vec<TrendPoint>> {
    sqlx::query_as(
        "SELECT r.id AS report_id, COALESCE(r.report_date, DATE(r.created_at)) AS date,
                CAST(rr.value_num AS DOUBLE) AS value, rr.value_text, rr.unit, rr.status
         FROM report_results rr JOIN reports r ON r.id = rr.report_id
         WHERE r.clinic_id = ? AND r.patient_id = ? AND r.deleted_at IS NULL
           AND rr.test_code = ? AND rr.value_num IS NOT NULL
         ORDER BY date, r.id",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .bind(code)
    .fetch_all(pool)
    .await
}

/// Earlier results of the given tests (before `visit_id`), for "change since last
/// report" comments (FR-40).
pub async fn earlier_results(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
    visit_id: u64,
    codes: &[String],
) -> DbResult<Vec<(String, NaiveDate, String, Option<String>)>> {
    if codes.is_empty() {
        return Ok(vec![]);
    }
    let marks = vec!["?"; codes.len()].join(",");
    let mut q = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT rr.test_name, COALESCE(r.report_date, DATE(r.created_at)), rr.value_text, rr.unit
         FROM report_results rr JOIN reports r ON r.id = rr.report_id
         WHERE r.clinic_id = ? AND r.patient_id = ? AND r.deleted_at IS NULL
           AND (r.visit_id IS NULL OR r.visit_id <> ?) AND rr.test_code IN ({marks})
         ORDER BY 2 DESC LIMIT 40"
    )))
    .bind(clinic_id)
    .bind(patient_id)
    .bind(visit_id);
    for c in codes {
        q = q.bind(c);
    }
    q.fetch_all(pool).await
}

/// Abnormal results with dates, for the patient summary (key lab values).
pub async fn abnormal_history(
    pool: &MySqlPool,
    clinic_id: u64,
    patient_id: u64,
) -> DbResult<Vec<(NaiveDate, String, String, Option<String>, String)>> {
    sqlx::query_as(
        "SELECT COALESCE(r.report_date, DATE(r.created_at)), rr.test_name, rr.value_text, rr.unit, rr.status
         FROM report_results rr JOIN reports r ON r.id = rr.report_id
         WHERE r.clinic_id = ? AND r.patient_id = ? AND r.deleted_at IS NULL
           AND rr.status IS NOT NULL AND rr.status <> 'normal'
         ORDER BY 1 DESC LIMIT 60",
    )
    .bind(clinic_id)
    .bind(patient_id)
    .fetch_all(pool)
    .await
}
