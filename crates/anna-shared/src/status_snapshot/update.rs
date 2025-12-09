//! Update check types (v0.0.211).

use serde::{Deserialize, Serialize};

/// Update check result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdateResult {
    /// Up to date
    UpToDate,
    /// Update available
    UpdateAvailable { version: String },
    /// Update downloaded
    Downloaded { version: String },
    /// Update installed
    Installed { version: String },
    /// Check failed
    Failed { reason: String },
    /// Not checked yet
    #[default]
    NotChecked,
}

/// Update subsystem information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Check interval in seconds
    pub interval_s: u64,
    /// Last check timestamp (epoch seconds)
    pub last_check_ts: Option<u64>,
    /// Next check timestamp (epoch seconds)
    pub next_check_ts: Option<u64>,
    /// Last check result
    pub last_result: UpdateResult,
}

impl UpdateInfo {
    pub fn new(interval_s: u64) -> Self {
        Self {
            interval_s,
            last_check_ts: None,
            next_check_ts: None,
            last_result: UpdateResult::NotChecked,
        }
    }
}
