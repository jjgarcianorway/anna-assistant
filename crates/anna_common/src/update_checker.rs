//! Update Checker v7.33.0 - Real Update Checking Implementation
//!
//! Performs actual update checks:
//! - Anna releases from GitHub API
//! - Pacman package updates via checkupdates
//!
//! All results stored to update_state.json for truthful reporting.

use crate::update_state::{UpdateState, UpdateResult, UpdateTarget, UpdateMode};

/// Check for Anna updates from GitHub
pub fn check_anna_updates(current_version: &str) -> UpdateResult {
    // Use GitHub API to check latest release
    let output = std::process::Command::new("curl")
        .args([
            "-sS",
            "--max-time", "10",
            "-H", "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/jjgarcianorway/anna-assistant/releases/latest"
        ])
        .output();

    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return UpdateResult::Failed {
                    error: format!("curl failed: {}", stderr.trim()),
                };
            }

            let stdout = String::from_utf8_lossy(&result.stdout);
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => {
                    if let Some(tag) = json.get("tag_name").and_then(|v| v.as_str()) {
                        let latest = tag.trim_start_matches('v');
                        if is_newer_version(latest, current_version) {
                            UpdateResult::UpdatesAvailable { count: 1 }
                        } else {
                            UpdateResult::NoUpdates
                        }
                    } else {
                        UpdateResult::Failed {
                            error: "No tag_name in response".to_string(),
                        }
                    }
                }
                Err(e) => UpdateResult::Failed {
                    error: format!("JSON parse error: {}", e),
                },
            }
        }
        Err(e) => UpdateResult::Failed {
            error: format!("Network error: {}", e),
        },
    }
}

/// Check for pacman package updates
pub fn check_pacman_updates() -> UpdateResult {
    // Use checkupdates if available (from pacman-contrib)
    let output = std::process::Command::new("checkupdates")
        .output();

    match output {
        Ok(result) => {
            // checkupdates exits 0 if updates, 1 if no updates, 2 if error
            match result.status.code() {
                Some(0) => {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let count = stdout.lines().count() as u32;
                    if count > 0 {
                        UpdateResult::UpdatesAvailable { count }
                    } else {
                        UpdateResult::NoUpdates
                    }
                }
                Some(1) => UpdateResult::NoUpdates,
                Some(2) => {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    UpdateResult::Failed {
                        error: stderr.trim().to_string(),
                    }
                }
                _ => UpdateResult::Failed {
                    error: "checkupdates exited abnormally".to_string(),
                },
            }
        }
        Err(_) => {
            // checkupdates not available, try pacman directly
            let pacman_output = std::process::Command::new("pacman")
                .args(["-Qu"])
                .output();

            match pacman_output {
                Ok(result) => {
                    if result.status.success() {
                        let stdout = String::from_utf8_lossy(&result.stdout);
                        let count = stdout.lines().filter(|l| !l.is_empty()).count() as u32;
                        if count > 0 {
                            UpdateResult::UpdatesAvailable { count }
                        } else {
                            UpdateResult::NoUpdates
                        }
                    } else {
                        UpdateResult::NoUpdates
                    }
                }
                Err(e) => UpdateResult::Failed {
                    error: format!("pacman not available: {}", e),
                },
            }
        }
    }
}

/// Run update check based on configured target
pub fn run_update_check(target: &UpdateTarget, current_version: &str) -> UpdateResult {
    match target {
        UpdateTarget::AnnaRelease => check_anna_updates(current_version),
        UpdateTarget::PacmanPackages => check_pacman_updates(),
        UpdateTarget::Both => {
            let anna_result = check_anna_updates(current_version);
            let pacman_result = check_pacman_updates();

            // Combine results
            match (&anna_result, &pacman_result) {
                (UpdateResult::UpdatesAvailable { count: a }, UpdateResult::UpdatesAvailable { count: b }) => {
                    UpdateResult::UpdatesAvailable { count: a + b }
                }
                (UpdateResult::UpdatesAvailable { count }, _) |
                (_, UpdateResult::UpdatesAvailable { count }) => {
                    UpdateResult::UpdatesAvailable { count: *count }
                }
                (UpdateResult::Failed { error: e1 }, UpdateResult::Failed { error: e2 }) => {
                    UpdateResult::Failed {
                        error: format!("{}, {}", e1, e2),
                    }
                }
                (UpdateResult::Failed { error }, _) | (_, UpdateResult::Failed { error }) => {
                    UpdateResult::Failed { error: error.clone() }
                }
                _ => UpdateResult::NoUpdates,
            }
        }
    }
}

/// Compare semantic versions (returns true if latest > current)
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.split('-').next()).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };

    let (lmaj, lmin, lpat) = parse_version(latest);
    let (cmaj, cmin, cpat) = parse_version(current);

    (lmaj, lmin, lpat) > (cmaj, cmin, cpat)
}

/// Check if it's time for the next scheduled check
pub fn is_check_due(state: &UpdateState) -> bool {
    if state.mode != UpdateMode::Auto {
        return false;
    }

    match state.next_check_epoch {
        Some(next) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now >= next
        }
        None => true, // No scheduled check, do it now
    }
}

/// Perform scheduled update check if due
pub fn maybe_run_scheduled_check(current_version: &str) -> Option<UpdateResult> {
    let mut state = UpdateState::load();

    if !is_check_due(&state) {
        return None;
    }

    let result = run_update_check(&state.target, current_version);
    state.record_check(result.clone());
    let _ = state.save();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("7.33.0", "7.32.0"));
        assert!(is_newer_version("7.32.1", "7.32.0"));
        assert!(is_newer_version("8.0.0", "7.99.99"));
        assert!(!is_newer_version("7.32.0", "7.32.0"));
        assert!(!is_newer_version("7.31.0", "7.32.0"));
    }

    #[test]
    fn test_check_due() {
        let mut state = UpdateState::default();
        state.mode = UpdateMode::Auto;
        state.next_check_epoch = None;
        assert!(is_check_due(&state));

        state.mode = UpdateMode::Manual;
        assert!(!is_check_due(&state));
    }
}
