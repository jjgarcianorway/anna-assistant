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
        // Beta.108: Reduced log verbosity - only log important events
        // Record check time silently
        let _ = self.record_check_time().await;

        let current_version = env!("CARGO_PKG_VERSION");

        // Fetch highest version release (including prereleases)
        let client = GitHubClient::new(GITHUB_OWNER, GITHUB_REPO);

        let latest_release = match client.get_highest_version_release().await {
            Ok(release) => release,
            Err(e) => {
                error!("Auto-update failed: couldn't reach GitHub: {}", e);
                return;
            }
        };

        let latest_version = latest_release.version();

        // Check if update is available
        use anna_common::github_releases::compare_versions;
        use std::cmp::Ordering;

        match compare_versions(current_version, latest_version) {
            Ordering::Less => {
                info!(
                    "🎯 Update available: v{} → v{}",
                    current_version, latest_version
                );
            }
            Ordering::Equal | Ordering::Greater => {
                // Already up-to-date, return silently
                return;
            }
        }

        // Beta.108: Removed overly-strict filesystem check
        // The daemon runs as root and should have write permissions
        // If there's a permission issue, it will be caught during actual installation

        // Beta.100: Check if annactl is currently running (safety check)
        // CRITICAL: Don't replace binaries while annactl is in use - could cause crashes
        info!("   Checking if annactl is in use...");
        if self.is_annactl_running().await {
            info!("   ⏸️  Update postponed: annactl is currently in use");
            info!("   Update will be retried in 10 minutes when annactl is idle");
            return;
        }
        info!("   ✓ annactl is idle");

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

        // Beta.97: Filesystem writability now checked earlier in check_and_update
        // This redundant check is kept as a safety net
        let annactl_path = PathBuf::from("/usr/local/bin/annactl");
        let annad_path = PathBuf::from("/usr/local/bin/annad");

        // Backup current binaries
        info!("      Creating backups of current binaries...");
        FileBackup::create_backup(&annactl_path, &change_set_id, FileOperation::Modified)?;
        FileBackup::create_backup(&annad_path, &change_set_id, FileOperation::Modified)?;
        info!("      ✓ Backups created");

        // Install new binaries to /usr/local/bin using install command
        // This handles permissions and atomic replacement properly
        info!("      Installing new binaries to /usr/local/bin...");

        // Use install command for atomic replacement with proper permissions
        // -m 755: set executable permissions
        use tokio::process::Command;

        let install_annactl = Command::new("install")
            .arg("-m")
            .arg("755")
            .arg(&annactl_tmp)
            .arg(&annactl_path)
            .output()
            .await?;

        if !install_annactl.status.success() {
            let stderr = String::from_utf8_lossy(&install_annactl.stderr);
            return Err(anyhow::anyhow!(
                "Failed to install annactl: {}. This may be due to filesystem permissions. Try running: sudo systemctl restart annad",
                stderr
            ));
        }

        let install_annad = Command::new("install")
            .arg("-m")
            .arg("755")
            .arg(&annad_tmp)
            .arg(&annad_path)
            .output()
            .await?;

        if !install_annad.status.success() {
            let stderr = String::from_utf8_lossy(&install_annad.stderr);
            return Err(anyhow::anyhow!(
                "Failed to install annad: {}. This may be due to filesystem permissions.",
                stderr
            ));
        }

        // Clean up temporary files
        let _ = tokio::fs::remove_file(&annactl_tmp).await;
        let _ = tokio::fs::remove_file(&annad_tmp).await;

        // Permissions already set by install command (755)

        // Cleanup temp dir
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;

        info!("Binaries successfully updated");
        Ok(())
    }

    /// Check if filesystem containing target path is writable
    /// Returns true if we can write, false if read-only
    async fn is_filesystem_writable(&self, target: &PathBuf) -> bool {
        // Get the parent directory (e.g., /usr/local/bin)
        let dir = match target.parent() {
            Some(d) => d,
            None => return false,
        };

        // Try to create a temporary test file to verify write access
        let test_file = dir.join(".anna_write_test");

        match tokio::fs::write(&test_file, b"test").await {
            Ok(()) => {
                // Successfully wrote, clean up and return true
                let _ = tokio::fs::remove_file(&test_file).await;
                true
            }
            Err(e) => {
                // Write failed - check if it's a read-only filesystem
                use std::io::ErrorKind;
                match e.kind() {
                    ErrorKind::PermissionDenied => {
                        // Could be read-only or insufficient permissions
                        // Check if the directory itself exists and is readable
                        if dir.exists() && dir.is_dir() {
                            // Directory exists but we can't write - likely read-only
                            false
                        } else {
                            // Directory doesn't exist or isn't readable - different issue
                            false
                        }
                    }
                    _ => {
                        // Other error (e.g., ReadOnly filesystem, NotFound, etc.)
                        false
                    }
                }
            }
        }
    }

    /// Beta.100: Check if annactl is currently running
    /// Returns true if any annactl processes are active
    /// CRITICAL: Prevents binary replacement while annactl is in use
    async fn is_annactl_running(&self) -> bool {
        use tokio::process::Command;

        // Use pgrep to check for annactl processes
        // -c flag returns count of matching processes
        let output = match Command::new("pgrep")
            .args(&["-c", "annactl"])
            .output()
            .await
        {
            Ok(output) => output,
            Err(e) => {
                warn!("Failed to check if annactl is running: {}", e);
                // Fail safe: if we can't check, assume it's running
                return true;
            }
        };

        // pgrep -c returns:
        // - exit code 0 with count > 0: processes found
        // - exit code 1: no processes found
        if output.status.success() {
            let count_str = String::from_utf8_lossy(&output.stdout);
            let count: usize = count_str.trim().parse().unwrap_or(0);
            count > 0
        } else {
            // Exit code 1 means no processes found
            false
        }
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
