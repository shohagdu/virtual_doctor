use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use vd_common::AppError;

/// Vitals recorded at a visit (PM-13). Units: BP mmHg, pulse /min, temperature °F,
/// weight kg, height cm, SpO₂ %, RBS mmol/L. BMI is calculated, never entered.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vitals {
    pub bp_sys: Option<u16>,
    pub bp_dia: Option<u16>,
    pub pulse: Option<u16>,
    pub temp: Option<f64>,
    pub weight: Option<f64>,
    pub height: Option<f64>,
    #[serde(default)]
    pub bmi: Option<f64>,
    pub spo2: Option<u8>,
    pub rbs: Option<f64>,
}

/// One stored vitals record with the visit it belongs to (trend charts, §5.4 Vitals tab).
#[derive(Clone, Debug, Serialize)]
pub struct VitalsRecord {
    pub visit_id: u64,
    pub visit_date: NaiveDate,
    #[serde(flatten)]
    pub vitals: Vitals,
    pub recorded_at: DateTime<Utc>,
}

fn range<T: PartialOrd + Copy + std::fmt::Display>(
    v: Option<T>,
    field: &str,
    min: T,
    max: T,
) -> Result<Option<T>, AppError> {
    match v {
        Some(x) if x < min || x > max => Err(AppError::Validation(format!(
            "{field}: must be between {min} and {max}"
        ))),
        _ => Ok(v),
    }
}

fn round(v: Option<f64>, places: i32) -> Option<f64> {
    let f = 10f64.powi(places);
    v.map(|x| (x * f).round() / f)
}

impl Vitals {
    /// Checks plausible ranges (catches typos such as 1200 for 120) and calculates BMI.
    pub fn validate(self) -> Result<Self, AppError> {
        let bp_sys = range(self.bp_sys, "BP systolic", 50, 300)?;
        let bp_dia = range(self.bp_dia, "BP diastolic", 20, 200)?;
        match (bp_sys, bp_dia) {
            (Some(s), Some(d)) if d >= s => {
                return Err(AppError::Validation(
                    "BP diastolic must be lower than systolic".into(),
                ));
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(AppError::Validation(
                    "enter both BP systolic and diastolic".into(),
                ));
            }
            _ => {}
        }
        let weight = round(range(self.weight, "weight (kg)", 0.3, 400.0)?, 2);
        let height = round(range(self.height, "height (cm)", 20.0, 250.0)?, 1);
        let bmi = match (weight, height) {
            (Some(w), Some(h)) => round(Some(w / (h / 100.0).powi(2)), 1),
            _ => None,
        };
        let v = Self {
            bp_sys,
            bp_dia,
            pulse: range(self.pulse, "pulse", 20, 250)?,
            temp: round(range(self.temp, "temperature (°F)", 90.0, 110.0)?, 1),
            weight,
            height,
            bmi,
            spo2: range(self.spo2, "SpO₂", 50, 100)?,
            rbs: round(range(self.rbs, "RBS (mmol/L)", 1.0, 50.0)?, 1),
        };
        if v == Self::default() {
            return Err(AppError::Validation("enter at least one vital sign".into()));
        }
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_bmi() {
        let v = Vitals {
            weight: Some(60.0),
            height: Some(160.0),
            bmi: Some(99.0), // client value is ignored
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(v.bmi, Some(23.4));
    }

    #[test]
    fn rejects_implausible_values() {
        let typo = Vitals {
            bp_sys: Some(1200),
            bp_dia: Some(80),
            ..Default::default()
        };
        assert!(typo.validate().is_err());
        let swapped = Vitals {
            bp_sys: Some(80),
            bp_dia: Some(120),
            ..Default::default()
        };
        assert!(swapped.validate().is_err());
        let half_bp = Vitals {
            bp_sys: Some(120),
            ..Default::default()
        };
        assert!(half_bp.validate().is_err());
        assert!(Vitals::default().validate().is_err());
    }

    #[test]
    fn rounds_decimals() {
        let v = Vitals {
            temp: Some(100.46),
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(v.temp, Some(100.5));
    }
}
