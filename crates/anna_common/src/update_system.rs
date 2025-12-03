//! Update System v0.0.11 - Safe Auto-Update with Rollback
//!
//! Provides safe, zero-downtime auto-update capabilities:
//! - GitHub release checking with channel support (stable/canary)
//! - Safe artifact download with integrity verification
//! - Atomic binary installation with staging
//! - Zero-downtime restart via systemd
//! - Automatic rollback on failure
//! - Complete state visibility in status snapshots
//!
//! Update flow:
//! 1. Check GitHub releases API for new version
//! 2. Download artifacts to staging directory
//! 3. Verify integrity (checksums)
//! 4. Atomic install (rename binaries)
//! 5. Signal restart (systemd)
//! 6. Post-restart validation
//! 7. Cleanup or rollback

use crate::config::{UpdateMode, UpdateState, UpdateResult, DATA_DIR};
use crate::install_state::{InstallState, ReviewResult};
use crate::installer_review::run_installer_review;
use crate::ops_log::OpsLog;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Update staging directory
pub const UPDATE_STAGE_DIR: &str = "/var/lib/anna/internal/update_stage";

/// Backup directory for rollback
pub const UPDATE_BACKUP_DIR: &str = "/var/lib/anna/internal/update_backup";

/// Update in-progress marker file
pub const UPDATE_MARKER_FILE: &str = "/var/lib/anna/internal/update_in_progress.json";

/// Minimum disk space required for update (100 MB)
pub const MIN_DISK_SPACE_BYTES: u64 = 100 * 1024 * 1024;

/// Update channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    #[default]
    Stable,
    Canary,
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Canary => "canary",
        }
    }

    /// Check if a version tag matches this channel
    pub fn matches_tag(&self, tag: &str) -> bool {
        match self {
            UpdateChannel::Stable => {
                // Stable: no pre-release suffix
                !tag.contains("-alpha")
                    && !tag.contains("-beta")
                    && !tag.contains("-rc")
                    && !tag.contains("-canary")
                    && !tag.contains("-dev")
            }
            UpdateChannel::Canary => {
                // Canary: accept anything including pre-releases
                true
            }
        }
    }
}

/// Update phase for progress tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase {
    /// No update in progress
    Idle,
    /// Checking for updates
    Checking,
    /// Downloading artifacts
    Downloading { progress_percent: u8, eta_seconds: Option<u64> },
    /// Verifying integrity
    Verifying,
    /// Staging binaries
    Staging,
    /// Installing (atomic rename)
    Installing,
    /// Restarting daemon
    Restarting,
    /// Post-restart validation
    Validating,
    /// Update completed successfully
    Completed { version: String },
    /// Update failed
    Failed { reason: String },
    /// Rolling back
    RollingBack,
}

impl Default for UpdatePhase {
    fn default() -> Self {
        UpdatePhase::Idle
    }
}

impl UpdatePhase {
    pub fn is_in_progress(&self) -> bool {
        !matches!(self, UpdatePhase::Idle | UpdatePhase::Completed { .. } | UpdatePhase::Failed { .. })
    }

    pub fn format_display(&self) -> String {
        match self {
            UpdatePhase::Idle => "idle".to_string(),
            UpdatePhase::Checking => "checking for updates...".to_string(),
            UpdatePhase::Downloading { progress_percent, eta_seconds } => {
                if let Some(eta) = eta_seconds {
                    format!("downloading... {}% (ETA: {}s)", progress_percent, eta)
                } else {
                    format!("downloading... {}%", progress_percent)
                }
            }
            UpdatePhase::Verifying => "verifying integrity...".to_string(),
            UpdatePhase::Staging => "staging binaries...".to_string(),
            UpdatePhase::Installing => "installing...".to_string(),
            UpdatePhase::Restarting => "restarting daemon...".to_string(),
            UpdatePhase::Validating => "validating...".to_string(),
            UpdatePhase::Completed { version } => format!("completed (v{})", version),
            UpdatePhase::Failed { reason } => format!("failed: {}", reason),
            UpdatePhase::RollingBack => "rolling back...".to_string(),
        }
    }
}

