use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

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
    pub medicines: Vec<String>,
    pub next_visit_date: Option<NaiveDate>,
}
