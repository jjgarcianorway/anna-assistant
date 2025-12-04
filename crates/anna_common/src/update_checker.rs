//! Update Checker v7.34.0 - Real Update Checking Implementation
//!
//! Performs actual update checks:
//! - Anna releases from GitHub API
//!
//! v7.34.0: Uses config::UpdateState for consistent state management
//! All results stored to /var/lib/anna/internal/update_state.json
//! Audit trail written to /var/lib/anna/internal/ops.log

use crate::config::{UpdateMode, UpdateState};
use crate::ops_log::OpsLog;

/// Result of an update check operation
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    /// No updates available
    UpToDate,
    /// Update available with version
    UpdateAvailable { version: String },
    /// Check failed with error
    Error { message: String },
}

/// Check for Anna updates from GitHub
pub fn check_anna_updates(current_version: &str) -> CheckResult {
    // Use GitHub API to check latest release with timeout
    let output = std::process::Command::new("curl")
        .args([
            "-sS",
            "--max-time",
            "10",
            "-H",
            "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/jjgarcianorway/anna-assistant/releases/latest",
        ])
        .output();

    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return CheckResult::Error {
                    message: format!("curl failed: {}", stderr.trim()),
                };
            }

            let stdout = String::from_utf8_lossy(&result.stdout);
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => {
                    if let Some(tag) = json.get("tag_name").and_then(|v| v.as_str()) {
                        let latest = tag.trim_start_matches('v');
                        if is_newer_version(latest, current_version) {
                            CheckResult::UpdateAvailable {
                                version: latest.to_string(),
                            }
                        } else {
                            CheckResult::UpToDate
                        }
                    } else {
                        CheckResult::Error {
                            message: "No tag_name in response".to_string(),
                        }
                    }
                }
                Err(e) => CheckResult::Error {
                    message: format!("JSON parse error: {}", e),
                },
            }
        }
        Err(e) => CheckResult::Error {
            message: format!("Network error: {}", e),
        },
    }
}

/// Run update check and record result to state (v7.34.0)
/// This is the main entry point for the update scheduler
pub fn run_update_check(current_version: &str) -> CheckResult {
    // Log check start
    let mut ops_log = OpsLog::open();
    ops_log.log("update_check", "started", None);

    let result = check_anna_updates(current_version);

    // Load current state
    let mut state = UpdateState::load();

    // Record result based on outcome
    match &result {
        CheckResult::UpToDate => {
            state.record_success(current_version, Some(current_version.to_string()));
            ops_log.log("update_check", "finished", Some("up_to_date"));
        }
        CheckResult::UpdateAvailable { version } => {
            state.record_success(current_version, Some(version.clone()));
            ops_log.log(
                "update_check",
                "finished",
                Some(&format!("update_available:{}", version)),
            );
        }
        CheckResult::Error { message } => {
            state.record_failure(current_version, message);
            ops_log.log(
                "update_check",
                "finished",
                Some(&format!("error:{}", message)),
            );
        }
    }

    // Save state immediately
    if let Err(e) = state.save() {
        ops_log.log("update_check", "error", Some(&format!("save_failed:{}", e)));
    }

    result
}

/// Check if an update check is due (delegates to UpdateState)
pub fn is_check_due(state: &UpdateState) -> bool {
    state.is_check_due()
}

/// Compare semantic versions (returns true if latest > current)
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts
            .get(2)
            .and_then(|s| s.split('-').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        (major, minor, patch)
    };

    let (lmaj, lmin, lpat) = parse_version(latest);
    let (cmaj, cmin, cpat) = parse_version(current);

    (lmaj, lmin, lpat) > (cmaj, cmin, cpat)
}

/// Check if daemon is running
pub fn is_daemon_running() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "annad"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("7.34.0", "7.33.0"));
        assert!(is_newer_version("7.33.1", "7.33.0"));
        assert!(is_newer_version("8.0.0", "7.99.99"));
        assert!(!is_newer_version("7.33.0", "7.33.0"));
        assert!(!is_newer_version("7.32.0", "7.33.0"));
    }

    #[test]
    fn test_check_due_never_checked() {
        let state = UpdateState::default();
        // Default state with mode=Auto and last_check_at=0 should be due
        assert!(state.is_check_due());
    }

    #[test]
    fn test_check_due_manual_mode() {
        let mut state = UpdateState::default();
        state.mode = UpdateMode::Manual;
        // Manual mode should never be due
        assert!(!state.is_check_due());
    }
}
