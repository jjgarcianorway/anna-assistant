//! Update Checker v7.43.0 - Find Highest Semantic Version
//!
//! Performs actual update checks:
//! - Anna releases from GitHub API
//!
//! v7.43.0: Fetch ALL releases and find highest semantic version
//! - The /releases/latest endpoint returns most recently CREATED release
//! - This caused bugs when backfilling old releases
//! - Now fetches all releases and picks highest version using semver comparison
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
/// v7.43.0: Fetches ALL releases and finds highest semantic version
pub fn check_anna_updates(current_version: &str) -> CheckResult {
    // v7.43.0: Fetch ALL releases, not just /releases/latest
    // The /latest endpoint returns the most recently CREATED release,
    // not the highest version. This caused bugs when backfilling releases.
    let output = std::process::Command::new("curl")
        .args([
            "-sS",
            "--max-time",
            "10",
            "-H",
            "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/jjgarcianorway/anna-assistant/releases?per_page=100",
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
                    // v7.43.0: Parse array of releases and find highest version
                    if let Some(releases) = json.as_array() {
                        let mut highest: Option<(u32, u32, u32, String)> = None;

                        for release in releases {
                            if let Some(tag) = release.get("tag_name").and_then(|v| v.as_str()) {
                                let version_str = tag.trim_start_matches('v');
                                if let Some((major, minor, patch)) = parse_version(version_str) {
                                    let dominated = highest
                                        .as_ref()
                                        .map(|(hm, hn, hp, _)| {
                                            (major, minor, patch) > (*hm, *hn, *hp)
                                        })
                                        .unwrap_or(true);
                                    if dominated {
                                        highest =
                                            Some((major, minor, patch, version_str.to_string()));
                                    }
                                }
                            }
                        }

                        if let Some((_, _, _, latest)) = highest {
                            if is_newer_version(&latest, current_version) {
                                CheckResult::UpdateAvailable { version: latest }
                            } else {
                                CheckResult::UpToDate
                            }
                        } else {
                            CheckResult::Error {
                                message: "No valid releases found".to_string(),
                            }
                        }
                    } else {
                        CheckResult::Error {
                            message: "Response is not an array of releases".to_string(),
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

/// Parse version string into (major, minor, patch) tuple
fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        // Handle patch with potential suffix like "75-beta"
        let patch_str = parts[2].split('-').next().unwrap_or(parts[2]);
        let patch = patch_str.parse().ok()?;
        Some((major, minor, patch))
    } else {
        None
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
    let l = parse_version(latest).unwrap_or((0, 0, 0));
    let c = parse_version(current).unwrap_or((0, 0, 0));
    l > c
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
