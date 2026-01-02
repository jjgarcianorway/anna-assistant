//! Utility functions for alarm management.

/// Generate a unique alarm ID
pub fn generate_alarm_id() -> String {
    format!(
        "ALM-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

/// Get current timestamp in seconds since Unix epoch
pub fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