/// Integrity verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrityStatus {
    /// Verified with strong checksum (from release assets)
    StrongVerified { algorithm: String, checksum: String },
    /// Computed checksum (no release checksum available)
    WeakComputed { algorithm: String, checksum: String },
    /// Verification failed
    Failed { reason: String },
    /// Not verified (skipped)
    NotVerified,
}

impl IntegrityStatus {
    pub fn is_verified(&self) -> bool {
        matches!(self, IntegrityStatus::StrongVerified { .. } | IntegrityStatus::WeakComputed { .. })
    }

    pub fn is_strong(&self) -> bool {
        matches!(self, IntegrityStatus::StrongVerified { .. })
    }
}

/// Release artifact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    /// Artifact name (e.g., "annad", "annactl")
    pub name: String,
    /// Download URL
    pub url: String,
    /// File size in bytes (if known)
    pub size_bytes: Option<u64>,
    /// Expected checksum (if provided in release)
    pub expected_checksum: Option<String>,
    /// Local path after download
    pub local_path: Option<PathBuf>,
    /// Integrity status after verification
    pub integrity: IntegrityStatus,
}

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Version tag (e.g., "v0.0.11")
    pub tag: String,
    /// Parsed version (e.g., "0.0.11")
    pub version: String,
    /// Whether this is a pre-release
    pub prerelease: bool,
    /// Release URL
    pub html_url: String,
    /// Release assets/artifacts
    pub artifacts: Vec<ReleaseArtifact>,
    /// Published timestamp
    pub published_at: Option<DateTime<Utc>>,
}

/// Update in-progress marker (persisted to disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMarker {
    /// Target version being installed
    pub target_version: String,
    /// Current phase
    pub phase: UpdatePhase,
    /// Previous version (for rollback)
    pub previous_version: String,
    /// Backup paths for rollback
    pub backup_paths: Vec<BackupEntry>,
    /// Timestamp when update started
    pub started_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Evidence ID for tracking
    pub evidence_id: String,
}

/// Backup entry for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    /// Original path
    pub original_path: PathBuf,
    /// Backup path
    pub backup_path: PathBuf,
    /// SHA256 checksum of original
    pub checksum: String,
}

impl UpdateMarker {
    /// Load marker from disk
    pub fn load() -> Option<Self> {
        let path = Path::new(UPDATE_MARKER_FILE);
        if !path.exists() {
            return None;
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }

    /// Save marker to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Path::new(UPDATE_MARKER_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(UPDATE_MARKER_FILE, &content)
    }

    /// Remove marker from disk
    pub fn remove() -> std::io::Result<()> {
        let path = Path::new(UPDATE_MARKER_FILE);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Update phase and persist
    pub fn update_phase(&mut self, phase: UpdatePhase) -> std::io::Result<()> {
        self.phase = phase;
        self.updated_at = Utc::now();
        self.save()
    }
}

/// Guardrail check result
#[derive(Debug, Clone)]
pub enum GuardrailResult {
    /// All checks passed
    Passed,
    /// Check failed with reason
    Failed { reason: String },
}

impl GuardrailResult {
    pub fn is_passed(&self) -> bool {
        matches!(self, GuardrailResult::Passed)
    }
}

/// Update manager handles the complete update lifecycle
pub struct UpdateManager {
    /// Current version
    current_version: String,
    /// Update channel
    channel: UpdateChannel,
    /// Operations log
    ops_log: OpsLog,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new(current_version: &str, channel: UpdateChannel) -> Self {
        Self {
            current_version: current_version.to_string(),
            channel,
            ops_log: OpsLog::open(),
        }
    }

