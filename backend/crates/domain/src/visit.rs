use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::patient::Sex;
use crate::prescription::MedicineLine;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisitType {
    #[default]
    New,
    Followup,
    Report,
}

impl VisitType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Followup => "followup",
            Self::Report => "report",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "new" => Some(Self::New),
            "followup" => Some(Self::Followup),
            "report" => Some(Self::Report),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisitStatus {
    Waiting,
    InConsultation,
    Seen,
    Cancelled,
}

impl VisitStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::InConsultation => "in_consultation",
            Self::Seen => "seen",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "waiting" => Some(Self::Waiting),
            "in_consultation" => Some(Self::InConsultation),
            "seen" => Some(Self::Seen),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// Whether the queue may move from `self` to `to` (§5.1: Waiting → In Consultation →
    /// Seen, plus cancel / restore and reopening a seen visit).
    pub fn can_become(self, to: Self) -> bool {
        use VisitStatus::*;
        matches!(
            (self, to),
            (Waiting, InConsultation | Seen | Cancelled)
                | (InConsultation, Waiting | Seen | Cancelled)
                | (Seen, InConsultation)
                | (Cancelled, Waiting)
        ) || self == to
    }

    /// Statuses only a doctor sets: they mean the doctor is seeing or has seen the patient.
    pub fn doctor_only(self) -> bool {
        matches!(self, Self::InConsultation | Self::Seen)
    }
}

/// Moves `id` to index `position` (clamped) in the queue order. `None` if `id` is not
/// in the queue.
pub fn reorder(mut order: Vec<u64>, id: u64, position: usize) -> Option<Vec<u64>> {
    let from = order.iter().position(|&x| x == id)?;
    order.remove(from);
    order.insert(position.min(order.len()), id);
    Some(order)
}

#[derive(Clone, Debug, Serialize)]
pub struct Visit {
    pub id: u64,
    pub patient_id: u64,
    pub visit_no: u32,
    pub visit_date: NaiveDate,
    pub visit_type: VisitType,
    pub status: VisitStatus,
    pub queue_serial: Option<u32>,
    pub doctor_id: Option<u64>,
    pub created_at: DateTime<Utc>,
}

/// One entry of the patient's visit timeline (§5.4 Visits tab).
/// Clinical fields are `None` for roles with limited history access.
#[derive(Clone, Debug, Serialize)]
pub struct VisitSummary {
    #[serde(flatten)]
    pub visit: Visit,
    pub prescription: Option<PrescriptionSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PrescriptionSummary {
    pub version: u32,
    pub locked: bool,
    pub complaints: Vec<String>,
    pub diagnosis: Vec<String>,
    pub medicines: Vec<MedicineLine>,
    pub next_visit_date: Option<NaiveDate>,
}

/// A row of today's queue (§5.1).
#[derive(Clone, Debug, Serialize)]
pub struct QueueEntry {
    #[serde(flatten)]
    pub visit: Visit,
    pub patient: QueuePatient,
    pub has_allergy: bool,
    pub has_vitals: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueuePatient {
    pub id: u64,
    pub patient_code: String,
    pub title: Option<String>,
    pub name_en: String,
    pub name_bn: Option<String>,
    pub phone: String,
    pub dob: NaiveDate,
    pub sex: Sex,
}

/// Dashboard counter cards (§5.1).
#[derive(Clone, Debug, Default, Serialize)]
pub struct QueueCounters {
    pub today: i64,
    pub waiting: i64,
    pub in_consultation: i64,
    pub seen: i64,
    pub followups_due: i64,
}

#[cfg(test)]
mod tests {
    use super::VisitStatus::*;
    use super::*;

    #[test]
    fn transitions() {
        assert!(Waiting.can_become(InConsultation));
        assert!(InConsultation.can_become(Seen));
        assert!(Seen.can_become(InConsultation));
        assert!(Cancelled.can_become(Waiting));
        assert!(!Seen.can_become(Waiting));
        assert!(!Seen.can_become(Cancelled));
        assert!(!Cancelled.can_become(Seen));
    }

    #[test]
    fn reorders() {
        assert_eq!(reorder(vec![1, 2, 3, 4], 4, 0), Some(vec![4, 1, 2, 3]));
        assert_eq!(reorder(vec![1, 2, 3, 4], 1, 2), Some(vec![2, 3, 1, 4]));
        assert_eq!(reorder(vec![1, 2, 3], 2, 99), Some(vec![1, 3, 2]));
        assert_eq!(reorder(vec![1, 2, 3], 9, 0), None);
    }
}
