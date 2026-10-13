use serde::Serialize;

use crate::{DbResult, MySqlPool};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MedicineSuggestion {
    pub medicine_id: Option<u64>,
    pub brand: String,
    pub generic: Option<String>,
    pub strength: Option<String>,
    pub form: Option<String>,
}

/// Brand/generic suggestions for the ℞ editor: the medicine catalogue first, then
/// medicines this clinic has prescribed before (most used first).
pub async fn search(
    pool: &MySqlPool,
    clinic_id: u64,
    q: &str,
    limit: usize,
) -> DbResult<Vec<MedicineSuggestion>> {
    let q = q.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let mut out: Vec<MedicineSuggestion> = sqlx::query_as(
        "SELECT id AS medicine_id, brand, generic, strength, form FROM medicines
         WHERE is_active = 1 AND (clinic_id IS NULL OR clinic_id = ?)
           AND (brand LIKE CONCAT(?, '%') OR generic LIKE CONCAT(?, '%'))
         ORDER BY brand, strength LIMIT ?",
    )
    .bind(clinic_id)
    .bind(q)
    .bind(q)
    .bind(limit as u32)
    .fetch_all(pool)
    .await?;

    let used: Vec<MedicineSuggestion> = sqlx::query_as(
        "SELECT MAX(pm.medicine_id) AS medicine_id, pm.brand, MAX(pm.generic) AS generic,
                pm.strength, MAX(pm.form) AS form
         FROM prescription_medicines pm
         JOIN prescriptions p ON p.id = pm.prescription_id
         JOIN visits v ON v.id = p.visit_id
         WHERE v.clinic_id = ? AND pm.brand LIKE CONCAT(?, '%')
         GROUP BY pm.brand, pm.strength
         ORDER BY COUNT(*) DESC LIMIT ?",
    )
    .bind(clinic_id)
    .bind(q)
    .bind(limit as u32)
    .fetch_all(pool)
    .await?;

    for m in used {
        let dup = out.iter().any(|o| {
            o.brand.eq_ignore_ascii_case(&m.brand) && o.strength.as_deref() == m.strength.as_deref()
        });
        if !dup && out.len() < limit {
            out.push(m);
        }
    }
    Ok(out)
}

/// Fills in missing generic names from the catalogue, or from how this clinic wrote the
/// same brand before, so the allergy check can match by generic (PM-09).
pub async fn fill_generics(
    pool: &MySqlPool,
    clinic_id: u64,
    lines: &mut [vd_domain::prescription::MedicineLine],
) -> DbResult<()> {
    for line in lines.iter_mut().filter(|l| l.generic.is_none()) {
        let brand = line.brand.trim();
        if brand.is_empty() {
            continue;
        }
        let generic: Option<String> = sqlx::query_scalar(
            "SELECT generic FROM (
                SELECT generic, 0 AS rank_ FROM medicines
                 WHERE brand = ? AND is_active = 1 AND generic <> ''
                   AND (clinic_id IS NULL OR clinic_id = ?)
                UNION ALL
                SELECT pm.generic, 1 FROM prescription_medicines pm
                 JOIN prescriptions p ON p.id = pm.prescription_id
                 JOIN visits v ON v.id = p.visit_id
                 WHERE v.clinic_id = ? AND pm.brand = ? AND pm.generic IS NOT NULL
             ) g ORDER BY rank_ LIMIT 1",
        )
        .bind(brand)
        .bind(clinic_id)
        .bind(clinic_id)
        .bind(brand)
        .fetch_optional(pool)
        .await?;
        line.generic = generic;
    }
    Ok(())
}

/// Adds or updates a clinic's catalogue rows (roadmap 2.2), matched on brand and
/// strength. Returns (added, updated).
pub async fn import(
    pool: &MySqlPool,
    clinic_id: u64,
    rows: &[vd_domain::catalogue::CatalogueRow],
) -> DbResult<(usize, usize)> {
    let mut tx = pool.begin().await?;
    let (mut added, mut updated) = (0, 0);
    for r in rows {
        let existing: Option<u64> = sqlx::query_scalar(
            "SELECT id FROM medicines WHERE clinic_id = ? AND brand = ? AND strength <=> ? LIMIT 1",
        )
        .bind(clinic_id)
        .bind(&r.brand)
        .bind(&r.strength)
        .fetch_optional(&mut *tx)
        .await?;
        match existing {
            Some(id) => {
                sqlx::query(
                    "UPDATE medicines SET generic = ?, form = ?, company = ?, price = ?, is_active = 1 WHERE id = ?",
                )
                .bind(&r.generic)
                .bind(&r.form)
                .bind(&r.company)
                .bind(r.price)
                .bind(id)
                .execute(&mut *tx)
                .await?;
                updated += 1;
            }
            None => {
                sqlx::query(
                    "INSERT INTO medicines (clinic_id, brand, generic, strength, form, company, price)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(clinic_id)
                .bind(&r.brand)
                .bind(&r.generic)
                .bind(&r.strength)
                .bind(&r.form)
                .bind(&r.company)
                .bind(r.price)
                .execute(&mut *tx)
                .await?;
                added += 1;
            }
        }
    }
    tx.commit().await?;
    Ok((added, updated))
}

/// Catalogue size (active and inactive): the clinic's own rows and the shared ones.
pub async fn catalogue_size(pool: &MySqlPool, clinic_id: u64) -> DbResult<(i64, i64)> {
    sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(clinic_id = ?), 0) AS SIGNED), CAST(COALESCE(SUM(clinic_id IS NULL), 0) AS SIGNED)
         FROM medicines",
    )
    .bind(clinic_id)
    .fetch_one(pool)
    .await
}

