// GitHub Releases API Client
// Phase 3.10: AUR-Aware Auto-Upgrade System
//
// Fetches release information from GitHub for auto-updates

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub prerelease: bool,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

impl GitHubRelease {
    /// Get version from tag name (strip 'v' prefix)
    pub fn version(&self) -> &str {
        self.tag_name.strip_prefix('v').unwrap_or(&self.tag_name)
    }

    /// Find asset by name pattern
    pub fn find_asset(&self, pattern: &str) -> Option<&GitHubAsset> {
        self.assets.iter().find(|a| a.name.contains(pattern))
    }
}

/// GitHub API client
pub struct GitHubClient {
    repo_owner: String,
    repo_name: String,
    user_agent: String,
}

impl GitHubClient {
    /// Create new GitHub client
    pub fn new(repo_owner: impl Into<String>, repo_name: impl Into<String>) -> Self {
        Self {
            repo_owner: repo_owner.into(),
            repo_name: repo_name.into(),
            user_agent: format!("anna-assistant/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Get latest release (excluding prereleases)
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/vnd.github.v3+json")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context("Failed to fetch latest release from GitHub")?;

        if !response.status().is_success() {
            anyhow::bail!("GitHub API returned error: {}", response.status());
        }

        let release: GitHubRelease = response
            .json()
            .await
            .context("Failed to parse GitHub release JSON")?;

        Ok(release)
    }

    /// Get all releases (including prereleases)
    pub async fn get_releases(&self) -> Result<Vec<GitHubRelease>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.repo_owner, self.repo_name
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/vnd.github.v3+json")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context("Failed to fetch releases from GitHub")?;

        if !response.status().is_success() {
            anyhow::bail!("GitHub API returned error: {}", response.status());
        }

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .context("Failed to parse GitHub releases JSON")?;

        Ok(releases)
    }

    /// Download asset to file
    pub async fn download_asset(&self, asset: &GitHubAsset, dest_path: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .get(&asset.browser_download_url)
            .header("User-Agent", &self.user_agent)
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .await
            .context("Failed to download asset")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read download bytes")?;

        tokio::fs::write(dest_path, bytes)
            .await
            .context("Failed to write downloaded file")?;

        Ok(())
    }
}

/// Version comparison (semver-like)
pub fn compare_versions(current: &str, latest: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    // Strip 'v' prefix if present
    let current = current.strip_prefix('v').unwrap_or(current);
    let latest = latest.strip_prefix('v').unwrap_or(latest);

    // Parse version components
    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    current_parts.cmp(&latest_parts)
}

/// Check if update is available
pub fn is_update_available(current: &str, latest: &str) -> bool {
    compare_versions(current, latest) == std::cmp::Ordering::Less
}

/// Parse version string into comparable tuple
fn parse_version(version: &str) -> (u32, u32, u32, String) {
    // Format: major.minor.patch-prerelease
    let parts: Vec<&str> = version.split('-').collect();
    let version_parts = parts[0];
    let prerelease = parts.get(1).unwrap_or(&"").to_string();

    let nums: Vec<u32> = version_parts
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    let major = nums.get(0).copied().unwrap_or(0);
    let minor = nums.get(1).copied().unwrap_or(0);
    let patch = nums.get(2).copied().unwrap_or(0);

    (major, minor, patch, prerelease)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_update_available("3.9.0", "3.9.1"));
        assert!(is_update_available("3.9.1", "3.10.0"));
        assert!(is_update_available("3.9.0-alpha.1", "3.9.0"));
        assert!(!is_update_available("3.10.0", "3.9.1"));
        assert!(!is_update_available("3.10.0", "3.10.0"));
    }

    #[test]
    fn test_version_parsing() {
        let v1 = parse_version("3.9.1-alpha.1");
        let v2 = parse_version("3.9.1");
        let v3 = parse_version("3.10.0");

        assert!(v1 < v2); // alpha < stable
        assert!(v2 < v3); // 3.9 < 3.10
    }

    #[test]
    fn test_strip_v_prefix() {
        assert_eq!(
            "3.9.1",
            "v3.9.1".strip_prefix('v').unwrap()
        );
    }
}
