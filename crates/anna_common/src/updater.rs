//! Auto-update system for Anna v0.4.0
//!
//! Checks GitHub releases and performs self-updates.
//! v0.4.0: Dev auto-update every 10 minutes when enabled.

use crate::{UpdateChannel, UpdateConfig, UpdateResult, UpdateState, VersionInfo};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";
const GITHUB_API: &str = "https://api.github.com";

/// Default paths for state and config
pub const STATE_DIR: &str = "/var/lib/anna";
pub const CONFIG_DIR: &str = "/etc/anna";
pub const USER_CONFIG_DIR: &str = ".config/anna";

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
    pub html_url: String,
}

/// GitHub release asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Update check result
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub info: VersionInfo,
    pub annad_url: Option<String>,
    pub annactl_url: Option<String>,
    pub checksums_url: Option<String>,
}

impl UpdateCheckResult {
    /// Create from GitHub release
    pub fn from_release(current: &str, release: &GitHubRelease) -> Self {
        let latest = release.tag_name.trim_start_matches('v').to_string();
        let update_available = Self::version_newer(&latest, current);

        let arch = std::env::consts::ARCH;
        let target = match arch {
            "x86_64" => "x86_64-unknown-linux-gnu",
            "aarch64" => "aarch64-unknown-linux-gnu",
            _ => "x86_64-unknown-linux-gnu",
        };

        let annad_name = format!("annad-{}-{}", latest, target);
        let annactl_name = format!("annactl-{}-{}", latest, target);

        let annad_url = release
            .assets
            .iter()
            .find(|a| a.name == annad_name)
            .map(|a| a.browser_download_url.clone());

        let annactl_url = release
            .assets
            .iter()
            .find(|a| a.name == annactl_name)
            .map(|a| a.browser_download_url.clone());

        let checksums_url = release
            .assets
            .iter()
            .find(|a| a.name == "SHA256SUMS")
            .map(|a| a.browser_download_url.clone());

        Self {
            info: VersionInfo {
                current: current.to_string(),
                latest,
                update_available,
                release_notes: release.body.clone(),
                download_url: Some(release.html_url.clone()),
            },
            annad_url,
            annactl_url,
            checksums_url,
        }
    }

    /// Compare semantic versions (handles pre-release tags)
    fn version_newer(latest: &str, current: &str) -> bool {
        let parse = |v: &str| -> (u32, u32, u32, bool) {
            let parts: Vec<&str> = v.split('.').collect();
            let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let (patch, is_prerelease) = parts.get(2).map(|s| {
                if s.contains('-') {
                    let p = s.split('-').next().and_then(|p| p.parse().ok()).unwrap_or(0);
                    (p, true)
                } else {
                    (s.parse().ok().unwrap_or(0), false)
                }
            }).unwrap_or((0, false));
            (major, minor, patch, is_prerelease)
        };

        let (lmaj, lmin, lpat, l_pre) = parse(latest);
        let (cmaj, cmin, cpat, c_pre) = parse(current);

        // Compare versions
        match (lmaj, lmin, lpat).cmp(&(cmaj, cmin, cpat)) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                // Same version: release is newer than prerelease
                c_pre && !l_pre
            }
        }
    }
}

/// Get the releases API URL
pub fn releases_url() -> String {
    format!("{}/repos/{}/releases", GITHUB_API, GITHUB_REPO)
}

/// Get the latest release API URL
pub fn latest_release_url(channel: UpdateChannel) -> String {
    match channel {
        UpdateChannel::Stable => {
            format!("{}/repos/{}/releases/latest", GITHUB_API, GITHUB_REPO)
        }
        UpdateChannel::Beta | UpdateChannel::Dev => {
            // For beta/dev, we get all releases and find the latest pre-release
            format!("{}/repos/{}/releases", GITHUB_API, GITHUB_REPO)
        }
    }
}

/// Get the state file path
pub fn state_file_path() -> PathBuf {
    PathBuf::from(STATE_DIR).join("update_state.json")
}

/// Get the config file path (system-wide)
pub fn config_file_path() -> PathBuf {
    PathBuf::from(CONFIG_DIR).join("config.toml")
}

/// Get user config file path
pub fn user_config_file_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(USER_CONFIG_DIR).join("config.toml"))
}

/// Load update state from disk
pub fn load_update_state() -> UpdateState {
    let path = state_file_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        UpdateState::default()
    }
}

/// Save update state to disk
pub fn save_update_state(state: &UpdateState) -> std::io::Result<()> {
    let path = state_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)
}

/// Load update config (checks user config first, then system config)
pub fn load_update_config() -> UpdateConfig {
    // Try user config first
    if let Some(user_path) = user_config_file_path() {
        if user_path.exists() {
            if let Ok(content) = fs::read_to_string(&user_path) {
                if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                    if let Some(update) = config.get("update") {
                        if let Ok(update_config) = update.clone().try_into() {
                            return update_config;
                        }
                    }
                }
            }
        }
    }

    // Try system config
    let system_path = config_file_path();
    if system_path.exists() {
        if let Ok(content) = fs::read_to_string(&system_path) {
            if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                if let Some(update) = config.get("update") {
                    if let Ok(update_config) = update.clone().try_into() {
                        return update_config;
                    }
                }
            }
        }
    }

    UpdateConfig::default()
}

