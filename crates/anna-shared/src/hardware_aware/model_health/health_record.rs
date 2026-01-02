//! Health record for individual models (v0.0.434).
//!
//! Tracks installation status, verification results, and usage statistics for a model.

use super::model_status::{timestamp_now, InstalledBy, ModelStatus};
use serde::{Deserialize, Serialize};

/// Health record for a single model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealthRecord {
    /// Model name.
    pub name: String,
    /// Current status.
    pub status: ModelStatus,
    /// Who installed it.
    pub installed_by: InstalledBy,
    /// When first detected/installed.
    pub installed_at: Option<String>,
    /// Last verification time.
    pub last_verified: Option<String>,
    /// Last verification result message.
    pub last_verify_message: Option<String>,
    /// Number of successful uses.
    pub use_count: u64,
    /// Last error message (if broken).
    pub last_error: Option<String>,
}

impl ModelHealthRecord {
    /// Create a new record for a missing model.
    pub fn missing(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ModelStatus::Missing,
            installed_by: InstalledBy::Unknown,
            installed_at: None,
            last_verified: None,
            last_verify_message: None,
            use_count: 0,
            last_error: None,
        }
    }

    /// Create a new record for a model installed by Anna.
    pub fn installed_by_anna(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ModelStatus::Unverified,
            installed_by: InstalledBy::Anna,
            installed_at: Some(timestamp_now()),
            last_verified: None,
            last_verify_message: None,
            use_count: 0,
            last_error: None,
        }
    }

    /// Create a new record for a pre-existing model.
    pub fn detected(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ModelStatus::Unverified,
            installed_by: InstalledBy::User,
            installed_at: Some(timestamp_now()),
            last_verified: None,
            last_verify_message: None,
            use_count: 0,
            last_error: None,
        }
    }

    /// Mark as verified OK.
    pub fn mark_ok(&mut self, message: &str) {
        self.status = ModelStatus::Ok;
        self.last_verified = Some(timestamp_now());
        self.last_verify_message = Some(message.to_string());
        self.last_error = None;
    }

    /// Mark as broken.
    pub fn mark_broken(&mut self, error: &str) {
        self.status = ModelStatus::Broken;
        self.last_verified = Some(timestamp_now());
        self.last_error = Some(error.to_string());
    }

    /// Record a successful use.
    pub fn record_use(&mut self) {
        self.use_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_ok_and_broken() {
        let mut record = ModelHealthRecord::missing("test_model");

        record.mark_ok("Test passed");
        assert_eq!(record.status, ModelStatus::Ok);
        assert!(record.last_verified.is_some());

        record.mark_broken("Test failed");
        assert_eq!(record.status, ModelStatus::Broken);
        assert!(record.last_error.is_some());
    }
}
