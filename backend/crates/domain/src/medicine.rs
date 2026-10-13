//! Medicine catalogue records (roadmap 2.2): the clinic's brands with their generic,
//! strength and form, used for ℞ suggestions and the allergy check.

use serde::{Deserialize, Serialize};
use vd_common::AppError;

use crate::text::{check_len, opt};

/// Dosage forms offered in the form picker (free text is allowed too).
pub const FORMS: &[&str] = &[
    "Tablet",
    "Capsule",
    "Syrup",
    "Suspension",
    "Injection",
    "Infusion",
    "Drops",
    "Eye drops",
    "Ear drops",
    "Nasal spray",
    "Inhaler",
    "Cream",
    "Ointment",
    "Gel",
    "Suppository",
    "Powder",
    "Sachet",
    "Solution",
];

#[derive(Clone, Debug, Serialize)]
pub struct Medicine {
    pub id: u64,
    pub brand: String,
    pub generic: String,
    pub strength: Option<String>,
    pub form: Option<String>,
    pub company: Option<String>,
    pub price: Option<f64>,
    pub is_active: bool,
    /// A shared catalogue row (not the clinic's own): read-only here.
    pub shared: bool,
    /// How many prescription lines use it; a used medicine can only be deactivated.
    pub used: i64,
}

/// What the add / edit form sends.
#[derive(Clone, Debug, Deserialize)]
pub struct MedicineInput {
    pub brand: String,
    pub generic: String,
    pub strength: Option<String>,
    pub form: Option<String>,
    pub company: Option<String>,
    pub price: Option<f64>,
}

impl MedicineInput {
    pub fn validate(self) -> Result<Self, AppError> {
        let brand = self.brand.trim().to_string();
        let generic = self.generic.trim().to_string();
        if brand.is_empty() || generic.is_empty() {
            return Err(AppError::Validation(
                "brand and generic name are required".into(),
            ));
        }
        check_len(&brand, "brand", 150)?;
        check_len(&generic, "generic", 255)?;
        let price = match self.price {
            Some(p) if !p.is_finite() || !(0.0..10_000_000.0).contains(&p) => {
                return Err(AppError::Validation(
                    "price must be between 0 and 10000000".into(),
                ));
            }
            p => p.map(|x| (x * 100.0).round() / 100.0),
        };
        Ok(Self {
            brand,
            generic,
            strength: opt(self.strength, "strength", 100)?,
            form: opt(self.form, "form", 50)?,
            company: opt(self.company, "company", 150)?,
            price,
        })
    }
}

/// CSV text of catalogue rows, in the import format (brand, generic, strength, form,
/// company, price), so an export can be edited and imported back.
pub fn to_csv(rows: &[Medicine]) -> String {
    fn cell(s: &str) -> String {
        if s.contains([',', '"', '\n', '\r']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }
    let mut out = String::from("brand,generic,strength,form,company,price\n");
    for m in rows {
        let fields = [
            cell(&m.brand),
            cell(&m.generic),
            cell(m.strength.as_deref().unwrap_or_default()),
            cell(m.form.as_deref().unwrap_or_default()),
            cell(m.company.as_deref().unwrap_or_default()),
            m.price.map(|p| format!("{p}")).unwrap_or_default(),
        ];
        out.push_str(&fields.join(","));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(brand: &str, generic: &str, price: Option<f64>) -> MedicineInput {
        MedicineInput {
            brand: brand.into(),
            generic: generic.into(),
            strength: Some(" 500 mg ".into()),
            form: Some("".into()),
            company: None,
            price,
        }
    }

    #[test]
    fn validates_input() {
        let m = input(" Napa ", "Paracetamol", Some(1.234))
            .validate()
            .unwrap();
        assert_eq!(m.brand, "Napa");
        assert_eq!(m.strength.as_deref(), Some("500 mg"));
        assert_eq!(m.form, None);
        assert_eq!(m.price, Some(1.23));
        assert!(input("", "Paracetamol", None).validate().is_err());
        assert!(input("Napa", " ", None).validate().is_err());
        assert!(input("Napa", "Paracetamol", Some(-1.0)).validate().is_err());
    }

    #[test]
    fn exports_csv_that_imports_back() {
        let rows = vec![Medicine {
            id: 1,
            brand: "Ace \"Plus\"".into(),
            generic: "Paracetamol, Caffeine".into(),
            strength: Some("500 mg".into()),
            form: Some("Tablet".into()),
            company: None,
            price: Some(2.5),
            is_active: true,
            shared: false,
            used: 0,
        }];
        let csv = to_csv(&rows);
        let (parsed, problems) = crate::catalogue::parse(&csv).unwrap();
        assert!(problems.is_empty());
        assert_eq!(parsed[0].brand, "Ace \"Plus\"");
        assert_eq!(parsed[0].generic, "Paracetamol, Caffeine");
        assert_eq!(parsed[0].price, Some(2.5));
    }
}