/// Check if enough time has passed since last update check
pub fn should_check_for_updates(config: &UpdateConfig, state: &UpdateState) -> bool {
    if !config.auto {
        return false;
    }

    let now = chrono::Utc::now().timestamp();
    let interval = config.effective_interval() as i64;

    match state.last_check {
        Some(last) => now - last >= interval,
        None => true, // Never checked, should check
    }
}

/// Record an update check
pub fn record_update_check(state: &mut UpdateState, result: UpdateResult, failure_reason: Option<String>) {
    let now = chrono::Utc::now().timestamp();
    state.last_check = Some(now);
    state.last_result = Some(result);

    match result {
        UpdateResult::Ok => {
            state.last_successful_update = Some(now);
            state.last_failure_reason = None;
        }
        UpdateResult::Failed => {
            state.last_failed_update = Some(now);
            state.last_failure_reason = failure_reason;
        }
        _ => {}
    }
}

/// Download URL for install script
pub fn install_script_url() -> String {
    format!(
        "https://raw.githubusercontent.com/{}/main/scripts/install.sh",
        GITHUB_REPO
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_newer() {
        assert!(UpdateCheckResult::version_newer("0.2.0", "0.0.1"));
        assert!(UpdateCheckResult::version_newer("0.1.0", "0.0.1"));
        assert!(UpdateCheckResult::version_newer("1.0.0", "0.9.9"));
        assert!(!UpdateCheckResult::version_newer("0.0.1", "0.0.1"));
        assert!(!UpdateCheckResult::version_newer("0.0.1", "0.1.0"));
    }

    #[test]
    fn test_version_with_prerelease() {
        assert!(UpdateCheckResult::version_newer("0.2.0", "0.2.0-beta"));
        assert!(UpdateCheckResult::version_newer("0.2.0-beta", "0.1.0"));
    }

    // v0.4.0: Update scheduler tests
    #[test]
    fn test_should_check_for_updates_auto_disabled() {
        let config = UpdateConfig {
            channel: UpdateChannel::Dev,
            auto: false,
            interval_seconds: None,
        };
        let state = UpdateState::default();

        // Should not check if auto is disabled
        assert!(!should_check_for_updates(&config, &state));
    }

    #[test]
    fn test_should_check_for_updates_never_checked() {
        let config = UpdateConfig {
            channel: UpdateChannel::Dev,
            auto: true,
            interval_seconds: None,
        };
        let state = UpdateState::default(); // last_check is None

        // Should check if never checked before
        assert!(should_check_for_updates(&config, &state));
    }

    #[test]
    fn test_should_check_for_updates_interval_passed() {
        let config = UpdateConfig {
            channel: UpdateChannel::Dev,
            auto: true,
            interval_seconds: Some(60), // 60 seconds
        };

        // Last check was 120 seconds ago
        let mut state = UpdateState::default();
        state.last_check = Some(chrono::Utc::now().timestamp() - 120);

        // Should check because interval has passed
        assert!(should_check_for_updates(&config, &state));
    }

    #[test]
    fn test_should_check_for_updates_interval_not_passed() {
        let config = UpdateConfig {
            channel: UpdateChannel::Dev,
            auto: true,
            interval_seconds: Some(600), // 10 minutes
        };

        // Last check was 60 seconds ago
        let mut state = UpdateState::default();
        state.last_check = Some(chrono::Utc::now().timestamp() - 60);

        // Should not check because interval has not passed
        assert!(!should_check_for_updates(&config, &state));
    }

    #[test]
    fn test_record_update_check_success() {
        let mut state = UpdateState::default();
        record_update_check(&mut state, UpdateResult::Ok, None);

        assert!(state.last_check.is_some());
        assert_eq!(state.last_result, Some(UpdateResult::Ok));
        assert!(state.last_successful_update.is_some());
        assert!(state.last_failure_reason.is_none());
    }

    #[test]
    fn test_record_update_check_failure() {
        let mut state = UpdateState::default();
        record_update_check(&mut state, UpdateResult::Failed, Some("Download failed".to_string()));

        assert!(state.last_check.is_some());
        assert_eq!(state.last_result, Some(UpdateResult::Failed));
        assert!(state.last_failed_update.is_some());
        assert_eq!(state.last_failure_reason, Some("Download failed".to_string()));
    }

    #[test]
    fn test_latest_release_url_stable() {
        let url = latest_release_url(UpdateChannel::Stable);
        assert!(url.contains("/releases/latest"));
    }

    #[test]
    fn test_latest_release_url_dev() {
        let url = latest_release_url(UpdateChannel::Dev);
        assert!(url.contains("/releases"));
        assert!(!url.contains("/latest"));
    }

    #[test]
    fn test_state_file_path() {
        let path = state_file_path();
        assert!(path.to_string_lossy().contains("update_state.json"));
    }
}
