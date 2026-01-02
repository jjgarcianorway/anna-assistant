//! Utility functions for recipe handling (v0.0.427).

/// Get current timestamp as ISO 8601 string
pub(crate) fn now_iso8601() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
