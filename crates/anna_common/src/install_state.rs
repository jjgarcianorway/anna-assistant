//! Install State for Anna v0.0.10
//!
//! Tracks installation state for accurate uninstall and drift detection:
//! - Binary install paths discovered at install time
//! - Unit file paths
//! - Data/config paths
//! - Helper provenance records
//! - Last installer review run + results
//!
//! Written by annad, read by annactl for reset/uninstall operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Path to the install state file
pub const INSTALL_STATE_PATH: &str = "/var/lib/anna/install_state.json";

/// Current schema version
pub const INSTALL_STATE_SCHEMA: u32 = 1;

/// Binary info with path and optional checksum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    /// Path to the binary
    pub path: PathBuf,
    /// SHA256 checksum (if computed)
    pub checksum: Option<String>,
    /// Version string
    pub version: Option<String>,
    /// Last verified timestamp
    pub last_verified: Option<DateTime<Utc>>,
}

/// Systemd unit info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitInfo {
    /// Path to the unit file
    pub path: PathBuf,
    /// Expected ExecStart value
    pub exec_start: Option<String>,
    /// Whether unit should be enabled
    pub should_be_enabled: bool,
    /// Last verified state
    pub last_state: Option<String>,
}

/// Directory info for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    /// Path to directory
    pub path: PathBuf,
    /// Expected owner (uid)
    pub expected_owner: Option<u32>,
    /// Expected group (gid)
    pub expected_group: Option<u32>,
    /// Expected permissions (octal)
    pub expected_perms: Option<u32>,
    /// Whether this is an Anna-internal directory
    pub anna_internal: bool,
}

/// Installer review result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewResult {
    /// All checks passed
    Healthy,
    /// Issues found but auto-repaired
    Repaired { fixes: Vec<String> },
    /// Issues found that need manual attention
    NeedsAttention { issues: Vec<String> },
    /// Review failed to complete
    Failed { reason: String },
}

impl std::fmt::Display for ReviewResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewResult::Healthy => write!(f, "healthy"),
            ReviewResult::Repaired { fixes } => write!(f, "repaired ({} fixes)", fixes.len()),
            ReviewResult::NeedsAttention { issues } => {
                write!(f, "needs attention ({} issues)", issues.len())
            }
            ReviewResult::Failed { reason } => write!(f, "failed: {}", reason),
        }
    }
}

/// Last installer review record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastReview {
    /// When the review ran
    pub timestamp: DateTime<Utc>,
    /// Result of the review
    pub result: ReviewResult,
    /// Evidence IDs for any fixes
    pub evidence_ids: Vec<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Complete install state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallState {
    /// Schema version
    pub schema_version: u32,
    /// When this state was created
    pub created_at: DateTime<Utc>,
    /// When this state was last updated
    pub updated_at: DateTime<Utc>,

    // Binary tracking
    /// annactl binary info
    pub annactl: Option<BinaryInfo>,
    /// annad binary info
    pub annad: Option<BinaryInfo>,

    // Systemd tracking
    /// annad.service unit info
    pub annad_unit: Option<UnitInfo>,

    // Directory tracking
    /// Data directories
    pub data_dirs: Vec<DirectoryInfo>,
    /// Config file path
    pub config_path: Option<PathBuf>,

    // Helper tracking (references helpers.json)
    /// Path to helpers manifest
    pub helpers_manifest_path: Option<PathBuf>,

    // Review tracking
    /// Last installer review
    pub last_review: Option<LastReview>,
    /// Review history (last N reviews)
    pub review_history: Vec<LastReview>,
}

