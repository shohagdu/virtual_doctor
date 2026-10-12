use serde::{Deserialize, Serialize};
use vd_common::AppError;

use crate::text::{list, opt};

/// Persistent patient data carried into every new prescription (PM-08, §5.4 left panel).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MedicalProfile {
    #[serde(default)]
    pub allergies: Vec<String>,
    #[serde(default)]
    pub chronic_conditions: Vec<String>,
    #[serde(default)]
    pub long_term_medicines: Vec<String>,
    #[serde(default)]
    pub vaccination: Vec<String>,
    pub past_history: Option<String>,
    pub family_history: Option<String>,
    pub social_history: Option<String>,
    pub gynae_obs_history: Option<String>,
    pub pregnancy_status: Option<String>,
}

impl MedicalProfile {
    pub fn validate(self) -> Result<Self, AppError> {
        Ok(Self {
            allergies: list(self.allergies, "allergies", 50, 150)?,
            chronic_conditions: list(self.chronic_conditions, "chronic_conditions", 50, 150)?,
            long_term_medicines: list(self.long_term_medicines, "long_term_medicines", 50, 200)?,
            vaccination: list(self.vaccination, "vaccination", 100, 150)?,
            past_history: opt(self.past_history, "past_history", 5000)?,
            family_history: opt(self.family_history, "family_history", 5000)?,
            social_history: opt(self.social_history, "social_history", 5000)?,
            gynae_obs_history: opt(self.gynae_obs_history, "gynae_obs_history", 5000)?,
            pregnancy_status: opt(self.pregnancy_status, "pregnancy_status", 30)?,
        })
    }
}

/// Combines two patients' profiles when duplicates are merged (PM-19): lists are
/// joined without repeats (case-insensitive), and free-text histories are kept from
/// both when both have text.
pub fn merge(keep: MedicalProfile, other: MedicalProfile) -> MedicalProfile {
    fn join(mut a: Vec<String>, b: Vec<String>) -> Vec<String> {
        for x in b {
            if !a.iter().any(|y| y.eq_ignore_ascii_case(&x)) {
                a.push(x);
            }
        }
        a
    }
    fn text(a: Option<String>, b: Option<String>) -> Option<String> {
        match (a, b) {
            (Some(a), Some(b)) if a.trim() == b.trim() => Some(a),
            (Some(a), Some(b)) => Some(format!("{a}\n{b}")),
            (a, b) => a.or(b),
        }
    }
    MedicalProfile {
        allergies: join(keep.allergies, other.allergies),
        chronic_conditions: join(keep.chronic_conditions, other.chronic_conditions),
        long_term_medicines: join(keep.long_term_medicines, other.long_term_medicines),
        vaccination: join(keep.vaccination, other.vaccination),
        past_history: text(keep.past_history, other.past_history),
        family_history: text(keep.family_history, other.family_history),
        social_history: text(keep.social_history, other.social_history),
        gynae_obs_history: text(keep.gynae_obs_history, other.gynae_obs_history),
        pregnancy_status: keep.pregnancy_status.or(other.pregnancy_status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_lists() {
        let p = MedicalProfile {
            allergies: vec![
                " Penicillin ".into(),
                "".into(),
                "Penicillin".into(),
                "Sulfa".into(),
            ],
            past_history: Some("  ".into()),
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(p.allergies, vec!["Penicillin", "Sulfa"]);
        assert_eq!(p.past_history, None);
    }

    #[test]
    fn rejects_long_items() {
        let p = MedicalProfile {
            allergies: vec!["x".repeat(151)],
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn merges_profiles() {
        let keep = MedicalProfile {
            allergies: vec!["Penicillin".into()],
            past_history: Some("Typhoid 2019".into()),
            ..Default::default()
        };
        let other = MedicalProfile {
            allergies: vec!["penicillin".into(), "Sulfa".into()],
            chronic_conditions: vec!["Diabetes".into()],
            past_history: Some("Appendectomy".into()),
            family_history: Some("HTN".into()),
            ..Default::default()
        };
        let m = merge(keep, other);
        assert_eq!(m.allergies, vec!["Penicillin", "Sulfa"]);
        assert_eq!(m.chronic_conditions, vec!["Diabetes"]);
        assert_eq!(
            m.past_history.as_deref(),
            Some("Typhoid 2019\nAppendectomy")
        );
        assert_eq!(m.family_history.as_deref(), Some("HTN"));
    }
}
