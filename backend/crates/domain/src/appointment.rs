//! Appointments (Patient_Management_Spec §5.6, PM-15).

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

use crate::visit::{QueuePatient, VisitType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppointmentStatus {
    Booked,
    Arrived,
    Seen,
    NoShow,
    Cancelled,
}

impl AppointmentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Booked => "booked",
            Self::Arrived => "arrived",
            Self::Seen => "seen",
            Self::NoShow => "no_show",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "booked" => Some(Self::Booked),
            "arrived" => Some(Self::Arrived),
            "seen" => Some(Self::Seen),
            "no_show" => Some(Self::NoShow),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// Booked → Arrived (moves to the queue) → Seen, or No-show / Cancelled. A no-show
    /// or cancelled booking can be restored. Seen follows the visit, never set by hand.
    pub fn can_become(self, to: Self) -> bool {
        use AppointmentStatus::*;
        matches!(
            (self, to),
            (Booked, Arrived | NoShow | Cancelled) | (NoShow | Cancelled, Booked)
        )
    }

    /// Statuses that hold a slot.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Booked | Self::Arrived | Self::Seen)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Appointment {
    pub id: u64,
    pub start_at: DateTime<Utc>,
    /// Clinic-local date and time of `start_at`.
    pub date: NaiveDate,
    #[serde(with = "hhmm")]
    pub time: NaiveTime,
    pub visit_type: VisitType,
    pub status: AppointmentStatus,
    pub notes: Option<String>,
    pub visit_id: Option<u64>,
    pub sms_sent_at: Option<DateTime<Utc>>,
    pub patient: QueuePatient,
}

/// One bookable time on a day (§5.6 "time slots based on the chamber schedule").
#[derive(Clone, Debug, Serialize)]
pub struct Slot {
    #[serde(with = "hhmm")]
    pub time: NaiveTime,
    pub taken: bool,
    /// The slot has already started (today only).
    pub past: bool,
}

pub(crate) mod hhmm {
    use chrono::NaiveTime;
    use serde::Serializer;

    pub fn serialize<S: Serializer>(t: &NaiveTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&t.format("%H:%M").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AppointmentStatus::*;

    #[test]
    fn transitions() {
        assert!(Booked.can_become(Arrived));
        assert!(Booked.can_become(NoShow));
        assert!(Cancelled.can_become(Booked));
        assert!(!Arrived.can_become(Booked));
        assert!(!Booked.can_become(Seen));
        assert!(!Seen.can_become(Cancelled));
        assert!(Seen.is_active() && !NoShow.is_active());
    }
}
