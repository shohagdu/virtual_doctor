use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
// SQL below is assembled only from compile-time constants; values are always bound.
use sqlx::AssertSqlSafe;
use vd_domain::patient::{NewPatient, Patient, Sex, patient_code};

use crate::{DbResult, MySqlPool};

const COLUMNS: &str = "id, patient_code, title, name_en, name_bn, phone, dob, dob_estimated, sex, \
                       blood_group, address, district, created_at";

#[derive(sqlx::FromRow)]
struct PatientRow {
    id: u64,
    patient_code: String,
    title: Option<String>,
    name_en: String,
    name_bn: Option<String>,
    phone: String,
    dob: NaiveDate,
    dob_estimated: bool,
    sex: String,
    blood_group: Option<String>,
    address: Option<String>,
    district: Option<String>,
    created_at: NaiveDateTime,
}

impl From<PatientRow> for Patient {
    fn from(r: PatientRow) -> Self {
        Self {
            id: r.id,
            patient_code: r.patient_code,
            title: r.title,
            name_en: r.name_en,
            name_bn: r.name_bn,
            phone: r.phone,
            dob: r.dob,
            dob_estimated: r.dob_estimated,
            sex: Sex::parse(&r.sex).unwrap_or(Sex::O),
            blood_group: r.blood_group,
            address: r.address,
            district: r.district,
            created_at: r.created_at.and_utc(),
        }
    }
}

pub async fn get(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<Option<Patient>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM patients WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL"
    );
    let row: Option<PatientRow> = sqlx::query_as(AssertSqlSafe(sql))
        .bind(clinic_id)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(Into::into))
}

/// Possible duplicates: same phone, and a similar name (PM-02). Family members share
/// a phone (PM-03), so only a name match on that phone counts as a duplicate.
pub async fn find_duplicates(
    pool: &MySqlPool,
    clinic_id: u64,
    phone: &str,
    name_en: &str,
) -> DbResult<Vec<Patient>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM patients
         WHERE clinic_id = ? AND phone = ? AND deleted_at IS NULL
           AND (name_en = ? OR SOUNDEX(name_en) = SOUNDEX(?))
         LIMIT 10"
    );
    let rows: Vec<PatientRow> = sqlx::query_as(AssertSqlSafe(sql))
        .bind(clinic_id)
        .bind(phone)
        .bind(name_en)
        .bind(name_en)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// Inserts the patient and assigns the next per-clinic, per-year patient code.
pub async fn create(
    pool: &MySqlPool,
    clinic_id: u64,
    created_by: u64,
    p: &NewPatient,
) -> DbResult<Patient> {
    let year = Utc::now().year();
    let mut tx = pool.begin().await?;

    // Row-locked counter: concurrent registrations never get the same code.
    sqlx::query(
        "INSERT INTO clinic_counters (clinic_id, name, year, value) VALUES (?, 'patient', ?, 1)
         ON DUPLICATE KEY UPDATE value = value + 1",
    )
    .bind(clinic_id)
    .bind(year)
    .execute(&mut *tx)
    .await?;
    let seq: u64 = sqlx::query_scalar(
        "SELECT value FROM clinic_counters WHERE clinic_id = ? AND name = 'patient' AND year = ?",
    )
    .bind(clinic_id)
    .bind(year)
    .fetch_one(&mut *tx)
    .await?;

    let res = sqlx::query(
        "INSERT INTO patients (clinic_id, patient_code, title, name_en, name_bn, phone, dob,
            dob_estimated, sex, blood_group, address, district, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(patient_code(year, seq))
    .bind(&p.title)
    .bind(&p.name_en)
    .bind(&p.name_bn)
    .bind(&p.phone)
    .bind(p.dob)
    .bind(p.dob_estimated)
    .bind(p.sex.as_str())
    .bind(&p.blood_group)
    .bind(&p.address)
    .bind(&p.district)
    .bind(created_by)
    .execute(&mut *tx)
    .await?;
    let id = res.last_insert_id();

    sqlx::query("INSERT INTO patient_medical_profile (patient_id) VALUES (?)")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(get(pool, clinic_id, id)
        .await?
        .expect("patient just inserted"))
}

/// Replaces the demographic fields (PATCH /patients/{id}). Returns `None` if not found.
pub async fn update(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    p: &NewPatient,
) -> DbResult<Option<Patient>> {
    let res = sqlx::query(
        "UPDATE patients SET title = ?, name_en = ?, name_bn = ?, phone = ?, dob = ?,
            dob_estimated = ?, sex = ?, blood_group = ?, address = ?, district = ?
         WHERE clinic_id = ? AND id = ? AND deleted_at IS NULL",
    )
    .bind(&p.title)
    .bind(&p.name_en)
    .bind(&p.name_bn)
    .bind(&p.phone)
    .bind(p.dob)
    .bind(p.dob_estimated)
    .bind(p.sex.as_str())
    .bind(&p.blood_group)
    .bind(&p.address)
    .bind(&p.district)
    .bind(clinic_id)
    .bind(id)
    .execute(pool)
    .await?;
    if res.rows_affected() == 0 && get(pool, clinic_id, id).await?.is_none() {
        return Ok(None);
    }
    get(pool, clinic_id, id).await
}

/// Search by name (English/Bangla), phone prefix, or patient code. Newest first.
pub async fn search(
    pool: &MySqlPool,
    clinic_id: u64,
    q: &str,
    page: u32,
    per_page: u32,
) -> DbResult<(Vec<Patient>, i64)> {
    let q = q.trim();
    let offset = page.saturating_sub(1) as u64 * per_page as u64;
    let filter = "clinic_id = ? AND deleted_at IS NULL AND (? = '' OR patient_code = ? \
                  OR phone LIKE CONCAT(?, '%') OR name_en LIKE CONCAT('%', ?, '%') \
                  OR name_bn LIKE CONCAT('%', ?, '%'))";

    let total: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM patients WHERE {filter}"
    )))
    .bind(clinic_id)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .fetch_one(pool)
    .await?;

    let rows: Vec<PatientRow> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM patients WHERE {filter} ORDER BY id DESC LIMIT ? OFFSET ?"
    )))
    .bind(clinic_id)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((rows.into_iter().map(Into::into).collect(), total))
}

