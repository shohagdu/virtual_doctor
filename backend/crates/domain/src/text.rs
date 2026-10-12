//! Input clean-up helpers shared by the validators.

use vd_common::AppError;

/// Trims an optional string; empty becomes `None`. Errors if longer than `max` chars.
pub fn opt(v: Option<String>, field: &str, max: usize) -> Result<Option<String>, AppError> {
    let v = v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    if let Some(s) = &v {
        check_len(s, field, max)?;
    }
    Ok(v)
}

/// Trims every item, drops empty ones and exact duplicates, and enforces limits.
pub fn list(
    items: Vec<String>,
    field: &str,
    max_items: usize,
    max_len: usize,
) -> Result<Vec<String>, AppError> {
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim().to_string();
        if item.is_empty() || out.contains(&item) {
            continue;
        }
        check_len(&item, field, max_len)?;
        out.push(item);
    }
    if out.len() > max_items {
        return Err(AppError::Validation(format!(
            "{field}: at most {max_items} items"
        )));
    }
    Ok(out)
}

pub fn check_len(s: &str, field: &str, max: usize) -> Result<(), AppError> {
    if s.chars().count() > max {
        return Err(AppError::Validation(format!(
            "{field}: at most {max} characters"
        )));
    }
    Ok(())
}