// ---- Managing the catalogue (Medicines page) ----

use vd_common::AppError;
use vd_domain::medicine::{Medicine, MedicineInput};

#[derive(sqlx::FromRow)]
struct MedicineRow {
    id: u64,
    clinic_id: Option<u64>,
    brand: String,
    generic: String,
    strength: Option<String>,
    form: Option<String>,
    company: Option<String>,
    price: Option<f64>,
    is_active: bool,
    used: i64,
}

impl From<MedicineRow> for Medicine {
    fn from(r: MedicineRow) -> Self {
        Self {
            id: r.id,
            brand: r.brand,
            generic: r.generic,
            strength: r.strength,
            form: r.form,
            company: r.company,
            price: r.price,
            is_active: r.is_active,
            shared: r.clinic_id.is_none(),
            used: r.used,
        }
    }
}

const RECORD_COLUMNS: &str = "m.id, m.clinic_id, m.brand, m.generic, m.strength, m.form, m.company,
    CAST(m.price AS DOUBLE) AS price, m.is_active,
    (SELECT CAST(COUNT(*) AS SIGNED) FROM prescription_medicines pm WHERE pm.medicine_id = m.id) AS used";

/// Filters for the Medicines page.
pub struct ListFilter<'a> {
    pub q: &'a str,
    pub form: Option<&'a str>,
    pub company: Option<&'a str>,
    /// Some(true) active only, Some(false) inactive only, None both.
    pub active: Option<bool>,
    pub page: u32,
    pub per_page: u32,
}

const VISIBLE: &str = "(m.clinic_id = ? OR m.clinic_id IS NULL)";

/// The clinic's catalogue (own and shared rows), by brand. Returns (rows, total).
pub async fn list(
    pool: &MySqlPool,
    clinic_id: u64,
    f: &ListFilter<'_>,
) -> DbResult<(Vec<Medicine>, i64)> {
    let filter = format!(
        "{VISIBLE} AND (? = '' OR m.brand LIKE CONCAT('%', ?, '%') OR m.generic LIKE CONCAT('%', ?, '%')
            OR m.company LIKE CONCAT('%', ?, '%'))
          AND (? IS NULL OR m.form = ?) AND (? IS NULL OR m.company = ?) AND (? IS NULL OR m.is_active = ?)"
    );
    let q = f.q.trim();
    let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT CAST(COUNT(*) AS SIGNED) FROM medicines m WHERE {filter}"
    )))
    .bind(clinic_id)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(f.form)
    .bind(f.form)
    .bind(f.company)
    .bind(f.company)
    .bind(f.active)
    .bind(f.active)
    .fetch_one(pool)
    .await?;
    let rows: Vec<MedicineRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {RECORD_COLUMNS} FROM medicines m WHERE {filter}
         ORDER BY m.brand, m.strength, m.id LIMIT ? OFFSET ?"
    )))
    .bind(clinic_id)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(f.form)
    .bind(f.form)
    .bind(f.company)
    .bind(f.company)
    .bind(f.active)
    .bind(f.active)
    .bind(f.per_page)
    .bind(f.page.saturating_sub(1) as u64 * f.per_page as u64)
    .fetch_all(pool)
    .await?;
    Ok((rows.into_iter().map(Into::into).collect(), total))
}

/// Every catalogue row the clinic sees, for export.
pub async fn all(pool: &MySqlPool, clinic_id: u64) -> DbResult<Vec<Medicine>> {
    let rows: Vec<MedicineRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {RECORD_COLUMNS} FROM medicines m WHERE {VISIBLE} ORDER BY m.brand, m.strength, m.id"
    )))
    .bind(clinic_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &MySqlPool, clinic_id: u64, id: u64) -> DbResult<Option<Medicine>> {
    let row: Option<MedicineRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {RECORD_COLUMNS} FROM medicines m WHERE {VISIBLE} AND m.id = ?"
    )))
    .bind(clinic_id)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