impl Default for InstallState {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallState {
    /// Create new install state with defaults
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            schema_version: INSTALL_STATE_SCHEMA,
            created_at: now,
            updated_at: now,
            annactl: None,
            annad: None,
            annad_unit: None,
            data_dirs: Self::default_data_dirs(),
            config_path: Some(PathBuf::from("/etc/anna/config.toml")),
            helpers_manifest_path: Some(PathBuf::from(crate::helpers::HELPERS_STATE_FILE)),
            last_review: None,
            review_history: Vec::new(),
        }
    }

    /// Default data directories
    fn default_data_dirs() -> Vec<DirectoryInfo> {
        vec![
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna/internal"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna/internal/snapshots"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna/rollback"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna/rollback/files"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna/rollback/logs"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna/knowledge"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
            DirectoryInfo {
                path: PathBuf::from("/var/lib/anna/telemetry"),
                expected_owner: Some(0),
                expected_group: Some(0),
                expected_perms: Some(0o755),
                anna_internal: true,
            },
        ]
    }

    /// Load install state from disk
    pub fn load() -> Option<Self> {
        let path = Path::new(INSTALL_STATE_PATH);
        if !path.exists() {
            return None;
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }

    /// Load or create default install state
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    /// Ensure install state exists on disk - v0.0.25: called on daemon start
    /// Creates default state if missing, returns true if created
    pub fn ensure_initialized() -> std::io::Result<bool> {
        let path = Path::new(INSTALL_STATE_PATH);
        if path.exists() {
            return Ok(false);
        }

        // Create default state with discovered binary paths
        let mut state = Self::default();

        // Discover binary paths
        if let Ok(annactl_path) = which("annactl") {
            state.annactl = Some(BinaryInfo {
                path: annactl_path,
                checksum: None,
                version: get_binary_version("annactl"),
                last_verified: Some(Utc::now()),
            });
        }
        if let Ok(annad_path) = which("annad") {
            state.annad = Some(BinaryInfo {
                path: annad_path,
                checksum: None,
                version: get_binary_version("annad"),
                last_verified: Some(Utc::now()),
            });
        }

        // Discover unit file path
        let unit_path = PathBuf::from("/etc/systemd/system/annad.service");
        if unit_path.exists() {
            state.annad_unit = Some(UnitInfo {
                path: unit_path,
                exec_start: Some("/usr/local/bin/annad".to_string()),
                should_be_enabled: true,
                last_state: Some("active".to_string()),
            });
        }

        state.save()?;
        Ok(true)
    }

    /// Save install state to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Path::new(INSTALL_STATE_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(INSTALL_STATE_PATH, &content)
    }

    /// Update binary info
    pub fn set_binary_info(&mut self, name: &str, info: BinaryInfo) {
        match name {
            "annactl" => self.annactl = Some(info),
            "annad" => self.annad = Some(info),
            _ => {}
        }
        self.updated_at = Utc::now();
    }

    /// Update unit info
    pub fn set_unit_info(&mut self, info: UnitInfo) {
        self.annad_unit = Some(info);
        self.updated_at = Utc::now();
    }

    /// Record a review result
    pub fn record_review(&mut self, review: LastReview) {
        // Add to history (keep last 10)
        if let Some(prev) = self.last_review.take() {
            self.review_history.insert(0, prev);
            self.review_history.truncate(10);
        }
        self.last_review = Some(review);
        self.updated_at = Utc::now();
    }

    /// Get all paths that should be removed on uninstall
    pub fn get_uninstall_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Data directories (in reverse order for proper removal)
        for dir in self.data_dirs.iter().rev() {
            if dir.anna_internal {
                paths.push(dir.path.clone());
            }
        }

        // Config directory
        paths.push(PathBuf::from("/etc/anna"));

        // Unit file
        if let Some(ref unit) = self.annad_unit {
            paths.push(unit.path.clone());
        }

        // Binaries (if we know their paths)
        if let Some(ref bin) = self.annactl {
            paths.push(bin.path.clone());
        }
        if let Some(ref bin) = self.annad {
            paths.push(bin.path.clone());
        }

        paths
    }

    /// Check if a path is safe to remove (part of Anna installation)
    pub fn is_safe_to_remove(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Always safe: Anna data directories
        if path_str.starts_with("/var/lib/anna") {
            return true;
        }

        // Always safe: Anna config directory
        if path_str.starts_with("/etc/anna") {
            return true;
        }

        // Safe if it's a tracked binary
        if let Some(ref bin) = self.annactl {
            if bin.path == path {
                return true;
            }
        }
        if let Some(ref bin) = self.annad {
            if bin.path == path {
                return true;
            }
        }

        // Safe if it's the tracked unit file
        if let Some(ref unit) = self.annad_unit {
            if unit.path == path {
                return true;
            }
        }

        false
    }
}

