//! Medical certificates, sick leave and referral letters for a visit (roadmap 2.9).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use vd_common::AppError;

use crate::text::{check_len, opt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    /// Unfit for work / school for a period.
    SickLeave,
    /// Fit to resume work / school.
    Fitness,
    /// Letter referring the patient to another doctor or hospital.
    Referral,
}

impl DocumentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SickLeave => "sick_leave",
            Self::Fitness => "fitness",
            Self::Referral => "referral",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [Self::SickLeave, Self::Fitness, Self::Referral]
            .into_iter()
            .find(|k| k.as_str() == s)
    }
}

/// What the doctor fills in; which fields apply depends on the kind.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DocumentBody {
    pub diagnosis: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    /// Referral: doctor / department / hospital referred to.
    pub referred_to: Option<String>,
    pub reason: Option<String>,
    /// Referral: clinical summary, findings and treatment so far.
    pub summary: Option<String>,
    pub remarks: Option<String>,
}

impl DocumentBody {
    pub fn validate(self, kind: DocumentKind) -> Result<Self, AppError> {
        let bad = |m: &str| Err(AppError::Validation(m.into()));
        let b = Self {
            diagnosis: opt(self.diagnosis, "diagnosis", 300)?,
            from: self.from,
            to: self.to,
            referred_to: opt(self.referred_to, "referred to", 200)?,
            reason: opt(self.reason, "reason", 500)?,
            summary: opt(self.summary, "summary", 3000)?,
            remarks: opt(self.remarks, "remarks", 1000)?,
        };
        match kind {
            DocumentKind::SickLeave => {
                let (Some(from), Some(to)) = (b.from, b.to) else {
                    return bad("give the rest period (from and to)");
                };
                if to < from {
                    return bad("the rest period ends before it starts");
                }
                if (to - from).num_days() > 365 {
                    return bad("the rest period is longer than a year");
                }
                if b.diagnosis.is_none() {
                    return bad("give the diagnosis");
                }
            }
            DocumentKind::Fitness => {
                if b.from.is_none() {
                    return bad("give the date the patient is fit from");
                }
            }
            DocumentKind::Referral => {
                if b.referred_to.is_none() || b.reason.is_none() {
                    return bad("give who the patient is referred to, and why");
                }
                if let Some(s) = &b.summary {
                    check_len(s, "summary", 3000)?;
                }
            }
        }
        Ok(b)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Document {
    pub id: u64,
    pub visit_id: u64,
    pub patient_id: u64,
    pub kind: DocumentKind,
    pub body: DocumentBody,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    }

    #[test]
    fn checks_fields_by_kind() {
        let ok = DocumentBody {
            diagnosis: Some("Enteric fever".into()),
            from: d("2026-10-12"),
            to: d("2026-10-18"),
            ..Default::default()
        };
        assert!(ok.clone().validate(DocumentKind::SickLeave).is_ok());
        let backwards = DocumentBody {
            from: d("2026-10-18"),
            to: d("2026-10-12"),
            ..ok.clone()
        };
        assert!(backwards.validate(DocumentKind::SickLeave).is_err());
        assert!(
            DocumentBody::default()
                .validate(DocumentKind::Referral)
                .is_err()
        );
        let referral = DocumentBody {
            referred_to: Some("Nephrology, DMCH".into()),
            reason: Some("Raised creatinine".into()),
            ..Default::default()
        };
        assert!(referral.validate(DocumentKind::Referral).is_ok());
    }
}