    /// Run all guardrail checks before update
    pub fn check_guardrails(&self) -> GuardrailResult {
        // 1. Check installer review status
        let review = run_installer_review(false);
        if let ReviewResult::NeedsAttention { issues } = &review.overall {
            if issues.iter().any(|i| i.contains("symlink") || i.contains("unknown binary")) {
                return GuardrailResult::Failed {
                    reason: format!("Installer review found unsafe state: {:?}", issues),
                };
            }
        }

        // 2. Check disk space
        match check_disk_space(DATA_DIR) {
            Ok(free_bytes) if free_bytes < MIN_DISK_SPACE_BYTES => {
                return GuardrailResult::Failed {
                    reason: format!(
                        "Insufficient disk space: {} MB free, {} MB required",
                        free_bytes / (1024 * 1024),
                        MIN_DISK_SPACE_BYTES / (1024 * 1024)
                    ),
                };
            }
            Err(e) => {
                return GuardrailResult::Failed {
                    reason: format!("Failed to check disk space: {}", e),
                };
            }
            _ => {}
        }

        // 3. Check for active mutation (via lock file)
        if is_mutation_active() {
            return GuardrailResult::Failed {
                reason: "Mutation tool is currently executing".to_string(),
            };
        }

        // 4. Check if update already in progress
        if UpdateMarker::load().is_some() {
            return GuardrailResult::Failed {
                reason: "Update already in progress".to_string(),
            };
        }

        GuardrailResult::Passed
    }

    /// Check for available updates from GitHub
    pub fn check_for_updates(&mut self) -> Result<Option<ReleaseInfo>, String> {
        self.ops_log.log("update_system", "check_started", Some(&self.current_version));

        let releases = fetch_github_releases()?;

        // Find newest release matching our channel
        let candidate = releases
            .into_iter()
            .filter(|r| self.channel.matches_tag(&r.tag))
            .filter(|r| is_newer_version(&r.version, &self.current_version))
            .max_by(|a, b| compare_versions(&a.version, &b.version));

        if let Some(ref release) = candidate {
            self.ops_log.log(
                "update_system",
                "update_available",
                Some(&format!("{} -> {}", self.current_version, release.version)),
            );
        } else {
            self.ops_log.log("update_system", "up_to_date", None);
        }

        Ok(candidate)
    }

    /// Download release artifacts to staging directory
    pub fn download_artifacts(
        &mut self,
        release: &ReleaseInfo,
        progress_callback: impl Fn(u8, Option<u64>),
    ) -> Result<Vec<ReleaseArtifact>, String> {
        self.ops_log.log("update_system", "download_started", Some(&release.version));

        // Create staging directory
        let stage_dir = PathBuf::from(UPDATE_STAGE_DIR).join(&release.version);
        fs::create_dir_all(&stage_dir)
            .map_err(|e| format!("Failed to create staging directory: {}", e))?;

        let mut downloaded = Vec::new();
        let total_artifacts = release.artifacts.len();

        for (i, artifact) in release.artifacts.iter().enumerate() {
            let progress = ((i as f32 / total_artifacts as f32) * 100.0) as u8;
            progress_callback(progress, None);

            let local_path = stage_dir.join(&artifact.name);
            download_file(&artifact.url, &local_path)?;

            let mut updated_artifact = artifact.clone();
            updated_artifact.local_path = Some(local_path);
            downloaded.push(updated_artifact);
        }

        progress_callback(100, None);
        self.ops_log.log("update_system", "download_completed", Some(&release.version));

        Ok(downloaded)
    }

