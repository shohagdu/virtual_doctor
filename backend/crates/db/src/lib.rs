//! MySQL access (sqlx). Queries are runtime-checked so the crate builds without a live database.

pub mod ai;
pub mod appointments;
pub mod audit;
pub mod bills;
pub mod clinics;
pub mod documents;
pub mod favorites;
pub mod followups;
pub mod medical_profile;
pub mod medicines;
pub mod merge;
pub mod patients;
pub mod prescriptions;
pub mod queue;
pub mod reports;
pub mod settings;
pub mod sms;
pub mod templates;
pub mod users;
pub mod visits;
pub mod vitals;

pub use sqlx::MySqlPool;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::str::FromStr;
use std::time::Duration;

pub type DbResult<T> = Result<T, sqlx::Error>;

/// Connects with session time zone UTC (all DATETIME columns are UTC).
pub async fn connect(url: &str) -> DbResult<MySqlPool> {
    let opts = MySqlConnectOptions::from_str(url)?
        .charset("utf8mb4")
        .timezone(Some("+00:00".into()));
    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(opts)
        .await
}

pub async fn migrate(pool: &MySqlPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}
