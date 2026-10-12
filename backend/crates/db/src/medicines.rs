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
         WHERE is_active = 1 AND (brand LIKE CONCAT(?, '%') OR generic LIKE CONCAT(?, '%'))
         ORDER BY brand, strength LIMIT ?",
    )
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
                UNION ALL
                SELECT pm.generic, 1 FROM prescription_medicines pm
                 JOIN prescriptions p ON p.id = pm.prescription_id
                 JOIN visits v ON v.id = p.visit_id
                 WHERE v.clinic_id = ? AND pm.brand = ? AND pm.generic IS NOT NULL
             ) g ORDER BY rank_ LIMIT 1",
        )
        .bind(brand)
        .bind(clinic_id)
        .bind(brand)
        .fetch_optional(pool)
        .await?;
        line.generic = generic;
    }
    Ok(())
}