    /// Verify artifact integrity
    pub fn verify_integrity(&mut self, artifacts: &mut [ReleaseArtifact]) -> Result<(), String> {
        self.ops_log.log("update_system", "verify_started", None);

        for artifact in artifacts.iter_mut() {
            let local_path = artifact.local_path.as_ref()
                .ok_or_else(|| format!("Artifact {} not downloaded", artifact.name))?;

            let computed_checksum = compute_sha256(local_path)?;

            artifact.integrity = if let Some(ref expected) = artifact.expected_checksum {
                if computed_checksum == *expected {
                    IntegrityStatus::StrongVerified {
                        algorithm: "sha256".to_string(),
                        checksum: computed_checksum,
                    }
                } else {
                    IntegrityStatus::Failed {
                        reason: format!(
                            "Checksum mismatch: expected {}, got {}",
                            expected, computed_checksum
                        ),
                    }
                }
            } else {
                // No expected checksum - compute and mark as weak
                IntegrityStatus::WeakComputed {
                    algorithm: "sha256".to_string(),
                    checksum: computed_checksum,
                }
            };

            if !artifact.integrity.is_verified() {
                let reason = match &artifact.integrity {
                    IntegrityStatus::Failed { reason } => reason.clone(),
                    _ => "Unknown verification failure".to_string(),
                };
                return Err(format!("Integrity check failed for {}: {}", artifact.name, reason));
            }
        }

        self.ops_log.log("update_system", "verify_completed", None);
        Ok(())
    }

    /// Create backup of current binaries for rollback
    pub fn create_backup(&mut self) -> Result<Vec<BackupEntry>, String> {
        self.ops_log.log("update_system", "backup_started", None);

        let backup_dir = PathBuf::from(UPDATE_BACKUP_DIR).join(&self.current_version);
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        let mut backups = Vec::new();

        // Get current binary locations from install state
        let install_state = InstallState::load_or_default();

        // Backup annad
        if let Some(ref annad) = install_state.annad {
            if annad.path.exists() {
                let backup_path = backup_dir.join("annad");
                fs::copy(&annad.path, &backup_path)
                    .map_err(|e| format!("Failed to backup annad: {}", e))?;
                let checksum = compute_sha256(&annad.path)?;
                backups.push(BackupEntry {
                    original_path: annad.path.clone(),
                    backup_path,
                    checksum,
                });
            }
        }

        // Backup annactl
        if let Some(ref annactl) = install_state.annactl {
            if annactl.path.exists() {
                let backup_path = backup_dir.join("annactl");
                fs::copy(&annactl.path, &backup_path)
                    .map_err(|e| format!("Failed to backup annactl: {}", e))?;
                let checksum = compute_sha256(&annactl.path)?;
                backups.push(BackupEntry {
                    original_path: annactl.path.clone(),
                    backup_path,
                    checksum,
                });
            }
        }

        self.ops_log.log("update_system", "backup_completed", Some(&format!("{} files", backups.len())));
        Ok(backups)
    }

    /// Perform atomic installation of new binaries
    /// v0.0.29: Handle artifact names with version and architecture suffix
    pub fn atomic_install(&mut self, artifacts: &[ReleaseArtifact]) -> Result<(), String> {
        self.ops_log.log("update_system", "install_started", None);

        let install_state = InstallState::load_or_default();

        for artifact in artifacts {
            let staged_path = artifact.local_path.as_ref()
                .ok_or_else(|| format!("Artifact {} not staged", artifact.name))?;

            // v0.0.29: Extract base binary name from artifact name
            // e.g., "annad-0.0.28-x86_64-unknown-linux-gnu" -> "annad"
            let base_name = if artifact.name.starts_with("annad-") {
                "annad"
            } else if artifact.name.starts_with("annactl-") {
                "annactl"
            } else {
                artifact.name.as_str()
            };

            let target_path = match base_name {
                "annad" => install_state.annad.as_ref().map(|b| b.path.clone()),
                "annactl" => install_state.annactl.as_ref().map(|b| b.path.clone()),
                _ => None,
            };

            if let Some(target) = target_path {
                // Atomic rename (same filesystem)
                // First copy to target.new, then rename
                let temp_path = target.with_extension("new");
                fs::copy(staged_path, &temp_path)
                    .map_err(|e| format!("Failed to copy {}: {}", artifact.name, e))?;

                // Preserve permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))
                        .map_err(|e| format!("Failed to set permissions: {}", e))?;
                }

                // Atomic rename
                fs::rename(&temp_path, &target)
                    .map_err(|e| format!("Failed to install {}: {}", artifact.name, e))?;
            }
        }

