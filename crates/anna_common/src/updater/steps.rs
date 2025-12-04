//! Update Execution Steps v0.0.73
//!
//! Implements the complete update flow with proper error handling,
//! atomic installs, restart semantics, and rollback support.

use super::lock::UpdateLock;
use super::state::{UpdateStateV73, UpdateStep, UpdateStepResult};
use super::{UPDATE_BACKUP_DIR, UPDATE_STATE_FILE};
use crate::install_state::InstallState;
use crate::ops_log::OpsLog;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Staging directory for downloads
const STAGING_DIR: &str = "/var/lib/anna/internal/update_staging";

/// Result of update execution
#[derive(Debug)]
pub enum UpdateResult {
    /// No update available
    NoUpdate,
    /// Update completed successfully
    Updated { version: String },
    /// Update failed
    Failed { reason: String },
    /// Rolled back after failure
    RolledBack { reason: String },
}

/// GitHub release info
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub tag: String,
    pub version: String,
    pub annad_url: Option<String>,
    pub annactl_url: Option<String>,
    pub checksum_url: Option<String>,
}

/// Update executor - orchestrates the complete update flow
pub struct UpdateExecutor {
    state: UpdateStateV73,
    ops_log: OpsLog,
    /// Mock mode for testing
    mock_mode: bool,
    /// Mock release for testing
    mock_release: Option<ReleaseInfo>,
}

impl UpdateExecutor {
    pub fn new() -> Self {
        Self {
            state: UpdateStateV73::load(),
            ops_log: OpsLog::open(),
            mock_mode: false,
            mock_release: None,
        }
    }

    /// Create executor with mock release for testing
    #[cfg(test)]
    pub fn with_mock_release(release: ReleaseInfo) -> Self {
        Self {
            state: UpdateStateV73::default(),
            ops_log: OpsLog::open(),
            mock_mode: true,
            mock_release: Some(release),
        }
    }

    /// Run the complete update flow
    pub fn run(&mut self) -> UpdateResult {
        self.ops_log.log("updater", "run_started", None);

        // Step 1: Acquire lock
        let lock = match self.acquire_lock() {
            Ok(l) => l,
            Err(e) => {
                self.ops_log
                    .log("updater", "lock_failed", Some(&e.to_string()));
                return UpdateResult::Failed {
                    reason: e.to_string(),
                };
            }
        };

        // Step 2: Check remote release
        let release = match self.check_remote() {
            Ok(Some(r)) => r,
            Ok(None) => {
                let _ = self.state.record_no_update();
                self.ops_log.log("updater", "no_update", None);
                return UpdateResult::NoUpdate;
            }
            Err(e) => {
                let _ = self.state.fail_step(&e);
                return UpdateResult::Failed { reason: e };
            }
        };

        // Step 3: Compare versions
        if !self.is_newer(&release.version) {
            let _ = self.state.record_no_update();
            self.ops_log.log("updater", "already_current", None);
            return UpdateResult::NoUpdate;
        }

        let _ = self.state.record_update_available(&release.version);
        self.ops_log.log(
            "updater",
            "update_available",
            Some(&format!(
                "{} -> {}",
                self.state.current_version, release.version
            )),
        );

        // Step 4: Download assets
        let (cli_path, daemon_path) = match self.download_assets(&release) {
            Ok(paths) => paths,
            Err(e) => {
                let _ = self.state.fail_step(&e);
                return UpdateResult::Failed { reason: e };
            }
        };

        // Step 5: Verify assets
        if let Err(e) = self.verify_assets(&cli_path, &daemon_path) {
            let _ = self.state.fail_step(&e);
            return UpdateResult::Failed { reason: e };
        }

        // Step 6-7: Install CLI and daemon with backup
        if let Err(e) = self.install_with_backup(&cli_path, &daemon_path) {
            // Rollback on install failure
            self.ops_log.log("updater", "install_failed", Some(&e));
            if let Err(rb_err) = self.rollback() {
                let _ = self
                    .state
                    .record_rollback(&format!("{} (rollback also failed: {})", e, rb_err));
                return UpdateResult::Failed {
                    reason: format!("{} and rollback failed: {}", e, rb_err),
                };
            }
            let _ = self.state.record_rollback(&e);
            return UpdateResult::RolledBack { reason: e };
        }

        // Step 8: Restart daemon if updated
        let need_restart = daemon_path.is_some();
        if need_restart {
            if let Err(e) = self.restart_daemon() {
                self.ops_log.log("updater", "restart_failed", Some(&e));
                // Rollback on restart failure
                if let Err(rb_err) = self.rollback() {
                    let _ = self
                        .state
                        .record_rollback(&format!("{} (rollback failed: {})", e, rb_err));
                    return UpdateResult::Failed {
                        reason: format!("{} and rollback failed: {}", e, rb_err),
                    };
                }
                // Restart again after rollback
                let _ = self.restart_daemon();
                let _ = self.state.record_rollback(&e);
                return UpdateResult::RolledBack { reason: e };
            }
        }

        // Step 9: Healthcheck
        if let Err(e) = self.healthcheck(&release.version) {
            self.ops_log.log("updater", "healthcheck_failed", Some(&e));
            // Rollback on healthcheck failure
            if let Err(rb_err) = self.rollback() {
                let _ = self
                    .state
                    .record_rollback(&format!("{} (rollback failed: {})", e, rb_err));
                return UpdateResult::Failed {
                    reason: format!("{} and rollback failed: {}", e, rb_err),
                };
            }
            if need_restart {
                let _ = self.restart_daemon();
            }
            let _ = self.state.record_rollback(&e);
            return UpdateResult::RolledBack { reason: e };
        }

        // Step 10: Cleanup and success
        let _ = self.cleanup();
        let _ = self.state.record_updated(&release.version);
        self.ops_log
            .log("updater", "update_complete", Some(&release.version));

        // Lock released on drop
        drop(lock);

        UpdateResult::Updated {
            version: release.version,
        }
    }

