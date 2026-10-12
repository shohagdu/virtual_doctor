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
}