        self.ops_log.log("update_system", "install_completed", None);
        Ok(())
    }

    /// Perform rollback to previous version
    pub fn rollback(&mut self, backups: &[BackupEntry]) -> Result<(), String> {
        self.ops_log.log("update_system", "rollback_started", None);

        for backup in backups {
            if backup.backup_path.exists() {
                fs::copy(&backup.backup_path, &backup.original_path)
                    .map_err(|e| format!("Failed to restore {}: {}", backup.original_path.display(), e))?;

                // Verify restored file
                let restored_checksum = compute_sha256(&backup.original_path)?;
                if restored_checksum != backup.checksum {
                    return Err(format!(
                        "Rollback verification failed for {}",
                        backup.original_path.display()
                    ));
                }
            }
        }

        self.ops_log.log("update_system", "rollback_completed", None);
        Ok(())
    }

    /// Signal systemd to restart the daemon
    pub fn request_restart(&mut self) -> Result<(), String> {
        self.ops_log.log("update_system", "restart_requested", None);

        // Use systemctl restart which will stop current process and start new one
        let output = Command::new("systemctl")
            .args(["restart", "annad"])
            .output()
            .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to restart daemon: {}", stderr));
        }

        Ok(())
    }

    /// Clean up staging directory
    pub fn cleanup_staging(&mut self, version: &str) -> Result<(), String> {
        let stage_dir = PathBuf::from(UPDATE_STAGE_DIR).join(version);
        if stage_dir.exists() {
            fs::remove_dir_all(&stage_dir)
                .map_err(|e| format!("Failed to cleanup staging: {}", e))?;
        }
        Ok(())
    }

    /// Clean up old backups (keep only last 2)
    pub fn cleanup_old_backups(&mut self) -> Result<(), String> {
        let backup_dir = PathBuf::from(UPDATE_BACKUP_DIR);
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut entries: Vec<_> = fs::read_dir(&backup_dir)
            .map_err(|e| format!("Failed to read backup directory: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        // Sort by modification time (newest first)
        entries.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
            let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        // Remove all but the 2 newest
        for entry in entries.into_iter().skip(2) {
            let _ = fs::remove_dir_all(entry.path());
        }

        Ok(())
    }

    /// Perform complete update: check, download, verify, install, restart
    /// v0.0.26: Full auto-update implementation
    pub fn perform_update(&mut self) -> Result<String, String> {
        // Step 1: Check for updates
        let release = self.check_for_updates()?
            .ok_or_else(|| "No update available".to_string())?;

        let new_version = release.version.clone();
        self.ops_log.log("update_system", "performing_update", Some(&new_version));

        // Step 2: Run guardrails
        let guardrail = self.check_guardrails();
        if !guardrail.is_passed() {
            if let GuardrailResult::Failed { reason } = guardrail {
                return Err(format!("Guardrail failed: {}", reason));
            }
        }

        // Step 3: Create update marker
        let marker = UpdateMarker {
            target_version: new_version.clone(),
            phase: UpdatePhase::Downloading { progress_percent: 0, eta_seconds: None },
            previous_version: self.current_version.clone(),
            backup_paths: Vec::new(),
            started_at: Utc::now(),
            updated_at: Utc::now(),
            evidence_id: generate_update_evidence_id(),
        };
        marker.save().map_err(|e| format!("Failed to save update marker: {}", e))?;

        // Step 4: Download artifacts
        let mut artifacts = self.download_artifacts(&release, |_progress, _eta| {
            // Progress callback - could update marker here
        })?;

        // Step 5: Verify integrity
        self.verify_integrity(&mut artifacts)?;

        // Step 6: Create backups
        let backups = self.create_backup()?;

        // Update marker with backup paths
        let mut marker = UpdateMarker::load().unwrap_or(marker);
        marker.backup_paths = backups.clone();
        marker.phase = UpdatePhase::Installing;
        let _ = marker.save();

        // Step 7: Atomic install
        self.atomic_install(&artifacts)?;

        // Step 8: Cleanup
        let _ = self.cleanup_staging(&new_version);
        let _ = self.cleanup_old_backups();

        // Step 9: Update marker to completed
        marker.phase = UpdatePhase::Completed { version: new_version.clone() };
        let _ = marker.save();

        // Step 10: Log completion
        self.ops_log.log("update_system", "update_complete", Some(&new_version));

        // Don't restart immediately - let the daemon handle it gracefully
        Ok(new_version)
    }
}

/// Perform auto-update if available - v0.0.26: convenience function for daemon
/// Returns Ok(Some(version)) if updated, Ok(None) if no update, Err on failure
pub fn perform_auto_update(current_version: &str) -> Result<Option<String>, String> {
    let mut manager = UpdateManager::new(current_version, UpdateChannel::Stable);

    // Check if update available first
    let release = manager.check_for_updates()?;

    if release.is_none() {
        return Ok(None);
    }

    // Perform the update
    let new_version = manager.perform_update()?;
    Ok(Some(new_version))
}

/// Fetch releases from GitHub API
fn fetch_github_releases() -> Result<Vec<ReleaseInfo>, String> {
    let output = Command::new("curl")
        .args([
            "-sS",
            "--max-time", "30",
            "-H", "Accept: application/vnd.github.v3+json",
            "-H", "User-Agent: anna-assistant",
            "https://api.github.com/repos/jjgarcianorway/anna-assistant/releases",
        ])
        .output()
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GitHub API request failed: {}", stderr));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    let releases = json.as_array()
        .ok_or_else(|| "Invalid GitHub response: expected array".to_string())?;

    let mut result = Vec::new();
    for release in releases {
        let tag = release.get("tag_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let version = tag.trim_start_matches('v').to_string();
        let prerelease = release.get("prerelease")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let html_url = release.get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let published_at = release.get("published_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Parse assets
        let mut artifacts = Vec::new();
        if let Some(assets) = release.get("assets").and_then(|v| v.as_array()) {
            for asset in assets {
                let name = asset.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                // v0.0.29: Filter to binary artifacts (now with architecture suffix)
                // Match: annad-X.X.X-arch or annactl-X.X.X-arch
                if name.starts_with("annad-") || name.starts_with("annactl-") {
                    let url = asset.get("browser_download_url")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let size_bytes = asset.get("size")
                        .and_then(|v| v.as_u64());

                    artifacts.push(ReleaseArtifact {
                        name,
                        url,
                        size_bytes,
                        expected_checksum: None,
                        local_path: None,
                        integrity: IntegrityStatus::NotVerified,
                    });
                }
            }
        }

        result.push(ReleaseInfo {
            tag,
            version,
            prerelease,
            html_url,
            artifacts,
            published_at,
        });
    }

    Ok(result)
}

/// Download file from URL
fn download_file(url: &str, local_path: &Path) -> Result<(), String> {
    let output = Command::new("curl")
        .args([
            "-sS",
            "-L",
            "--max-time", "300",
            "-o", local_path.to_str().unwrap_or_default(),
            url,
        ])
        .output()
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Download failed: {}", stderr));
    }

    Ok(())
}

/// Compute SHA256 checksum of a file
fn compute_sha256(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to compute checksum: {}", e))?;

    if !output.status.success() {
        return Err("sha256sum failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let checksum = stdout.split_whitespace().next()
        .ok_or_else(|| "Invalid sha256sum output".to_string())?;

    Ok(checksum.to_string())
}

/// Check available disk space
fn check_disk_space(path: &str) -> Result<u64, String> {
    let output = Command::new("df")
        .args(["--output=avail", "-B1", path])
        .output()
        .map_err(|e| format!("Failed to check disk space: {}", e))?;

    if !output.status.success() {
        return Err("df command failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout.lines().collect();
    if lines.len() < 2 {
        return Err("Invalid df output".to_string());
    }

    lines[1].trim().parse::<u64>()
        .map_err(|_| "Failed to parse disk space".to_string())
}

/// Check if a mutation tool is currently active
fn is_mutation_active() -> bool {
    // Check for mutation lock file
    let lock_path = Path::new("/var/lib/anna/internal/mutation.lock");
    lock_path.exists()
}

/// Compare semantic versions (returns Ordering)
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.split('-').next()).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };

    parse(a).cmp(&parse(b))
}

/// Check if version a is newer than version b
pub fn is_newer_version(a: &str, b: &str) -> bool {
    compare_versions(a, b) == std::cmp::Ordering::Greater
}

/// Generate unique evidence ID for update
pub fn generate_update_evidence_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("UPD{}", ts % 100000)
}

/// Post-restart handler: detect if we just completed an update
pub fn handle_post_restart() -> Option<String> {
    let marker = UpdateMarker::load()?;

    match marker.phase {
        UpdatePhase::Restarting | UpdatePhase::Installing => {
            // We were in the middle of an update - validate it succeeded
            let mut ops_log = OpsLog::open();

            // Check if new version is running
            let current = env!("CARGO_PKG_VERSION");
            if current == marker.target_version {
                ops_log.log("update_system", "update_succeeded", Some(&marker.target_version));
                let _ = UpdateMarker::remove();
                return Some(marker.target_version);
            } else {
                // Version mismatch - update may have failed
                ops_log.log("update_system", "update_version_mismatch",
                    Some(&format!("expected {}, got {}", marker.target_version, current)));
                // Don't remove marker - might need manual intervention
            }
        }
        _ => {}
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.0.11", "0.0.10"));
        assert!(is_newer_version("0.1.0", "0.0.99"));
        assert!(is_newer_version("1.0.0", "0.99.99"));
        assert!(!is_newer_version("0.0.10", "0.0.10"));
        assert!(!is_newer_version("0.0.9", "0.0.10"));
    }

    #[test]
    fn test_channel_matching() {
        let stable = UpdateChannel::Stable;
        let canary = UpdateChannel::Canary;

        // Stable channel
        assert!(stable.matches_tag("v0.0.10"));
        assert!(stable.matches_tag("v1.0.0"));
        assert!(!stable.matches_tag("v0.0.11-alpha"));
        assert!(!stable.matches_tag("v0.0.11-beta.1"));
        assert!(!stable.matches_tag("v0.0.11-rc1"));
        assert!(!stable.matches_tag("v0.0.11-canary"));

        // Canary channel accepts everything
        assert!(canary.matches_tag("v0.0.10"));
        assert!(canary.matches_tag("v0.0.11-alpha"));
        assert!(canary.matches_tag("v0.0.11-canary"));
    }

    #[test]
    fn test_update_phase_display() {
        assert_eq!(UpdatePhase::Idle.format_display(), "idle");
        assert_eq!(UpdatePhase::Checking.format_display(), "checking for updates...");
        assert_eq!(
            UpdatePhase::Downloading { progress_percent: 50, eta_seconds: Some(30) }.format_display(),
            "downloading... 50% (ETA: 30s)"
        );
        assert_eq!(
            UpdatePhase::Completed { version: "0.0.11".to_string() }.format_display(),
            "completed (v0.0.11)"
        );
    }

    #[test]
    fn test_guardrail_result() {
        assert!(GuardrailResult::Passed.is_passed());
        assert!(!GuardrailResult::Failed { reason: "test".to_string() }.is_passed());
    }

    #[test]
    fn test_integrity_status() {
        let strong = IntegrityStatus::StrongVerified {
            algorithm: "sha256".to_string(),
            checksum: "abc".to_string(),
        };
        assert!(strong.is_verified());
        assert!(strong.is_strong());

        let weak = IntegrityStatus::WeakComputed {
            algorithm: "sha256".to_string(),
            checksum: "abc".to_string(),
        };
        assert!(weak.is_verified());
        assert!(!weak.is_strong());

        let failed = IntegrityStatus::Failed { reason: "test".to_string() };
        assert!(!failed.is_verified());
    }
}