    fn acquire_lock(&mut self) -> Result<UpdateLock, String> {
        let _ = self.state.start_step(UpdateStep::AcquireLock);
        UpdateLock::acquire("update_start").map_err(|e| e.to_string())
    }

    fn check_remote(&mut self) -> Result<Option<ReleaseInfo>, String> {
        let _ = self.state.start_step(UpdateStep::CheckRemote);

        if self.mock_mode {
            return Ok(self.mock_release.clone());
        }

        // Fetch from GitHub API
        let output = Command::new("curl")
            .args([
                "-sS",
                "--max-time",
                "30",
                "-H",
                "Accept: application/vnd.github.v3+json",
                "-H",
                "User-Agent: anna-assistant",
                "https://api.github.com/repos/jjgarcianorway/anna-assistant/releases/latest",
            ])
            .output()
            .map_err(|e| format!("curl failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check for rate limit
            if stderr.contains("rate limit") || stderr.contains("403") {
                return Err("GitHub API rate limit. Will retry later.".to_string());
            }
            return Err(format!("GitHub API error: {}", stderr));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let tag = json["tag_name"].as_str().ok_or("No tag_name in release")?;
        let version = tag.trim_start_matches('v').to_string();

        // Find asset URLs
        let assets = json["assets"].as_array().ok_or("No assets in release")?;
        let mut annad_url = None;
        let mut annactl_url = None;
        let mut checksum_url = None;

        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            let url = asset["browser_download_url"].as_str();

            if name.starts_with("annad-") && name.contains("linux") {
                annad_url = url.map(|s| s.to_string());
            } else if name.starts_with("annactl-") && name.contains("linux") {
                annactl_url = url.map(|s| s.to_string());
            } else if name == "SHA256SUMS" {
                checksum_url = url.map(|s| s.to_string());
            }
        }

        Ok(Some(ReleaseInfo {
            tag: tag.to_string(),
            version,
            annad_url,
            annactl_url,
            checksum_url,
        }))
    }

    fn is_newer(&self, new_version: &str) -> bool {
        crate::is_newer_version(new_version, &self.state.current_version)
    }

    fn download_assets(
        &mut self,
        release: &ReleaseInfo,
    ) -> Result<(Option<PathBuf>, Option<PathBuf>), String> {
        let _ = self.state.start_step(UpdateStep::DownloadAssets);

        fs::create_dir_all(STAGING_DIR)
            .map_err(|e| format!("Failed to create staging dir: {}", e))?;

        let mut cli_path = None;
        let mut daemon_path = None;

        // Download annactl if available
        if let Some(ref url) = release.annactl_url {
            let path = PathBuf::from(STAGING_DIR).join("annactl.new");
            self.download_file(url, &path)?;
            cli_path = Some(path);
            self.ops_log.log("updater", "downloaded_cli", None);
        }

        // Download annad if available
        if let Some(ref url) = release.annad_url {
            let path = PathBuf::from(STAGING_DIR).join("annad.new");
            self.download_file(url, &path)?;
            daemon_path = Some(path);
            self.ops_log.log("updater", "downloaded_daemon", None);
        }

        Ok((cli_path, daemon_path))
    }

    fn download_file(&self, url: &str, dest: &Path) -> Result<(), String> {
        let output = Command::new("curl")
            .args(["-sSL", "--max-time", "120", "-o"])
            .arg(dest)
            .arg(url)
            .output()
            .map_err(|e| format!("Download failed: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Download failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Check file exists and has size > 0
        let meta = fs::metadata(dest).map_err(|e| format!("File check failed: {}", e))?;
        if meta.len() == 0 {
            return Err("Downloaded file is empty".to_string());
        }

        Ok(())
    }

    fn verify_assets(
        &mut self,
        cli_path: &Option<PathBuf>,
        daemon_path: &Option<PathBuf>,
    ) -> Result<(), String> {
        let _ = self.state.start_step(UpdateStep::VerifyAssets);

        // Compute and store checksums
        if let Some(path) = cli_path {
            let checksum = self.compute_sha256(path)?;
            self.state.downloaded_cli_checksum = Some(checksum.clone());
            self.ops_log
                .log("updater", "cli_checksum", Some(&checksum[..16]));
        }

        if let Some(path) = daemon_path {
            let checksum = self.compute_sha256(path)?;
            self.state.downloaded_daemon_checksum = Some(checksum.clone());
            self.ops_log
                .log("updater", "daemon_checksum", Some(&checksum[..16]));

            // Verify it's executable (has ELF header)
            self.verify_executable(path)?;
        }

        let _ = self.state.save();
        Ok(())
    }

    fn compute_sha256(&self, path: &Path) -> Result<String, String> {
        let mut file = fs::File::open(path).map_err(|e| format!("Open failed: {}", e))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| format!("Read error: {}", e))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn verify_executable(&self, path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(path).map_err(|e| format!("Open failed: {}", e))?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)
            .map_err(|e| format!("Read failed: {}", e))?;

        // ELF magic: 0x7F 'E' 'L' 'F'
        if magic != [0x7F, b'E', b'L', b'F'] {
            return Err("Downloaded file is not a valid ELF executable".to_string());
        }
        Ok(())
    }

    fn install_with_backup(
        &mut self,
        cli_path: &Option<PathBuf>,
        daemon_path: &Option<PathBuf>,
    ) -> Result<(), String> {
        let install_state = InstallState::load_or_default();

        // Create backup directory
        let backup_dir = PathBuf::from(UPDATE_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Backup dir creation failed: {}", e))?;

        // Backup and install CLI
        if let Some(staged) = cli_path {
            let _ = self.state.start_step(UpdateStep::InstallCli);
            if let Some(ref info) = install_state.annactl {
                // Backup current
                let backup_path =
                    backup_dir.join(format!("annactl-{}", self.state.current_version));
                fs::copy(&info.path, &backup_path)
                    .map_err(|e| format!("CLI backup failed: {}", e))?;
                let checksum = self.compute_sha256(&info.path)?;
                self.state.backup_cli_path = Some(backup_path.to_string_lossy().to_string());
                self.state.backup_cli_checksum = Some(checksum);

                // Atomic install: copy to .new, then rename
                self.atomic_install(staged, &info.path)?;
                self.ops_log.log("updater", "installed_cli", None);
            }
        }

        // Backup and install daemon
        if let Some(staged) = daemon_path {
            let _ = self.state.start_step(UpdateStep::InstallDaemon);
            if let Some(ref info) = install_state.annad {
                // Backup current
                let backup_path =
                    backup_dir.join(format!("annad-{}", self.state.current_daemon_version));
                fs::copy(&info.path, &backup_path)
                    .map_err(|e| format!("Daemon backup failed: {}", e))?;
                let checksum = self.compute_sha256(&info.path)?;
                self.state.backup_daemon_path = Some(backup_path.to_string_lossy().to_string());
                self.state.backup_daemon_checksum = Some(checksum);

                // Atomic install
                self.atomic_install(staged, &info.path)?;
                self.ops_log.log("updater", "installed_daemon", None);
            }
        }

        let _ = self.state.save();
        Ok(())
    }

    fn atomic_install(&self, src: &Path, dest: &Path) -> Result<(), String> {
        let temp_dest = dest.with_extension("new");

        // Copy to temp location
        fs::copy(src, &temp_dest).map_err(|e| format!("Copy failed: {}", e))?;

        // Set executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temp_dest, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Chmod failed: {}", e))?;
        }

        // Atomic rename
        fs::rename(&temp_dest, dest).map_err(|e| format!("Rename failed: {}", e))?;

        Ok(())
    }

    fn restart_daemon(&mut self) -> Result<(), String> {
        let _ = self.state.start_step(UpdateStep::RestartDaemon);

        let output = Command::new("systemctl")
            .args(["restart", "annad"])
            .output()
            .map_err(|e| format!("systemctl failed: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Daemon restart failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Wait briefly for daemon to start
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Verify daemon is running
        let status = Command::new("systemctl")
            .args(["is-active", "annad"])
            .output()
            .map_err(|e| format!("Status check failed: {}", e))?;

        if !status.status.success() {
            return Err("Daemon failed to start after restart".to_string());
        }

        self.ops_log.log("updater", "daemon_restarted", None);
        Ok(())
    }

    fn healthcheck(&mut self, expected_version: &str) -> Result<(), String> {
        let _ = self.state.start_step(UpdateStep::Healthcheck);

        // Check CLI version
        let install_state = InstallState::load_or_default();
        if let Some(ref info) = install_state.annactl {
            let output = Command::new(&info.path)
                .arg("--version")
                .output()
                .map_err(|e| format!("CLI version check failed: {}", e))?;

            let version_output = String::from_utf8_lossy(&output.stdout);
            if !version_output.contains(expected_version) {
                return Err(format!(
                    "CLI version mismatch: expected {}, got {}",
                    expected_version,
                    version_output.trim()
                ));
            }
        }

        // Check daemon version via systemctl show (check if running)
        let status = Command::new("systemctl")
            .args(["is-active", "annad"])
            .output()
            .map_err(|e| format!("Daemon check failed: {}", e))?;

        if !status.status.success() {
            return Err("Daemon is not running after update".to_string());
        }

        self.ops_log
            .log("updater", "healthcheck_passed", Some(expected_version));
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), String> {
        let _ = self.state.start_step(UpdateStep::Rollback);
        self.ops_log.log("updater", "rollback_started", None);

        let install_state = InstallState::load_or_default();

        // Rollback CLI
        if let (Some(backup_path), Some(ref checksum)) =
            (&self.state.backup_cli_path, &self.state.backup_cli_checksum)
        {
            if Path::new(backup_path).exists() {
                if let Some(ref info) = install_state.annactl {
                    fs::copy(backup_path, &info.path)
                        .map_err(|e| format!("CLI rollback failed: {}", e))?;

                    // Verify checksum
                    let restored_checksum = self.compute_sha256(&info.path)?;
                    if &restored_checksum != checksum {
                        return Err("CLI rollback checksum mismatch".to_string());
                    }
                    self.ops_log.log("updater", "cli_rolled_back", None);
                }
            }
        }

        // Rollback daemon
        if let (Some(backup_path), Some(ref checksum)) = (
            &self.state.backup_daemon_path,
            &self.state.backup_daemon_checksum,
        ) {
            if Path::new(backup_path).exists() {
                if let Some(ref info) = install_state.annad {
                    fs::copy(backup_path, &info.path)
                        .map_err(|e| format!("Daemon rollback failed: {}", e))?;

                    // Verify checksum
                    let restored_checksum = self.compute_sha256(&info.path)?;
                    if &restored_checksum != checksum {
                        return Err("Daemon rollback checksum mismatch".to_string());
                    }
                    self.ops_log.log("updater", "daemon_rolled_back", None);
                }
            }
        }

        Ok(())
    }

    fn cleanup(&self) -> Result<(), String> {
        // Remove staging directory
        if Path::new(STAGING_DIR).exists() {
            let _ = fs::remove_dir_all(STAGING_DIR);
        }

        // Keep only last 2 backups
        if let Ok(entries) = fs::read_dir(UPDATE_BACKUP_DIR) {
            let mut backups: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            backups.sort_by(|a, b| {
                let a_time = a.metadata().and_then(|m| m.modified()).ok();
                let b_time = b.metadata().and_then(|m| m.modified()).ok();
                b_time.cmp(&a_time)
            });

            for backup in backups.into_iter().skip(4) {
                let _ = fs::remove_file(backup.path());
            }
        }

        Ok(())
    }
}

impl Default for UpdateExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        let executor = UpdateExecutor::new();
        // These tests depend on current version, but we can test the logic
        assert!(crate::is_newer_version("99.99.99", "0.0.1"));
        assert!(!crate::is_newer_version("0.0.1", "0.0.2"));
    }

    #[test]
    fn test_release_info_creation() {
        let release = ReleaseInfo {
            tag: "v0.0.73".to_string(),
            version: "0.0.73".to_string(),
            annad_url: Some("https://example.com/annad".to_string()),
            annactl_url: Some("https://example.com/annactl".to_string()),
            checksum_url: None,
        };
        assert_eq!(release.version, "0.0.73");
        assert!(release.annad_url.is_some());
    }
}
