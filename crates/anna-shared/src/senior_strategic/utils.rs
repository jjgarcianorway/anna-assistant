//! Utility functions for strategic analysis.

/// Calculate days between two timestamps
pub fn days_between(start: u64, end: u64) -> u32 {
    ((end.saturating_sub(start)) / 86400) as u32
}

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    format!(
        "SESS-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

/// Generate a unique insight ID
pub fn generate_insight_id() -> String {
    format!(
        "INS-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

/// Get current timestamp (unix epoch seconds)
pub fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
