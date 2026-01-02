//! Helper state tracking (v0.0.434).
//!
//! Tracks who installed helpers and their usage statistics.

use serde::{Deserialize, Serialize};

/// Who installed the helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelperInstalledBy {
    /// Anna installed this helper.
    Anna,
    /// User installed this helper.
    User,
}

impl HelperInstalledBy {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Anna => "anna",
            Self::User => "user",
        }
    }
}

/// State of a helper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperState {
    /// Helper ID.
    pub id: String,
    /// Who installed it.
    pub installed_by: HelperInstalledBy,
    /// When installed/detected.
    pub installed_at: String,
    /// Number of times used.
    pub use_count: u64,
    /// Last used timestamp.
    pub last_used: Option<String>,
}

impl HelperState {
    /// Create for Anna-installed helper.
    pub fn installed_by_anna(id: &str) -> Self {
        Self {
            id: id.to_string(),
            installed_by: HelperInstalledBy::Anna,
            installed_at: timestamp_now(),
            use_count: 0,
            last_used: None,
        }
    }

    /// Create for user-installed helper.
    pub fn detected_user(id: &str) -> Self {
        Self {
            id: id.to_string(),
            installed_by: HelperInstalledBy::User,
            installed_at: timestamp_now(),
            use_count: 0,
            last_used: None,
        }
    }

    /// Record usage.
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used = Some(timestamp_now());
    }
}

/// Get current timestamp.
pub(crate) fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_state_usage() {
        let mut state = HelperState::installed_by_anna("test");
        assert_eq!(state.use_count, 0);

        state.record_use();
        assert_eq!(state.use_count, 1);
        assert!(state.last_used.is_some());
    }
}
