//! Auto-update scheduler for Anna v0.4.0
//!
//! Runs in the background and checks for updates based on config.
//! Dev mode: Every 10 minutes when enabled.

use anna_common::{
    load_update_config, load_update_state, record_update_check, save_update_state,
    should_check_for_updates, GitHubRelease, UpdateCheckResult, UpdateConfig, UpdateResult,
    UpdateState,
};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Auto-update scheduler state
pub struct AutoUpdateScheduler {
    config: Arc<RwLock<UpdateConfig>>,
    state: Arc<RwLock<UpdateState>>,
    http_client: reqwest::Client,
}

impl AutoUpdateScheduler {
    /// Create a new auto-update scheduler
    pub fn new() -> Self {
        let config = load_update_config();
        let state = load_update_state();

        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(state)),
            http_client: reqwest::Client::builder()
                .user_agent(format!("Anna/{}", CURRENT_VERSION))
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Start the auto-update loop
    pub async fn start(self: Arc<Self>) {
        info!("🔄  Auto-update scheduler started");

        loop {
            // Reload config each iteration (allows runtime changes)
            let config = load_update_config();
            *self.config.write().await = config.clone();

            if config.auto {
                let state = self.state.read().await.clone();

                if should_check_for_updates(&config, &state) {
                    info!(
                        "🔍  Checking for updates (channel: {}, interval: {}s)",
                        config.channel.as_str(),
                        config.effective_interval()
                    );

                    match self.check_and_update().await {
                        Ok(true) => {
                            info!("✅  Update applied successfully");
                            // After successful update, the daemon should restart
                            // This will be handled by the update process itself
                        }
                        Ok(false) => {
                            debug!("📋  No update available");
                        }
                        Err(e) => {
                            error!("❌  Update check failed: {}", e);
                        }
                    }
                }
            }

            // Sleep for 60 seconds before next check cycle
            // Actual interval is enforced by should_check_for_updates
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    /// Check for updates and apply if available
    async fn check_and_update(&self) -> Result<bool> {
        let config = self.config.read().await.clone();

        // Fetch release info
        let release = self.fetch_latest_release(&config).await?;
        let check_result = UpdateCheckResult::from_release(CURRENT_VERSION, &release);

        // Update state with check timestamp
        {
            let mut state = self.state.write().await;
            state.last_check = Some(chrono::Utc::now().timestamp());
        }

        if !check_result.info.update_available {
            // No update available
            let mut state = self.state.write().await;
            record_update_check(&mut state, UpdateResult::NoUpdate, None);
            let _ = save_update_state(&state);
            return Ok(false);
        }

        info!(
            "📦  New version available: {} -> {}",
            check_result.info.current, check_result.info.latest
        );

        // Attempt to apply update
        match self.apply_update(&check_result).await {
            Ok(()) => {
                let mut state = self.state.write().await;
                state.last_version_before = Some(CURRENT_VERSION.to_string());
                state.last_version_after = Some(check_result.info.latest.clone());
                record_update_check(&mut state, UpdateResult::Ok, None);
                let _ = save_update_state(&state);

                // Request daemon restart
                info!("🔄  Requesting daemon restart...");
                self.request_restart().await;

                Ok(true)
            }
            Err(e) => {
                let error_msg = e.to_string();
                error!("❌  Update failed: {}", error_msg);

                let mut state = self.state.write().await;
                record_update_check(&mut state, UpdateResult::Failed, Some(error_msg));
                let _ = save_update_state(&state);

                Err(e)
            }
        }
    }

    /// Fetch the latest release from GitHub
    async fn fetch_latest_release(&self, config: &UpdateConfig) -> Result<GitHubRelease> {
        let url = anna_common::latest_release_url(config.channel);

        let resp = self
            .http_client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .context("Failed to fetch releases")?;

        if !resp.status().is_success() {
            anyhow::bail!("GitHub API returned {}", resp.status());
        }

        match config.channel {
            anna_common::UpdateChannel::Stable => {
                resp.json::<GitHubRelease>()
                    .await
                    .context("Failed to parse release")
            }
            anna_common::UpdateChannel::Beta | anna_common::UpdateChannel::Dev => {
                // Get list of releases and find latest matching channel
                let releases: Vec<GitHubRelease> = resp
                    .json()
                    .await
                    .context("Failed to parse releases")?;

                // For dev, prefer prereleases with -dev suffix
                // For beta, prefer prereleases with -beta suffix
                let suffix = match config.channel {
                    anna_common::UpdateChannel::Dev => "-dev",
                    anna_common::UpdateChannel::Beta => "-beta",
                    _ => "",
                };

                releases
                    .into_iter()
                    .find(|r| {
                        if suffix.is_empty() {
                            true
                        } else {
                            r.prerelease || r.tag_name.contains(suffix)
                        }
                    })
                    .ok_or_else(|| anyhow::anyhow!("No matching release found"))
            }
        }
    }

    /// Apply the update atomically
    async fn apply_update(&self, check_result: &UpdateCheckResult) -> Result<()> {
        // Need both binaries
        let annad_url = check_result
            .annad_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No annad binary URL"))?;
        let annactl_url = check_result
            .annactl_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No annactl binary URL"))?;
        let checksums_url = check_result
            .checksums_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No checksums URL"))?;

        // Create temp directory for downloads
        let temp_dir = TempDir::new().context("Failed to create temp directory")?;

        // Download checksums first
        info!("📥  Downloading checksums...");
        let checksums = self.download_text(checksums_url).await?;

        // Download binaries
        info!("📥  Downloading annad...");
        let annad_path = temp_dir.path().join("annad");
        let annad_data = self.download_binary(annad_url).await?;

        info!("📥  Downloading annactl...");
        let annactl_path = temp_dir.path().join("annactl");
        let annactl_data = self.download_binary(annactl_url).await?;

        // Verify checksums
        info!("🔐  Verifying checksums...");
        self.verify_checksum(&annad_data, "annad", &checksums, &check_result.info.latest)?;
        self.verify_checksum(&annactl_data, "annactl", &checksums, &check_result.info.latest)?;

        // Write binaries to temp
        fs::write(&annad_path, &annad_data).context("Failed to write annad")?;
        fs::write(&annactl_path, &annactl_data).context("Failed to write annactl")?;

        // Set executable permissions
        fs::set_permissions(&annad_path, fs::Permissions::from_mode(0o755))?;
        fs::set_permissions(&annactl_path, fs::Permissions::from_mode(0o755))?;

        // Atomic swap
        info!("🔄  Installing binaries...");
        self.atomic_replace("/usr/local/bin/annad", &annad_path)?;
        self.atomic_replace("/usr/local/bin/annactl", &annactl_path)?;

        info!("✅  Binaries updated successfully");
        Ok(())
    }

    /// Download text content
    async fn download_text(&self, url: &str) -> Result<String> {
        let resp = self
            .http_client
            .get(url)
            .send()
            .await
            .context("Download failed")?;

        if !resp.status().is_success() {
            anyhow::bail!("Download failed: {}", resp.status());
        }

        resp.text().await.context("Failed to read response")
    }

    /// Download binary content
    async fn download_binary(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self
            .http_client
            .get(url)
            .send()
            .await
            .context("Download failed")?;

        if !resp.status().is_success() {
            anyhow::bail!("Download failed: {}", resp.status());
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .context("Failed to read response")
    }

    /// Verify checksum
    fn verify_checksum(
        &self,
        data: &[u8],
        name: &str,
        checksums: &str,
        version: &str,
    ) -> Result<()> {
        // Calculate SHA256
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let calculated = format!("{:x}", hash);

        // Find expected checksum
        let arch = std::env::consts::ARCH;
        let target = match arch {
            "x86_64" => "x86_64-unknown-linux-gnu",
            "aarch64" => "aarch64-unknown-linux-gnu",
            _ => "x86_64-unknown-linux-gnu",
        };
        let binary_name = format!("{}-{}-{}", name, version, target);

        for line in checksums.lines() {
            if line.contains(&binary_name) {
                let expected = line.split_whitespace().next().unwrap_or("");
                if calculated == expected {
                    debug!("✅  Checksum verified for {}", name);
                    return Ok(());
                } else {
                    anyhow::bail!(
                        "Checksum mismatch for {}: expected {}, got {}",
                        name,
                        expected,
                        calculated
                    );
                }
            }
        }

        anyhow::bail!("Checksum not found for {}", binary_name)
    }

    /// Atomic file replacement with backup
    fn atomic_replace(&self, target: &str, source: &PathBuf) -> Result<()> {
        let target_path = PathBuf::from(target);
        let backup_path = target_path.with_extension("bak");

        // Backup existing file
        if target_path.exists() {
            fs::copy(&target_path, &backup_path).context("Failed to create backup")?;
        }

        // Attempt atomic replace
        match fs::rename(source, &target_path) {
            Ok(()) => {
                // Success - remove backup
                let _ = fs::remove_file(&backup_path);
                Ok(())
            }
            Err(e) => {
                // Failed - restore backup
                warn!("⚠️  Replace failed, restoring backup: {}", e);
                if backup_path.exists() {
                    let _ = fs::rename(&backup_path, &target_path);
                }
                Err(e).context("Atomic replace failed")
            }
        }
    }

    /// Request daemon restart via systemd
    async fn request_restart(&self) {
        // Write restart marker file
        let marker = PathBuf::from("/var/lib/anna/restart_requested");
        if let Err(e) = fs::write(&marker, chrono::Utc::now().to_rfc3339()) {
            warn!("⚠️  Failed to write restart marker: {}", e);
        }

        // The actual restart is handled by systemd's Restart=on-failure
        // or by an external watcher. We just signal the intent here.
        info!("📋  Restart marker written. Daemon will restart on next cycle.");

        // Give time for logs to flush, then exit cleanly
        tokio::time::sleep(Duration::from_secs(1)).await;
        std::process::exit(0);
    }

    /// Get current update state for reporting
    pub async fn get_state(&self) -> UpdateState {
        self.state.read().await.clone()
    }

    /// Get current update config for reporting
    pub async fn get_config(&self) -> UpdateConfig {
        self.config.read().await.clone()
    }
}

impl Default for AutoUpdateScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = AutoUpdateScheduler::new();
        // Should not panic and have default config
        assert!(!scheduler.config.try_read().unwrap().auto);
    }
}
