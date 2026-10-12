//! Shared building blocks: configuration, errors, auth primitives.

pub mod auth;
pub mod config;
pub mod error;

pub use config::Config;
pub use error::{AppError, AppResult};