/// LIKE pattern matching `term` anywhere, with LIKE's wildcards escaped.
fn contains(term: &str) -> String {
    let escaped = term
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{}%", escaped.to_lowercase())
}

/// Patients matching smart-search filters (PM spec §7.2). Claude only turns the
/// sentence into these filters; this query runs locally on the clinic's data.
pub async fn filter(
    pool: &MySqlPool,
    clinic_id: u64,
    f: &vd_domain::report::SearchFilters,
    today: NaiveDate,
) -> DbResult<Vec<Patient>> {
    enum Arg {
        Text(String),
        Date(NaiveDate),
    }
    let mut sql = format!(
        "SELECT {} FROM patients p LEFT JOIN patient_medical_profile m ON m.patient_id = p.id
         WHERE p.clinic_id = ? AND p.deleted_at IS NULL",
        COLUMNS
            .split(", ")
            .map(|c| format!("p.{}", c.trim()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut args: Vec<Arg> = vec![];
    let any_like = |sql: &mut String, args: &mut Vec<Arg>, expr: &str, terms: &[String]| {
        if terms.is_empty() {
            return;
        }
        let parts = vec![format!("{expr} LIKE ?"); terms.len()].join(" OR ");
        sql.push_str(&format!(" AND ({parts})"));
        args.extend(terms.iter().map(|t| Arg::Text(contains(t))));
    };

    if let Some(sex) = &f.sex {
        sql.push_str(" AND p.sex = ?");
        args.push(Arg::Text(sex.clone()));
    }
    if let Some(min) = f.age_min {
        sql.push_str(" AND p.dob <= ?");
        args.push(Arg::Date(today - chrono::Months::new(12 * min as u32)));
    }
    if let Some(max) = f.age_max {
        sql.push_str(" AND p.dob > ?");
        args.push(Arg::Date(
            today - chrono::Months::new(12 * (max as u32 + 1)),
        ));
    }
    any_like(
        &mut sql,
        &mut args,
        "LOWER(CAST(m.chronic_conditions AS CHAR))",
        &f.chronic_terms,
    );

    let visit_filter = !f.diagnosis_terms.is_empty()
        || !f.complaint_terms.is_empty()
        || !f.medicine_terms.is_empty()
        || f.visited_from.is_some()
        || f.visited_to.is_some();
    if visit_filter {
        let mut sub = String::from(
            " AND EXISTS (SELECT 1 FROM visits v LEFT JOIN prescriptions rx ON rx.visit_id = v.id
              WHERE v.patient_id = p.id AND v.deleted_at IS NULL AND v.status <> 'cancelled'",
        );
        if let Some(d) = f.visited_from {
            sub.push_str(" AND v.visit_date >= ?");
            args.push(Arg::Date(d));
        }
        if let Some(d) = f.visited_to {
            sub.push_str(" AND v.visit_date <= ?");
            args.push(Arg::Date(d));
        }
        any_like(
            &mut sub,
            &mut args,
            "LOWER(CAST(rx.diagnosis AS CHAR))",
            &f.diagnosis_terms,
        );
        any_like(
            &mut sub,
            &mut args,
            "LOWER(CAST(rx.complaints AS CHAR))",
            &f.complaint_terms,
        );
        if !f.medicine_terms.is_empty() {
            let parts = vec![
                "LOWER(pm.brand) LIKE ? OR LOWER(COALESCE(pm.generic, '')) LIKE ?";
                f.medicine_terms.len()
            ]
            .join(" OR ");
            sub.push_str(&format!(
                " AND EXISTS (SELECT 1 FROM prescription_medicines pm WHERE pm.prescription_id = rx.id AND ({parts}))"
            ));
            for t in &f.medicine_terms {
                args.push(Arg::Text(contains(t)));
                args.push(Arg::Text(contains(t)));
            }
        }
        sub.push(')');
        sql.push_str(&sub);
    }
    sql.push_str(" ORDER BY p.name_en LIMIT 100");

    let mut q = sqlx::query_as::<_, PatientRow>(AssertSqlSafe(sql)).bind(clinic_id);
    for a in args {
        q = match a {
            Arg::Text(t) => q.bind(t),
            Arg::Date(d) => q.bind(d),
        };
    }
    Ok(q.fetch_all(pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}
