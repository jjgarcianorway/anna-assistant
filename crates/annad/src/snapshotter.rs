//! Snapshot Manager - Create and manage filesystem snapshots for rollback
//!
//! Supports multiple snapshot backends: Btrfs, Timeshift, rsync

use anna_common::{Config, RiskLevel};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info, warn};

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub method: SnapshotMethod,
    pub path: PathBuf,
    pub size_bytes: Option<u64>,
}

/// Supported snapshot methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapshotMethod {
    Btrfs,
    Timeshift,
    Rsync,
    None,
}

impl SnapshotMethod {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "btrfs" => SnapshotMethod::Btrfs,
            "timeshift" => SnapshotMethod::Timeshift,
            "rsync" => SnapshotMethod::Rsync,
            _ => SnapshotMethod::None,
        }
    }
}

/// Snapshot Manager
pub struct Snapshotter {
    config: Config,
}

impl Snapshotter {
    /// Create new snapshotter with config
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Check if snapshots are enabled and configured
    pub fn is_enabled(&self) -> bool {
        self.config.snapshots.enabled
            && SnapshotMethod::from_string(&self.config.snapshots.method) != SnapshotMethod::None
    }

    /// Check if a snapshot should be created for this risk level
    pub fn should_snapshot_for_risk(&self, risk: RiskLevel) -> bool {
        if !self.is_enabled() {
            return false;
        }

        if !self.config.snapshots.auto_snapshot_on_risk {
            return false;
        }

        self.config.snapshots.snapshot_risk_levels.contains(&risk)
    }

    /// Create a snapshot before executing an action
    pub async fn create_snapshot(&self, description: &str) -> Result<Snapshot> {
        if !self.is_enabled() {
            anyhow::bail!("Snapshots are not enabled in configuration");
        }

        let method = SnapshotMethod::from_string(&self.config.snapshots.method);

        match method {
            SnapshotMethod::Btrfs => self.create_btrfs_snapshot(description).await,
            SnapshotMethod::Timeshift => self.create_timeshift_snapshot(description).await,
            SnapshotMethod::Rsync => self.create_rsync_snapshot(description).await,
            SnapshotMethod::None => {
                anyhow::bail!("No snapshot method configured")
            }
        }
    }

    /// Create a Btrfs snapshot
    async fn create_btrfs_snapshot(&self, description: &str) -> Result<Snapshot> {
        info!("Creating Btrfs snapshot: {}", description);

        // Check if root filesystem is Btrfs
        let fs_check = Command::new("findmnt")
            .args(&["-n", "-o", "FSTYPE", "/"])
            .output()
            .await
            .context("Failed to check filesystem type")?;

        let fstype = String::from_utf8_lossy(&fs_check.stdout).trim().to_string();
        if fstype != "btrfs" {
            anyhow::bail!("Root filesystem is not Btrfs (detected: {})", fstype);
        }

        // Generate snapshot ID and path
        let snapshot_id = format!("anna-{}", Utc::now().format("%Y%m%d-%H%M%S"));
        let snapshot_path = Path::new(&self.config.snapshots.snapshot_path).join(&snapshot_id);

        // Create snapshot directory if it doesn't exist
        if let Some(parent) = snapshot_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create snapshot directory")?;
        }

