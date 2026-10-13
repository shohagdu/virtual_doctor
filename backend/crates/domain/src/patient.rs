use chrono::{DateTime, Months, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use vd_common::AppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    M,
    F,
    O,
}

impl Sex {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::M => "M",
            Self::F => "F",
            Self::O => "O",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "M" => Some(Self::M),
            "F" => Some(Self::F),
            "O" => Some(Self::O),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Patient {
    pub id: u64,
    pub patient_code: String,
    pub title: Option<String>,
    pub name_en: String,
    pub name_bn: Option<String>,
    pub phone: String,
    pub dob: NaiveDate,
    pub dob_estimated: bool,
    pub sex: Sex,
    pub blood_group: Option<String>,
    pub address: Option<String>,
    pub district: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Registration form as sent by the client (Patient_Management_Spec §5.2).
#[derive(Clone, Debug, Deserialize)]
pub struct NewPatientInput {
    pub title: Option<String>,
    pub name_en: String,
    pub name_bn: Option<String>,
    pub phone: String,
    pub dob: Option<String>,
    pub age_years: Option<u32>,
    pub sex: Sex,
    pub blood_group: Option<String>,
    pub address: Option<String>,
    pub district: Option<String>,
}

/// Validated registration data, ready to store.
#[derive(Clone, Debug)]
pub struct NewPatient {
    pub title: Option<String>,
    pub name_en: String,
    pub name_bn: Option<String>,
    pub phone: String,
    pub dob: NaiveDate,
    pub dob_estimated: bool,
    pub sex: Sex,
    pub blood_group: Option<String>,
    pub address: Option<String>,
    pub district: Option<String>,
}

fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn bounded(v: Option<String>, field: &str, max: usize) -> Result<Option<String>, AppError> {
    crate::text::opt(clean(v), field, max)
}

/// Bangladesh mobile format: 01[3-9]XXXXXXXX. Accepts +880 / 880 prefixes.
pub fn normalize_phone(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let local = digits.strip_prefix("88").unwrap_or(&digits);
    let b = local.as_bytes();
    (b.len() == 11 && b[0] == b'0' && b[1] == b'1' && (b'3'..=b'9').contains(&b[2]))
        .then(|| local.to_string())
}

impl NewPatientInput {
    pub fn validate(self, today: NaiveDate) -> Result<NewPatient, AppError> {
        let name_en = self.name_en.trim().to_string();
        if name_en.chars().count() < 2 {
            return Err(AppError::Validation("name is required".into()));
        }
        crate::text::check_len(&name_en, "name", 150)?;
        let phone = normalize_phone(&self.phone)
            .ok_or_else(|| AppError::Validation("phone must be in 01XXXXXXXXX format".into()))?;

        // Age only → store an estimated DOB so age stays correct over time (PM-04).
        let (dob, dob_estimated) = match (clean(self.dob), self.age_years) {
            (Some(d), _) => {
                let dob = NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                    .map_err(|_| AppError::Validation("dob must be YYYY-MM-DD".into()))?;
                if dob > today {
                    return Err(AppError::Validation("dob is in the future".into()));
                }
                (dob, false)
            }
            (None, Some(age)) if age <= 130 => (
                today
                    .checked_sub_months(Months::new(age * 12))
                    .ok_or_else(|| AppError::Validation("invalid age".into()))?,
                true,
            ),
            _ => return Err(AppError::Validation("age or dob is required".into())),
        };

        Ok(NewPatient {
            title: bounded(self.title, "title", 10)?,
            name_en,
            name_bn: bounded(self.name_bn, "name_bn", 150)?,
            phone,
            dob,
            dob_estimated,
            sex: self.sex,
            blood_group: bounded(self.blood_group, "blood_group", 5)?,
            address: bounded(self.address, "address", 255)?,
            district: bounded(self.district, "district", 100)?,
        })
    }
}

/// Public patient ID, e.g. `P-2026-000123`.
pub fn patient_code(year: i32, seq: u64) -> String {
    format!("P-{year}-{seq:06}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> NewPatientInput {
        NewPatientInput {
            title: Some("Ms".into()),
            name_en: " Maisha ".into(),
            name_bn: Some("".into()),
            phone: "+8801712345678".into(),
            dob: None,
            age_years: Some(18),
            sex: Sex::F,
            blood_group: None,
            address: None,
            district: None,
        }
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 12).unwrap()
    }

    #[test]
    fn age_only_gives_estimated_dob() {
        let p = input().validate(today()).unwrap();
        assert_eq!(p.dob, NaiveDate::from_ymd_opt(2008, 10, 12).unwrap());
        assert!(p.dob_estimated);
        assert_eq!(p.name_en, "Maisha");
        assert_eq!(p.name_bn, None);
        assert_eq!(p.phone, "01712345678");
    }

    #[test]
    fn dob_wins_over_age() {
        let mut i = input();
        i.dob = Some("2000-01-31".into());
        let p = i.validate(today()).unwrap();
        assert_eq!(p.dob, NaiveDate::from_ymd_opt(2000, 1, 31).unwrap());
        assert!(!p.dob_estimated);
    }

    #[test]
    fn rejects_bad_input() {
        let mut i = input();
        i.phone = "0123".into();
        assert!(i.validate(today()).is_err());

        let mut i = input();
        i.age_years = None;
        assert!(i.validate(today()).is_err());

        let mut i = input();
        i.dob = Some("2030-01-01".into());
        assert!(i.validate(today()).is_err());
    }

    #[test]
    fn phone_formats() {
        assert_eq!(
            normalize_phone("01912-345678").as_deref(),
            Some("01912345678")
        );
        assert_eq!(normalize_phone("01212345678"), None);
        assert_eq!(
            normalize_phone("8801312345678").as_deref(),
            Some("01312345678")
        );
    }

    #[test]
    fn code_format() {
        assert_eq!(patient_code(2026, 123), "P-2026-000123");
    }
}

/// A row of the patient list: the patient plus a little history (Patients page).
#[derive(Clone, Debug, Serialize)]
pub struct PatientListItem {
    #[serde(flatten)]
    pub patient: Patient,
    pub visit_count: i64,
    pub last_visit: Option<NaiveDate>,
    pub has_allergy: bool,
}

/// Patient list order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatientSort {
    /// Newest registered first.
    #[default]
    Newest,
    Name,
    /// Most recently seen first; never seen last.
    LastVisit,
}
