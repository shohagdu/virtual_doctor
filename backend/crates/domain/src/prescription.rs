use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use vd_common::AppError;

use crate::text::{check_len, list, opt};

/// One ℞ line, e.g. "Tab. Napa 500 mg — 1+1+1 — খাবারের পরে — 5 দিন".
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MedicineLine {
    pub medicine_id: Option<u64>,
    pub brand: String,
    pub generic: Option<String>,
    pub strength: Option<String>,
    pub form: Option<String>,
    pub dose_pattern: Option<String>,
    pub timing: Option<String>,
    pub duration: Option<String>,
    pub notes: Option<String>,
}

impl MedicineLine {
    fn validate(self) -> Result<Option<Self>, AppError> {
        let brand = self.brand.trim().to_string();
        if brand.is_empty() {
            return Ok(None); // blank row left in the editor
        }
        check_len(&brand, "medicine brand", 150)?;
        Ok(Some(Self {
            medicine_id: self.medicine_id,
            brand,
            generic: opt(self.generic, "generic", 255)?,
            strength: opt(self.strength, "strength", 100)?,
            form: opt(self.form, "form", 50)?,
            dose_pattern: opt(self.dose_pattern, "dose", 30)?,
            timing: opt(self.timing, "timing", 100)?,
            duration: opt(self.duration, "duration", 50)?,
            notes: opt(self.notes, "notes", 255)?,
        }))
    }
}

/// The editable content of a prescription. Sections are lists because each one is a
/// ➕/✕ list on the screen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PrescriptionBody {
    #[serde(default)]
    pub complaints: Vec<String>,
    #[serde(default)]
    pub examination: Vec<String>,
    #[serde(default)]
    pub investigations: Vec<String>,
    #[serde(default)]
    pub diagnosis: Vec<String>,
    #[serde(default)]
    pub treatment_plan: Vec<String>,
    #[serde(default)]
    pub advice: Vec<String>,
    #[serde(default)]
    pub next_plan: Vec<String>,
    pub operation_notes: Option<String>,
    pub next_visit_date: Option<NaiveDate>,
    #[serde(default)]
    pub medicines: Vec<MedicineLine>,
}

impl PrescriptionBody {
    pub fn validate(self, today: NaiveDate) -> Result<Self, AppError> {
        if let Some(d) = self.next_visit_date
            && d < today
        {
            return Err(AppError::Validation(
                "next visit date is in the past".into(),
            ));
        }
        let mut medicines = Vec::new();
        for m in self.medicines {
            if let Some(m) = m.validate()? {
                medicines.push(m);
            }
        }
        if medicines.len() > 50 {
            return Err(AppError::Validation("at most 50 medicines".into()));
        }
        Ok(Self {
            complaints: list(self.complaints, "complaints", 100, 500)?,
            examination: list(self.examination, "examination", 100, 500)?,
            investigations: list(self.investigations, "investigations", 100, 500)?,
            diagnosis: list(self.diagnosis, "diagnosis", 100, 500)?,
            treatment_plan: list(self.treatment_plan, "treatment_plan", 100, 500)?,
            advice: list(self.advice, "advice", 100, 500)?,
            next_plan: list(self.next_plan, "next_plan", 100, 500)?,
            operation_notes: opt(self.operation_notes, "operation_notes", 10_000)?,
            next_visit_date: self.next_visit_date,
            medicines,
        })
    }
}

/// A stored prescription version (PM-11). `id` is `None` for a visit with no saved
/// prescription yet (an empty draft).
#[derive(Clone, Debug, Serialize)]
pub struct Prescription {
    pub id: Option<u64>,
    pub visit_id: u64,
    pub version: u32,
    pub revision: u32,
    pub locked: bool,
    #[serde(flatten)]
    pub body: PrescriptionBody,
    pub updated_at: Option<DateTime<Utc>>,
}

/// The version the client last loaded; used to detect concurrent edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseVersion {
    pub id: u64,
    pub revision: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SavePrescription {
    pub base: Option<BaseVersion>,
    #[serde(flatten)]
    pub body: PrescriptionBody,
}

/// What a save should do, given the latest stored version.
#[derive(Debug, PartialEq, Eq)]
pub enum SavePlan {
    /// Update the unlocked draft in place.
    Update { id: u64, revision: u32 },
    /// Insert a new version (first save, or the latest one is locked).
    Insert { version: u32 },
}

/// Decides how to save, rejecting the save if someone else changed the prescription
/// after the client loaded it (§13).
pub fn plan_save(
    latest: Option<(BaseVersion, u32, bool)>, // (id+revision, version, locked)
    base: Option<BaseVersion>,
) -> Result<SavePlan, AppError> {
    match (latest, base) {
        (None, None) => Ok(SavePlan::Insert { version: 1 }),
        (Some((current, version, locked)), Some(base)) if current == base => Ok(if locked {
            SavePlan::Insert {
                version: version + 1,
            }
        } else {
            SavePlan::Update {
                id: current.id,
                revision: current.revision + 1,
            }
        }),
        _ => Err(AppError::Conflict(
            "this prescription was changed by someone else — reload to see the latest version"
                .into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 12).unwrap()
    }

    #[test]
    fn drops_blank_medicine_rows() {
        let body = PrescriptionBody {
            medicines: vec![
                MedicineLine {
                    brand: " Napa ".into(),
                    dose_pattern: Some("1+1+1".into()),
                    ..Default::default()
                },
                MedicineLine {
                    brand: "  ".into(),
                    ..Default::default()
                },
            ],
            diagnosis: vec!["UTI".into(), "".into()],
            ..Default::default()
        }
        .validate(today())
        .unwrap();
        assert_eq!(body.medicines.len(), 1);
        assert_eq!(body.medicines[0].brand, "Napa");
        assert_eq!(body.diagnosis, vec!["UTI"]);
    }

    #[test]
    fn rejects_past_next_visit() {
        let body = PrescriptionBody {
            next_visit_date: NaiveDate::from_ymd_opt(2026, 10, 1),
            ..Default::default()
        };
        assert!(body.validate(today()).is_err());
    }

    #[test]
    fn save_plans() {
        let b = BaseVersion { id: 5, revision: 2 };
        assert_eq!(
            plan_save(None, None).unwrap(),
            SavePlan::Insert { version: 1 }
        );
        assert_eq!(
            plan_save(Some((b, 1, false)), Some(b)).unwrap(),
            SavePlan::Update { id: 5, revision: 3 }
        );
        assert_eq!(
            plan_save(Some((b, 1, true)), Some(b)).unwrap(),
            SavePlan::Insert { version: 2 }
        );
        // Stale client: someone saved revision 3 in between
        assert!(
            plan_save(
                Some((BaseVersion { id: 5, revision: 3 }, 1, false)),
                Some(b)
            )
            .is_err()
        );
        // Client thinks there is nothing yet, but there is
        assert!(plan_save(Some((b, 1, false)), None).is_err());
        assert!(plan_save(None, Some(b)).is_err());
    }
}
