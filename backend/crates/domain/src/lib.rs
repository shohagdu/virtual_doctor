//! Business types and rules, independent of HTTP and the database.

pub mod allergy;
pub mod appointment;
pub mod billing;
pub mod followup;
pub mod lab;
pub mod medical_profile;
pub mod patient;
pub mod prescription;
pub mod report;
pub mod settings;
pub mod sms;
pub mod text;
pub mod visit;
pub mod vitals;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// Today's date in Bangladesh (Asia/Dhaka is UTC+6 with no daylight saving).
pub fn clinic_today() -> NaiveDate {
    (Utc::now() + CLINIC_OFFSET).date_naive()
}

/// Asia/Dhaka offset from UTC.
const CLINIC_OFFSET: Duration = Duration::hours(6);

/// A clinic-local date and time as UTC (DATETIME columns are stored in UTC).
pub fn clinic_to_utc(date: NaiveDate, time: NaiveTime) -> DateTime<Utc> {
    (date.and_time(time) - CLINIC_OFFSET).and_utc()
}

/// A UTC instant as clinic-local date and time.
pub fn utc_to_clinic(at: DateTime<Utc>) -> NaiveDateTime {
    (at + CLINIC_OFFSET).naive_utc()
}
