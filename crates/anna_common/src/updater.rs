//! Auto-update system for Anna
//!
//! Checks GitHub releases and performs self-updates.

use crate::{UpdateChannel, VersionInfo};
use serde::{Deserialize, Serialize};

const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";
const GITHUB_API: &str = "https://api.github.com";

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
        UpdateChannel::Beta => {
            // For beta, we get all releases and find the latest pre-release
            format!("{}/repos/{}/releases", GITHUB_API, GITHUB_REPO)
        }
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
}