/// Companies in use, for the filter.
pub async fn companies(pool: &MySqlPool, clinic_id: u64) -> DbResult<Vec<String>> {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT DISTINCT m.company FROM medicines m WHERE {VISIBLE} AND m.company IS NOT NULL ORDER BY m.company"
    )))
    .bind(clinic_id)
    .fetch_all(pool)
    .await
}

/// Dosage forms in use, for the filter.
pub async fn forms(pool: &MySqlPool, clinic_id: u64) -> DbResult<Vec<String>> {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT DISTINCT m.form FROM medicines m WHERE {VISIBLE} AND m.form IS NOT NULL ORDER BY m.form"
    )))
    .bind(clinic_id)
    .fetch_all(pool)
    .await
}

fn db(e: sqlx::Error) -> AppError {
    AppError::internal(e)
}

/// Another of the clinic's medicines with the same brand and strength.
async fn duplicate(
    pool: &MySqlPool,
    clinic_id: u64,
    m: &MedicineInput,
    except: Option<u64>,
) -> Result<(), AppError> {
    let found: Option<u64> = sqlx::query_scalar(
        "SELECT id FROM medicines WHERE clinic_id = ? AND brand = ? AND strength <=> ? AND id <> ? LIMIT 1",
    )
    .bind(clinic_id)
    .bind(&m.brand)
    .bind(&m.strength)
    .bind(except.unwrap_or(0))
    .fetch_optional(pool)
    .await
    .map_err(db)?;
    match found {
        Some(_) => Err(AppError::Conflict(format!(
            "{} {} is already in the list",
            m.brand,
            m.strength.as_deref().unwrap_or_default()
        ))),
        None => Ok(()),
    }
}

pub async fn create(pool: &MySqlPool, clinic_id: u64, m: &MedicineInput) -> Result<u64, AppError> {
    duplicate(pool, clinic_id, m, None).await?;
    Ok(sqlx::query(
        "INSERT INTO medicines (clinic_id, brand, generic, strength, form, company, price)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(clinic_id)
    .bind(&m.brand)
    .bind(&m.generic)
    .bind(&m.strength)
    .bind(&m.form)
    .bind(&m.company)
    .bind(m.price)
    .execute(pool)
    .await
    .map_err(db)?
    .last_insert_id())
}

/// The clinic's own row, or an error explaining why it cannot be changed.
async fn own(pool: &MySqlPool, clinic_id: u64, id: u64) -> Result<Medicine, AppError> {
    let m = get(pool, clinic_id, id)
        .await
        .map_err(db)?
        .ok_or(AppError::NotFound)?;
    if m.shared {
        return Err(AppError::Conflict(
            "this is a shared catalogue medicine and cannot be changed here".into(),
        ));
    }
    Ok(m)
}

pub async fn update(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    m: &MedicineInput,
) -> Result<(), AppError> {
    own(pool, clinic_id, id).await?;
    duplicate(pool, clinic_id, m, Some(id)).await?;
    sqlx::query(
        "UPDATE medicines SET brand = ?, generic = ?, strength = ?, form = ?, company = ?, price = ?
         WHERE id = ? AND clinic_id = ?",
    )
    .bind(&m.brand)
    .bind(&m.generic)
    .bind(&m.strength)
    .bind(&m.form)
    .bind(&m.company)
    .bind(m.price)
    .bind(id)
    .bind(clinic_id)
    .execute(pool)
    .await
    .map_err(db)?;
    Ok(())
}

/// Inactive medicines are no longer suggested in the ℞ editor but stay on old prescriptions.
pub async fn set_active(
    pool: &MySqlPool,
    clinic_id: u64,
    id: u64,
    active: bool,
) -> Result<(), AppError> {
    own(pool, clinic_id, id).await?;
    sqlx::query("UPDATE medicines SET is_active = ? WHERE id = ? AND clinic_id = ?")
        .bind(active)
        .bind(id)
        .bind(clinic_id)
        .execute(pool)
        .await
        .map_err(db)?;
    Ok(())
}

/// Deletes a medicine that no prescription uses; a used one can only be deactivated.
pub async fn delete(pool: &MySqlPool, clinic_id: u64, id: u64) -> Result<(), AppError> {
    let m = own(pool, clinic_id, id).await?;
    if m.used > 0 {
        return Err(AppError::Conflict(format!(
            "{} is used in {} prescription line(s) — deactivate it instead",
            m.brand, m.used
        )));
    }
    sqlx::query("DELETE FROM medicines WHERE id = ? AND clinic_id = ?")
        .bind(id)
        .bind(clinic_id)
        .execute(pool)
        .await
        .map_err(db)?;
    Ok(())
}
