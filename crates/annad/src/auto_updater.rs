// Auto-Updater Service for Daemon
// Phase Next: Aggressive Auto-Update with Transparent Notifications
//
// 10-minute check interval with automatic binary replacement for manual installations

use anna_common::github_releases::{is_update_available, GitHubClient};
use anna_common::installation_source::{detect_current_installation, InstallationSource};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

const GITHUB_OWNER: &str = "jjgarcianorway";
const GITHUB_REPO: &str = "anna-assistant";
const CHECK_INTERVAL: Duration = Duration::from_secs(10 * 60); // 10 minutes
const INITIAL_DELAY: Duration = Duration::from_secs(5 * 60); // 5 minutes after startup
const LAST_CHECK_FILE: &str = "/var/lib/anna/last_update_check";
const UPDATE_RECORD_FILE: &str = "/var/lib/anna/last_update_applied";
const PENDING_NOTICE_FILE: &str = "/var/lib/anna/pending_update_notice";

pub struct AutoUpdater {
    enabled: bool,
    source: InstallationSource,
}

impl AutoUpdater {
    pub fn new() -> Self {
        let source = detect_current_installation();
        let enabled = source.allows_auto_update();

        info!(
            "Auto-updater initialized: source={:?}, enabled={}",
            source, enabled
        );

        Self { enabled, source }
    }

    /// Start auto-update background service
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            if !self.enabled {
                info!(
                    "Auto-update service disabled for installation source: {}",
                    self.source.description()
                );
                return;
            }

            info!("Auto-update service started (checks every 10 minutes)");

            // Initial delay: 5 minutes after startup
            time::sleep(INITIAL_DELAY).await;

