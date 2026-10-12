//! Follow-up management (Patient_Management_Spec §5.7, PM-16).

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::visit::QueuePatient;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FollowupStatus {
    Pending,
    Came,
    Missed,
}

impl FollowupStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Came => "came",
            Self::Missed => "missed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "came" => Some(Self::Came),
            "missed" => Some(Self::Missed),
            _ => None,
        }
    }
}

/// The tabs of the "Follow-ups Due" list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bucket {
    #[default]
    Today,
    Overdue,
    /// Due today or in the next six days.
    Week,
    /// Marked came or missed, due in the last 30 days.
    Done,
}

/// Which follow-ups a bucket holds: a due-date range (inclusive) and whether it
/// lists pending ones or closed ones.
pub struct BucketFilter {
    pub from: Option<NaiveDate>,
    pub to: NaiveDate,
    pub pending: bool,
}

impl Bucket {
    pub fn filter(self, today: NaiveDate) -> BucketFilter {
        let day = Duration::days(1);
        match self {
            Self::Today => BucketFilter {
                from: Some(today),
                to: today,
                pending: true,
            },
            Self::Overdue => BucketFilter {
                from: None,
                to: today - day,
                pending: true,
            },
            Self::Week => BucketFilter {
                from: Some(today),
                to: today + day * 6,
                pending: true,
            },
            Self::Done => BucketFilter {
                from: Some(today - day * 30),
                to: today + day * 30,
                pending: false,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Followup {
    pub id: u64,
    pub due_date: NaiveDate,
    pub status: FollowupStatus,
    pub reminder_sent_at: Option<DateTime<Utc>>,
    /// The visit whose prescription set this Next Visit date.
    pub visit_id: Option<u64>,
    pub visit_date: Option<NaiveDate>,
    /// Diagnosis of that prescription; `None` for roles with limited history access.
    pub diagnosis: Option<Vec<String>>,
    pub chronic_conditions: Vec<String>,
    pub patient: QueuePatient,
    pub last_sms: Option<SmsState>,
}

/// Delivery state of the latest reminder SMS (§13 "show failed status").
#[derive(Clone, Debug, Serialize)]
pub struct SmsState {
    pub id: u64,
    pub status: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct BucketCounts {
    pub today: i64,
    pub overdue: i64,
    pub week: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_ranges() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        let w = Bucket::Week.filter(today);
        assert_eq!(w.from, Some(today));
        assert_eq!(w.to, NaiveDate::from_ymd_opt(2026, 10, 18).unwrap());
        let o = Bucket::Overdue.filter(today);
        assert_eq!(o.from, None);
        assert_eq!(o.to, NaiveDate::from_ymd_opt(2026, 10, 11).unwrap());
        assert!(!Bucket::Done.filter(today).pending);
    }
}
