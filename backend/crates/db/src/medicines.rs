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

/// Catalogue size: the clinic's own rows and the shared ones.
pub async fn catalogue_size(pool: &MySqlPool, clinic_id: u64) -> DbResult<(i64, i64)> {
    sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(clinic_id = ?), 0) AS SIGNED), CAST(COALESCE(SUM(clinic_id IS NULL), 0) AS SIGNED)
         FROM medicines WHERE is_active = 1",
    )
    .bind(clinic_id)
    .fetch_one(pool)
    .await
}