/// Discover current installation state
pub fn discover_install_state() -> InstallState {
    let mut state = InstallState::new();

    // Discover annactl binary
    if let Ok(path) = std::env::current_exe() {
        state.annactl = Some(BinaryInfo {
            path,
            checksum: None,
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            last_verified: Some(Utc::now()),
        });
    }

    // Discover annad binary (common locations)
    for path in &["/usr/bin/annad", "/usr/local/bin/annad"] {
        let p = Path::new(path);
        if p.exists() {
            state.annad = Some(BinaryInfo {
                path: p.to_path_buf(),
                checksum: None,
                version: None,
                last_verified: Some(Utc::now()),
            });
            break;
        }
    }

    // Discover systemd unit
    let unit_path = Path::new("/etc/systemd/system/annad.service");
    if unit_path.exists() {
        state.annad_unit = Some(UnitInfo {
            path: unit_path.to_path_buf(),
            exec_start: None,
            should_be_enabled: true,
            last_state: None,
        });
    }

    state
}

/// Find a binary in PATH - v0.0.25
fn which(name: &str) -> std::io::Result<PathBuf> {
    let output = std::process::Command::new("which").arg(name).output()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(path))
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{} not found in PATH", name),
        ))
    }
}

/// Get version from a binary - v0.0.25
fn get_binary_version(name: &str) -> Option<String> {
    let output = std::process::Command::new(name)
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Extract version from "name vX.Y.Z" or just "vX.Y.Z"
        for word in stdout.split_whitespace() {
            if word.starts_with('v')
                || word
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                return Some(word.trim_start_matches('v').to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_state_default() {
        let state = InstallState::new();
        assert_eq!(state.schema_version, INSTALL_STATE_SCHEMA);
        assert!(!state.data_dirs.is_empty());
        assert!(state.config_path.is_some());
    }

    #[test]
    fn test_default_data_dirs() {
        let dirs = InstallState::default_data_dirs();
        assert!(dirs
            .iter()
            .any(|d| d.path == PathBuf::from("/var/lib/anna")));
        assert!(dirs.iter().all(|d| d.anna_internal));
    }

    #[test]
    fn test_is_safe_to_remove() {
        let state = InstallState::new();

        // Anna paths are safe
        assert!(state.is_safe_to_remove(Path::new("/var/lib/anna")));
        assert!(state.is_safe_to_remove(Path::new("/var/lib/anna/internal")));
        assert!(state.is_safe_to_remove(Path::new("/etc/anna/config.toml")));

        // System paths are not safe
        assert!(!state.is_safe_to_remove(Path::new("/etc/passwd")));
        assert!(!state.is_safe_to_remove(Path::new("/var/lib/pacman")));
        assert!(!state.is_safe_to_remove(Path::new("/usr/bin/bash")));
    }

    #[test]
    fn test_review_result_display() {
        assert_eq!(format!("{}", ReviewResult::Healthy), "healthy");
        assert_eq!(
            format!(
                "{}",
                ReviewResult::Repaired {
                    fixes: vec!["fix1".into()]
                }
            ),
            "repaired (1 fixes)"
        );
        assert_eq!(
            format!(
                "{}",
                ReviewResult::NeedsAttention {
                    issues: vec!["issue1".into(), "issue2".into()]
                }
            ),
            "needs attention (2 issues)"
        );
    }

    #[test]
    fn test_record_review() {
        let mut state = InstallState::new();

        let review1 = LastReview {
            timestamp: Utc::now(),
            result: ReviewResult::Healthy,
            evidence_ids: vec![],
            duration_ms: 100,
        };

        state.record_review(review1);
        assert!(state.last_review.is_some());
        assert!(state.review_history.is_empty());

        let review2 = LastReview {
            timestamp: Utc::now(),
            result: ReviewResult::Repaired {
                fixes: vec!["test".into()],
            },
            evidence_ids: vec!["E1".into()],
            duration_ms: 200,
        };

        state.record_review(review2);
        assert!(state.last_review.is_some());
        assert_eq!(state.review_history.len(), 1);
    }
}
