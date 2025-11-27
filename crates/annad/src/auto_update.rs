//! Auto-update scheduler for Anna v0.5.0
//!
//! Runs in the background and checks for updates based on config.
//! Dev mode: Every 10 minutes (600s minimum) when enabled.

use anna_common::{
    load_update_state, record_update_check, save_update_state, AnnaConfigV5, GitHubRelease,
    UpdateCheckResult, UpdateResult, UpdateState, MIN_UPDATE_INTERVAL,
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

/// Auto-update scheduler state (v0.5.0)
pub struct AutoUpdateScheduler {
    config: Arc<RwLock<AnnaConfigV5>>,
    state: Arc<RwLock<UpdateState>>,
    http_client: reqwest::Client,
}

impl AutoUpdateScheduler {
    /// Create a new auto-update scheduler
    pub fn new() -> Self {
        let config = AnnaConfigV5::load();
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

    /// Check if enough time has passed since last update check
    /// v0.14.0: Works in both Normal and Dev modes when enabled
    fn should_check_for_updates(config: &AnnaConfigV5, state: &UpdateState) -> bool {
        // Must have auto-update enabled
        if !config.is_auto_update_enabled() {
            return false;
        }

        let now = chrono::Utc::now().timestamp();
        // Enforce minimum interval of 600 seconds
        let interval = config.update.effective_interval().max(MIN_UPDATE_INTERVAL) as i64;

        match state.last_check {
            Some(last) => now - last >= interval,
            None => true, // Never checked, should check
        }
    }

    /// Start the auto-update loop
    pub async fn start(self: Arc<Self>) {
        info!("🔄  Auto-update scheduler started");

        loop {
            // Reload config each iteration (allows runtime changes via NL)
            let config = AnnaConfigV5::load();
            *self.config.write().await = config.clone();

            if config.is_auto_update_enabled() {
                let state = self.state.read().await.clone();

                if Self::should_check_for_updates(&config, &state) {
                    let interval = config.update.effective_interval();
                    info!(
                        "🔍  Checking for updates (mode: {}, channel: {}, interval: {}s)",
                        config.core.mode.as_str(),
                        config.update.channel.as_str(),
                        interval
                    );

                    match self.check_and_update().await {
                        Ok(true) => {
                            info!("✅  Update applied successfully");
                            // After successful update, the daemon should restart
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

    /// Fetch the latest release from GitHub (v0.5.0 - single channel for now)
    async fn fetch_latest_release(&self, config: &AnnaConfigV5) -> Result<GitHubRelease> {
        // v0.5.0: All releases on main channel, channel value stored for future use
        let url = "https://api.github.com/repos/jjgarcianorway/anna-assistant/releases/latest"
            .to_string();

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

        // For v0.5.0, all channels use the latest release
        // Channel value is stored but doesn't affect endpoint yet
        debug!(
            "Fetching release for channel: {}",
            config.update.channel.as_str()
        );

        resp.json::<GitHubRelease>()
            .await
            .context("Failed to parse release")
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
        self.verify_checksum(
            &annactl_data,
            "annactl",
            &checksums,
            &check_result.info.latest,
        )?;

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
    /// SHA256SUMS format: <hash>  <filename>
    /// We look for lines ending with the binary name (annad or annactl)
    fn verify_checksum(
        &self,
        data: &[u8],
        name: &str,
        checksums: &str,
        _version: &str,
    ) -> Result<()> {
        // Calculate SHA256
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let calculated = format!("{:x}", hash);

        // Find expected checksum - look for simple filename match
        // SHA256SUMS format is: "hash  filename" (two spaces)
        for line in checksums.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let expected_hash = parts[0];
                let filename = parts[1];

                // Match if filename is exactly the binary name
                if filename == name {
                    if calculated == expected_hash {
                        debug!("Checksum verified for {}", name);
                        return Ok(());
                    } else {
                        anyhow::bail!(
                            "Checksum mismatch for {}: expected {}, got {}",
                            name,
                            expected_hash,
                            calculated
                        );
                    }
                }
            }
        }

        anyhow::bail!("Checksum not found for {} in SHA256SUMS", name)
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

        info!("🔄  Update applied - restarting daemon to load new version...");

        // Give time for logs to flush
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Exit with code 42 (special code for "restart after update")
        // systemd with Restart=always will restart us with the new binary
        // Code 42 chosen to be distinct from error codes (1) and success (0)
        std::process::exit(42);
    }

    /// Get current update state for reporting
    pub async fn get_state(&self) -> UpdateState {
        self.state.read().await.clone()
    }

    /// Get current config for reporting
    pub async fn get_config(&self) -> AnnaConfigV5 {
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
    use anna_common::CoreMode;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = AutoUpdateScheduler::new();
        // Should not panic and have default config
        let config = scheduler.config.try_read().unwrap();
        assert!(config.update.enabled); // Auto-update enabled by default
    }

    #[test]
    fn test_should_check_for_updates_normal_mode_enabled() {
        let config = AnnaConfigV5::default();
        let state = UpdateState::default();

        // v0.14.0: Default config has auto-update enabled in Normal mode
        // Auto-update now works in both Normal and Dev modes
        assert!(AutoUpdateScheduler::should_check_for_updates(
            &config, &state
        ));
    }

    #[test]
    fn test_should_check_for_updates_disabled() {
        let mut config = AnnaConfigV5::default();
        config.update.enabled = false;
        let state = UpdateState::default();

        // Auto-update disabled - should not check
        assert!(!AutoUpdateScheduler::should_check_for_updates(
            &config, &state
        ));
    }

    #[test]
    fn test_should_check_for_updates_dev_enabled_never_checked() {
        let mut config = AnnaConfigV5::default();
        config.core.mode = CoreMode::Dev;
        config.update.enabled = true;
        config.update.interval_seconds = MIN_UPDATE_INTERVAL;

        let state = UpdateState::default(); // last_check is None

        // Dev mode enabled, never checked - should check
        assert!(AutoUpdateScheduler::should_check_for_updates(
            &config, &state
        ));
    }

    #[test]
    fn test_should_check_for_updates_interval_not_passed() {
        let mut config = AnnaConfigV5::default();
        config.update.enabled = true;
        config.update.interval_seconds = MIN_UPDATE_INTERVAL;

        // Last check was 60 seconds ago
        let state = UpdateState {
            last_check: Some(chrono::Utc::now().timestamp() - 60),
            ..Default::default()
        };

        // Interval (600s) hasn't passed - should not check
        assert!(!AutoUpdateScheduler::should_check_for_updates(
            &config, &state
        ));
    }

    #[test]
    fn test_should_check_for_updates_interval_passed() {
        let mut config = AnnaConfigV5::default();
        config.update.enabled = true;
        config.update.interval_seconds = MIN_UPDATE_INTERVAL;

        // Last check was 700 seconds ago
        let state = UpdateState {
            last_check: Some(chrono::Utc::now().timestamp() - 700),
            ..Default::default()
        };

        // Interval (600s) has passed - should check
        assert!(AutoUpdateScheduler::should_check_for_updates(
            &config, &state
        ));
    }

    #[test]
    fn test_minimum_interval_enforced() {
        let mut config = AnnaConfigV5::default();
        config.update.enabled = true;
        config.update.interval_seconds = 100; // Below minimum

        // Last check was 200 seconds ago (above requested but below minimum)
        let state = UpdateState {
            last_check: Some(chrono::Utc::now().timestamp() - 200),
            ..Default::default()
        };

        // Even though 200 > 100 requested, minimum of 600 is enforced
        assert!(!AutoUpdateScheduler::should_check_for_updates(
            &config, &state
        ));
    }
}
