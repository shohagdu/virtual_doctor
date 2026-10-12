//! Business types and rules, independent of HTTP and the database.

pub mod medical_profile;
pub mod patient;
pub mod prescription;
pub mod text;
pub mod visit;

use chrono::{Duration, NaiveDate, Utc};

/// Today's date in Bangladesh (Asia/Dhaka is UTC+6 with no daylight saving).
pub fn clinic_today() -> NaiveDate {
    (Utc::now() + Duration::hours(6)).date_naive()
}