            loop {
                self.check_and_update().await;

                // Wait 10 minutes before next check
                time::sleep(CHECK_INTERVAL).await;
            }
        })
    }

    /// Check for updates and install if available
    async fn check_and_update(&self) {
        info!("🔄 Auto-update check starting...");
        info!("   Current version: v{}", env!("CARGO_PKG_VERSION"));

        // Record check time
        if let Err(e) = self.record_check_time().await {
            warn!("Failed to record check time: {}", e);
        }

        let current_version = env!("CARGO_PKG_VERSION");

        // Fetch highest version release (including prereleases)
        // Beta.73: Fixed to use get_highest_version_release() instead of get_latest_release()
        // This fixes the bug where beta.68 was treated as newer than beta.72!
        info!("   Fetching latest release from GitHub...");
        let client = GitHubClient::new(GITHUB_OWNER, GITHUB_REPO);

        let latest_release = match client.get_highest_version_release().await {
            Ok(release) => {
                info!("   ✓ Successfully fetched release info");
                release
            }
            Err(e) => {
                error!("   ✗ Failed to fetch latest release from GitHub: {}", e);
                error!("   Check network connectivity and GitHub API status");
                return;
            }
        };

        let latest_version = latest_release.version();
        info!("   Latest version on GitHub: v{}", latest_version);

        // Check if update is available
        use anna_common::github_releases::compare_versions;
        use std::cmp::Ordering;

        match compare_versions(current_version, latest_version) {
            Ordering::Less => {
                // Update available
                info!(
                    "   🎯 Update available: v{} → v{}",
                    current_version, latest_version
                );
            }
            Ordering::Equal => {
                info!("   ✓ Already running latest version: v{}", current_version);
                return;
            }
            Ordering::Greater => {
                // Running development version newer than GitHub
                info!(
                    "   📌 Running development version v{} (GitHub latest: v{})",
                    current_version, latest_version
                );
                info!("   No update needed - current version is newer");
                return;
            }
        }

        // Step 4: Perform automatic update
        info!("   Starting automatic update process...");
        match self.perform_update(&latest_release, latest_version).await {
            Ok(()) => {
                info!("   ✓ Update successfully installed: v{}", latest_version);

                // Write update record and pending notice
                if let Err(e) = self
                    .write_update_records(current_version, latest_version)
                    .await
                {
                    warn!("   Failed to write update records: {}", e);
                }

                // Restart daemon
                info!("   Restarting daemon to apply update...");
                if let Err(e) = self.restart_daemon().await {
                    error!("   ✗ Failed to restart daemon: {}", e);
                    error!("   You may need to manually restart: sudo systemctl restart annad");
                }
            }
            Err(e) => {
                error!("   ✗ Failed to perform update: {}", e);
                error!("   Auto-update will retry in 10 minutes");
            }
        }
    }

    /// Perform the actual update
    async fn perform_update(
        &self,
        release: &anna_common::github_releases::GitHubRelease,
        version: &str,
    ) -> anyhow::Result<()> {
        use anna_common::file_backup::{FileBackup, FileOperation};
        use std::path::PathBuf;

        info!("      Preparing update to v{}...", version);

        // Create change set ID for backup tracking
        let change_set_id = format!("auto_update_{}", version);

        // Download new binaries
        let temp_dir = PathBuf::from("/tmp/anna_update");
        info!("      Creating temporary directory: {}", temp_dir.display());
        tokio::fs::create_dir_all(&temp_dir).await?;

        // Download annactl and annad binaries
        // Note: GitHub release assets are named simply "annactl" and "annad"
        let annactl_url = format!(
            "https://github.com/{}/{}/releases/download/{}/annactl",
            GITHUB_OWNER, GITHUB_REPO, release.tag_name
        );
        let annad_url = format!(
            "https://github.com/{}/{}/releases/download/{}/annad",
            GITHUB_OWNER, GITHUB_REPO, release.tag_name
        );
        let checksums_url = format!(
            "https://github.com/{}/{}/releases/download/{}/SHA256SUMS",
            GITHUB_OWNER, GITHUB_REPO, release.tag_name
        );

        let annactl_tmp = temp_dir.join("annactl");
        let annad_tmp = temp_dir.join("annad");
        let checksums_tmp = temp_dir.join("SHA256SUMS");

        // Download files
        info!("      Downloading annactl binary...");
        self.download_file(&annactl_url, &annactl_tmp).await?;
        info!("      ✓ annactl downloaded");

        info!("      Downloading annad binary...");
        self.download_file(&annad_url, &annad_tmp).await?;
        info!("      ✓ annad downloaded");

        // Try to download and verify checksums (optional)
        info!("      Checking for SHA256SUMS...");
        match self.download_file(&checksums_url, &checksums_tmp).await {
            Ok(()) => {
                info!("      ✓ SHA256SUMS downloaded");
                info!("      Verifying checksums...");
                if let Err(e) = self.verify_checksums(&temp_dir).await {
                    warn!("      Checksum verification failed: {}", e);
                    warn!("      Proceeding without checksum verification");
                }
            }
            Err(e) => {
                warn!("      SHA256SUMS not available: {}", e);
                warn!("      Proceeding without checksum verification");
            }
        }

        // Backup current binaries
        info!("      Creating backups of current binaries...");
        let annactl_path = PathBuf::from("/usr/local/bin/annactl");
        let annad_path = PathBuf::from("/usr/local/bin/annad");

        FileBackup::create_backup(&annactl_path, &change_set_id, FileOperation::Modified)?;
        FileBackup::create_backup(&annad_path, &change_set_id, FileOperation::Modified)?;
        info!("      ✓ Backups created");

        // Copy new binaries to /usr/local/bin (can't use rename across filesystems)
        info!("      Installing new binaries to /usr/local/bin...");
        tokio::fs::copy(&annactl_tmp, &annactl_path).await?;
        tokio::fs::copy(&annad_tmp, &annad_path).await?;

        // Clean up temporary files
        let _ = tokio::fs::remove_file(&annactl_tmp).await;
        let _ = tokio::fs::remove_file(&annad_tmp).await;

        // Set executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&annactl_path, perms.clone())?;
            std::fs::set_permissions(&annad_path, perms)?;
        }

        // Cleanup temp dir
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;

        info!("Binaries successfully updated");
        Ok(())
    }

    /// Download a file from URL
    async fn download_file(&self, url: &str, dest: &PathBuf) -> anyhow::Result<()> {
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to download {}: HTTP {}", url, response.status());
        }

        let bytes = response.bytes().await?;
        tokio::fs::write(dest, &bytes).await?;

        Ok(())
    }

    /// Verify SHA256 checksums
    async fn verify_checksums(&self, dir: &PathBuf) -> anyhow::Result<()> {
        use sha2::{Digest, Sha256};

        let checksums_file = dir.join("SHA256SUMS");
        let checksums_content = tokio::fs::read_to_string(&checksums_file).await?;

        for line in checksums_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                continue;
            }

            let expected_hash = parts[0];
            let filename = parts[1];

            // Skip files we didn't download
            if !filename.starts_with("anna") {
                continue;
            }

            let file_path = dir.join(filename.trim_start_matches("./"));
            if !file_path.exists() {
                continue;
            }

            // Compute actual hash
            let file_bytes = tokio::fs::read(&file_path).await?;
            let mut hasher = Sha256::new();
            hasher.update(&file_bytes);
            let actual_hash = format!("{:x}", hasher.finalize());

            if actual_hash != expected_hash {
                anyhow::bail!(
                    "Checksum mismatch for {}: expected {}, got {}",
                    filename,
                    expected_hash,
                    actual_hash
                );
            }

            info!("✓ Checksum verified: {}", filename);
        }

        Ok(())
    }

    /// Write update records for notification
    async fn write_update_records(
        &self,
        from_version: &str,
        to_version: &str,
    ) -> Result<(), std::io::Error> {
        let record = format!("{}|{}", from_version, to_version);
        tokio::fs::write(UPDATE_RECORD_FILE, &record).await?;
        tokio::fs::write(PENDING_NOTICE_FILE, &record).await?;
        Ok(())
    }

    /// Restart the daemon
    async fn restart_daemon(&self) -> anyhow::Result<()> {
        // Use systemctl to restart the daemon
        let output = tokio::process::Command::new("systemctl")
            .args(&["restart", "annad"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to restart daemon: {}", stderr);
        }

        Ok(())
    }

    /// Record last check time
    async fn record_check_time(&self) -> Result<(), std::io::Error> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        tokio::fs::write(LAST_CHECK_FILE, timestamp).await
    }
}

/// Get last update check time
pub async fn get_last_check_time() -> Option<String> {
    tokio::fs::read_to_string(LAST_CHECK_FILE).await.ok()
}
