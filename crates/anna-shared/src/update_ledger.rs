//! Update ledger for tracking auto-update checks.
//!
//! Persists last N update checks for auditability and status display.
//! Storage: ~/.anna/update_ledger.json
//!
//! v0.0.29: Initial implementation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum entries to keep in ledger
const MAX_ENTRIES: usize = 20;

/// Result of an update check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UpdateCheckResult {
    /// Already up to date
    UpToDate,
    /// Update available
    UpdateAvailable { version: String },
    /// Update downloaded
    Downloaded { version: String },
    /// Update installed
    Installed { version: String },
    /// Check failed
    Failed { reason: String },
}

impl std::fmt::Display for UpdateCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UpToDate => write!(f, "up_to_date"),
            Self::UpdateAvailable { version } => write!(f, "update_available({})", version),
            Self::Downloaded { version } => write!(f, "downloaded({})", version),
            Self::Installed { version } => write!(f, "installed({})", version),
            Self::Failed { reason } => write!(f, "failed: {}", reason),
        }
    }
}

/// Single update check entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckEntry {
    /// When the check was performed (epoch seconds)
    pub checked_at_ts: u64,
    /// Current local version at time of check
    pub local_version: String,
    /// Remote tag found (if any)
    pub remote_tag: Option<String>,
    /// Result of the check
    pub result: UpdateCheckResult,
    /// Duration of the check in milliseconds
    pub duration_ms: u64,
}

impl UpdateCheckEntry {
    /// Create a new entry with current timestamp
    pub fn new(local_version: impl Into<String>, result: UpdateCheckResult, duration_ms: u64) -> Self {
        let checked_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            checked_at_ts,
            local_version: local_version.into(),
            remote_tag: None,
            result,
            duration_ms,
        }
    }

    /// Set remote tag
    pub fn with_remote_tag(mut self, tag: impl Into<String>) -> Self {
        self.remote_tag = Some(tag.into());
        self
    }
}

/// Update ledger containing history of update checks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateLedger {
    /// Entries in chronological order (oldest first)
    pub entries: Vec<UpdateCheckEntry>,
}

impl UpdateLedger {
    /// Create a new empty ledger
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add an entry, maintaining max size
    pub fn push(&mut self, entry: UpdateCheckEntry) {
        self.entries.push(entry);
        while self.entries.len() > MAX_ENTRIES {
            self.entries.remove(0);
        }
    }

    /// Get the most recent entry
    pub fn last(&self) -> Option<&UpdateCheckEntry> {
        self.entries.last()
    }

    /// Get the last N entries (most recent first)
    pub fn last_n(&self, n: usize) -> Vec<&UpdateCheckEntry> {
        self.entries.iter().rev().take(n).collect()
    }

    /// Get count of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Count successful checks
    pub fn success_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| !matches!(e.result, UpdateCheckResult::Failed { .. }))
            .count()
    }

    /// Count failed checks
    pub fn failure_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.result, UpdateCheckResult::Failed { .. }))
            .count()
    }

    /// Get last successful check
    pub fn last_success(&self) -> Option<&UpdateCheckEntry> {
        self.entries
            .iter()
            .rev()
            .find(|e| !matches!(e.result, UpdateCheckResult::Failed { .. }))
    }

    /// Get last remote tag found
    pub fn last_remote_tag(&self) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find_map(|e| e.remote_tag.as_deref())
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Get path to update ledger file
pub fn update_ledger_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".anna").join("update_ledger.json")
}

/// Load update ledger from disk
pub fn load_update_ledger() -> UpdateLedger {
    let path = update_ledger_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(ledger) = serde_json::from_str(&content) {
                return ledger;
            }
        }
    }
    UpdateLedger::new()
}

/// Save update ledger to disk
pub fn save_update_ledger(ledger: &UpdateLedger) -> std::io::Result<()> {
    let path = update_ledger_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(ledger)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, content)
}

/// Clear update ledger (for reset)
pub fn clear_update_ledger() -> std::io::Result<()> {
    let path = update_ledger_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_check_entry_new() {
        let entry = UpdateCheckEntry::new("0.0.29", UpdateCheckResult::UpToDate, 150);
        assert_eq!(entry.local_version, "0.0.29");
        assert_eq!(entry.duration_ms, 150);
        assert!(entry.checked_at_ts > 0);
    }

    #[test]
    fn test_update_ledger_push() {
        let mut ledger = UpdateLedger::new();
        assert!(ledger.is_empty());

        ledger.push(UpdateCheckEntry::new("0.0.28", UpdateCheckResult::UpToDate, 100));
        assert_eq!(ledger.len(), 1);

        ledger.push(UpdateCheckEntry::new(
            "0.0.28",
            UpdateCheckResult::UpdateAvailable {
                version: "0.0.29".to_string(),
            },
            120,
        ));
        assert_eq!(ledger.len(), 2);
    }

    #[test]
    fn test_update_ledger_max_entries() {
        let mut ledger = UpdateLedger::new();

        // Add more than MAX_ENTRIES
        for i in 0..25 {
            ledger.push(UpdateCheckEntry::new(
                format!("0.0.{}", i),
                UpdateCheckResult::UpToDate,
                100,
            ));
        }

        assert_eq!(ledger.len(), MAX_ENTRIES);
        // Oldest entries should be removed
        assert_eq!(ledger.entries[0].local_version, "0.0.5");
    }

    #[test]
    fn test_update_ledger_last() {
        let mut ledger = UpdateLedger::new();
        ledger.push(UpdateCheckEntry::new("0.0.28", UpdateCheckResult::UpToDate, 100));
        ledger.push(UpdateCheckEntry::new(
            "0.0.29",
            UpdateCheckResult::UpdateAvailable {
                version: "0.0.30".to_string(),
            },
            120,
        ));

        let last = ledger.last().unwrap();
        assert_eq!(last.local_version, "0.0.29");
    }

    #[test]
    fn test_update_ledger_counts() {
        let mut ledger = UpdateLedger::new();
        ledger.push(UpdateCheckEntry::new("0.0.28", UpdateCheckResult::UpToDate, 100));
        ledger.push(UpdateCheckEntry::new(
            "0.0.28",
            UpdateCheckResult::Failed {
                reason: "network error".to_string(),
            },
            50,
        ));
        ledger.push(UpdateCheckEntry::new("0.0.28", UpdateCheckResult::UpToDate, 100));

        assert_eq!(ledger.success_count(), 2);
        assert_eq!(ledger.failure_count(), 1);
    }

    #[test]
    fn test_update_check_result_display() {
        assert_eq!(UpdateCheckResult::UpToDate.to_string(), "up_to_date");
        assert_eq!(
            UpdateCheckResult::UpdateAvailable {
                version: "0.0.30".to_string()
            }
            .to_string(),
            "update_available(0.0.30)"
        );
    }

    #[test]
    fn test_update_ledger_serialization() {
        let mut ledger = UpdateLedger::new();
        ledger.push(
            UpdateCheckEntry::new("0.0.29", UpdateCheckResult::UpToDate, 100)
                .with_remote_tag("v0.0.29"),
        );

        let json = serde_json::to_string(&ledger).unwrap();
        let parsed: UpdateLedger = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.last().unwrap().remote_tag, Some("v0.0.29".to_string()));
    }
}
