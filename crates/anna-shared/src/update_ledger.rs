//! Update ledger for tracking update attempts.
//!
//! INVARIANT: There is exactly ONE update ledger at /var/lib/anna/update_ledger.json.
//! No per-user ledgers. No home directory paths.

use crate::paths::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Path to the update ledger file (system-wide)
fn ledger_path() -> PathBuf {
    paths().update_ledger_file()
}

/// A single update check entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckEntry {
    pub timestamp: DateTime<Utc>,
    pub current_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_tag: Option<String>,
    pub result: UpdateCheckResult,
    pub duration_ms: u64,
}

impl UpdateCheckEntry {
    pub fn new(current_version: &str, result: UpdateCheckResult, duration_ms: u64) -> Self {
        Self {
            timestamp: Utc::now(),
            current_version: current_version.to_string(),
            remote_tag: None,
            result,
            duration_ms,
        }
    }

    pub fn with_remote_tag(mut self, tag: String) -> Self {
        self.remote_tag = Some(tag);
        self
    }
}

/// Result of an update check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateCheckResult {
    UpToDate,
    UpdateAvailable { version: String },
    Installed { version: String },
    Failed { reason: String },
}

/// The update ledger
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateLedger {
    pub checks: Vec<UpdateCheckEntry>,
}

impl UpdateLedger {
    pub fn push(&mut self, entry: UpdateCheckEntry) {
        self.checks.push(entry);
        // Keep only last 100 entries
        if self.checks.len() > 100 {
            self.checks.remove(0);
        }
    }
}

/// Load the update ledger from disk
pub fn load_update_ledger() -> UpdateLedger {
    let path = ledger_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(ledger) = serde_json::from_str(&contents) {
                return ledger;
            }
        }
    }
    UpdateLedger::default()
}

/// Save the update ledger to disk
pub fn save_update_ledger(ledger: &UpdateLedger) -> anyhow::Result<()> {
    let path = ledger_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(ledger)?;
    fs::write(&path, contents)?;
    Ok(())
}