        // Create Btrfs snapshot
        let output = Command::new("btrfs")
            .args(&[
                "subvolume",
                "snapshot",
                "-r", // Read-only
                "/",
                snapshot_path.to_str().unwrap(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute btrfs command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Btrfs snapshot failed: {}", stderr);
        }

        // Get snapshot size
        let size = self.get_directory_size(&snapshot_path).await.ok();

        info!("Btrfs snapshot created: {}", snapshot_path.display());

        Ok(Snapshot {
            id: snapshot_id,
            created_at: Utc::now(),
            description: description.to_string(),
            method: SnapshotMethod::Btrfs,
            path: snapshot_path,
            size_bytes: size,
        })
    }

    /// Create a Timeshift snapshot
    async fn create_timeshift_snapshot(&self, description: &str) -> Result<Snapshot> {
        info!("Creating Timeshift snapshot: {}", description);

        // Check if timeshift is installed
        let which_output = Command::new("which")
            .arg("timeshift")
            .output()
            .await
            .context("Failed to check for timeshift")?;

        if !which_output.status.success() {
            anyhow::bail!("Timeshift is not installed. Install with: pacman -S timeshift");
        }

        // Create snapshot with timeshift
        let output = Command::new("timeshift")
            .args(&[
                "--create",
                "--comments",
                description,
                "--tags",
                "A", // Anna tag
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute timeshift")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Timeshift snapshot failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse snapshot ID from output
        let snapshot_id = stdout
            .lines()
            .find(|line| line.contains("Tagged snapshot"))
            .and_then(|line| line.split_whitespace().last())
            .unwrap_or("unknown")
            .to_string();

        info!("Timeshift snapshot created: {}", snapshot_id);

        Ok(Snapshot {
            id: snapshot_id.clone(),
            created_at: Utc::now(),
            description: description.to_string(),
            method: SnapshotMethod::Timeshift,
            path: PathBuf::from(format!("/timeshift/snapshots/{}", snapshot_id)),
            size_bytes: None, // Timeshift doesn't easily provide size
        })
    }

    /// Create an rsync snapshot
    async fn create_rsync_snapshot(&self, description: &str) -> Result<Snapshot> {
        info!("Creating rsync snapshot: {}", description);

        // Generate snapshot ID and path
        let snapshot_id = format!("anna-{}", Utc::now().format("%Y%m%d-%H%M%S"));
        let snapshot_path = Path::new(&self.config.snapshots.snapshot_path).join(&snapshot_id);

        // Create snapshot directory
        tokio::fs::create_dir_all(&snapshot_path)
            .await
            .context("Failed to create snapshot directory")?;

        // Create rsync snapshot of important directories
        let dirs_to_backup = vec![
            "/etc",
            "/boot",
            "/home",
            "/usr/local",
            "/var/lib/pacman",
        ];

        for dir in dirs_to_backup {
            if !Path::new(dir).exists() {
                continue;
            }

            let dest_dir = snapshot_path.join(dir.trim_start_matches('/'));
            if let Some(parent) = dest_dir.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            info!("Backing up {} to {}", dir, dest_dir.display());

            let output = Command::new("rsync")
                .args(&[
                    "-aAXv",
                    "--quiet",
                    dir,
                    dest_dir.to_str().unwrap(),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
                .await
                .context("Failed to execute rsync")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Rsync warning for {}: {}", dir, stderr);
            }
        }

        // Save metadata
        let metadata_path = snapshot_path.join("anna-metadata.json");
        let metadata = serde_json::json!({
            "id": snapshot_id,
            "created_at": Utc::now(),
            "description": description,
            "method": "rsync",
        });
        tokio::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .await
            .context("Failed to write metadata")?;

        // Get snapshot size
        let size = self.get_directory_size(&snapshot_path).await.ok();

        info!("Rsync snapshot created: {}", snapshot_path.display());

        Ok(Snapshot {
            id: snapshot_id,
            created_at: Utc::now(),
            description: description.to_string(),
            method: SnapshotMethod::Rsync,
            path: snapshot_path,
            size_bytes: size,
        })
    }

    /// Get directory size in bytes
    async fn get_directory_size(&self, path: &Path) -> Result<u64> {
        let output = Command::new("du")
            .args(&["-sb", path.to_str().unwrap()])
            .output()
            .await
            .context("Failed to get directory size")?;

        if !output.status.success() {
            anyhow::bail!("Failed to calculate directory size");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let size_str = stdout.split_whitespace().next().unwrap_or("0");
        let size = size_str.parse::<u64>().unwrap_or(0);

        Ok(size)
    }

    /// List all snapshots
    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        if !self.is_enabled() {
            return Ok(Vec::new());
        }

        let method = SnapshotMethod::from_string(&self.config.snapshots.method);

        match method {
            SnapshotMethod::Btrfs => self.list_btrfs_snapshots().await,
            SnapshotMethod::Timeshift => self.list_timeshift_snapshots().await,
            SnapshotMethod::Rsync => self.list_rsync_snapshots().await,
            SnapshotMethod::None => Ok(Vec::new()),
        }
    }

    /// List Btrfs snapshots
    async fn list_btrfs_snapshots(&self) -> Result<Vec<Snapshot>> {
        let snapshot_dir = Path::new(&self.config.snapshots.snapshot_path);
        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = tokio::fs::read_dir(snapshot_dir).await?;
        let mut snapshots = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("anna-") {
                continue;
            }

            let path = entry.path();
            let size = self.get_directory_size(&path).await.ok();

            // Parse timestamp from name (anna-YYYYMMDD-HHMMSS)
            let created_at = chrono::NaiveDateTime::parse_from_str(
                &name.replace("anna-", ""),
                "%Y%m%d-%H%M%S",
            )
            .ok()
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or_else(Utc::now);

            snapshots.push(Snapshot {
                id: name.clone(),
                created_at,
                description: format!("Btrfs snapshot {}", name),
                method: SnapshotMethod::Btrfs,
                path,
                size_bytes: size,
            });
        }

        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    /// List Timeshift snapshots
    async fn list_timeshift_snapshots(&self) -> Result<Vec<Snapshot>> {
        let output = Command::new("timeshift")
            .args(&["--list", "--scripted"])
            .output()
            .await
            .context("Failed to list timeshift snapshots")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut snapshots = Vec::new();

        for line in stdout.lines() {
            if line.starts_with("anna-") || line.contains("Anna") {
                // Parse timeshift output (simplified)
                snapshots.push(Snapshot {
                    id: line.to_string(),
                    created_at: Utc::now(), // Would need to parse from timeshift output
                    description: "Timeshift snapshot".to_string(),
                    method: SnapshotMethod::Timeshift,
                    path: PathBuf::from("/timeshift/snapshots"),
                    size_bytes: None,
                });
            }
        }

        Ok(snapshots)
    }

    /// List rsync snapshots
    async fn list_rsync_snapshots(&self) -> Result<Vec<Snapshot>> {
        let snapshot_dir = Path::new(&self.config.snapshots.snapshot_path);
        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = tokio::fs::read_dir(snapshot_dir).await?;
        let mut snapshots = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("anna-") {
                continue;
            }

            let path = entry.path();
            let metadata_path = path.join("anna-metadata.json");

            // Try to read metadata
            if let Ok(metadata_str) = tokio::fs::read_to_string(&metadata_path).await {
                if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_str) {
                    let created_at = metadata["created_at"]
                        .as_str()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now);

                    let description = metadata["description"]
                        .as_str()
                        .unwrap_or("Rsync snapshot")
                        .to_string();

                    let size = self.get_directory_size(&path).await.ok();

                    snapshots.push(Snapshot {
                        id: name,
                        created_at,
                        description,
                        method: SnapshotMethod::Rsync,
                        path,
                        size_bytes: size,
                    });
                }
            }
        }

        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    /// Clean up old snapshots based on retention policy
    pub async fn cleanup_old_snapshots(&self) -> Result<usize> {
        if !self.is_enabled() {
            return Ok(0);
        }

        let snapshots = self.list_snapshots().await?;
        let max_snapshots = self.config.snapshots.max_snapshots;

        if snapshots.len() <= max_snapshots {
            return Ok(0);
        }

        let to_delete = &snapshots[max_snapshots..];
        let mut deleted = 0;

        for snapshot in to_delete {
            info!("Deleting old snapshot: {}", snapshot.id);
            match self.delete_snapshot(&snapshot).await {
                Ok(_) => deleted += 1,
                Err(e) => error!("Failed to delete snapshot {}: {}", snapshot.id, e),
            }
        }

        Ok(deleted)
    }

    /// Delete a specific snapshot
    async fn delete_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        match snapshot.method {
            SnapshotMethod::Btrfs => {
                Command::new("btrfs")
                    .args(&["subvolume", "delete", snapshot.path.to_str().unwrap()])
                    .output()
                    .await
                    .context("Failed to delete btrfs snapshot")?;
            }
            SnapshotMethod::Timeshift => {
                Command::new("timeshift")
                    .args(&["--delete", "--snapshot", &snapshot.id])
                    .output()
                    .await
                    .context("Failed to delete timeshift snapshot")?;
            }
            SnapshotMethod::Rsync => {
                tokio::fs::remove_dir_all(&snapshot.path)
                    .await
                    .context("Failed to delete rsync snapshot")?;
            }
            SnapshotMethod::None => {}
        }

        Ok(())
    }
}
