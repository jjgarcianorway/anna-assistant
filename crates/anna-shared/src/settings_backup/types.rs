// v0.0.575: Settings Backup Types (Phase 151)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Backup type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    /// Full settings backup
    Full,
    /// Incremental (changes only)
    Incremental,
    /// Before major change
    PreChange,
    /// Scheduled automatic backup
    Scheduled,
    /// Manual user backup
    Manual,
}

impl std::fmt::Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "Full"),
            Self::Incremental => write!(f, "Incremental"),
            Self::PreChange => write!(f, "Pre-Change"),
            Self::Scheduled => write!(f, "Scheduled"),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

/// Backup status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupStatus {
    /// Backup successful
    Success,
    /// Backup failed
    Failed,
    /// Backup in progress
    InProgress,
    /// Backup corrupted
    Corrupted,
    /// Backup expired (too old)
    Expired,
}

impl std::fmt::Display for BackupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::Failed => write!(f, "Failed"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Corrupted => write!(f, "Corrupted"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMeta {
    /// Backup ID
    pub id: u64,
    /// Backup type
    pub backup_type: BackupType,
    /// Status
    pub status: BackupStatus,
    /// Creation timestamp
    pub created: chrono::DateTime<chrono::Utc>,
    /// Size in bytes
    pub size_bytes: u64,
    /// Description
    pub description: String,
    /// Version at backup time
    pub version: String,
    /// Checksum
    pub checksum: Option<String>,
    /// File path
    pub path: Option<PathBuf>,
}

impl BackupMeta {
    /// Create new backup metadata
    pub fn new(id: u64, backup_type: BackupType, description: impl Into<String>) -> Self {
        Self {
            id,
            backup_type,
            status: BackupStatus::InProgress,
            created: chrono::Utc::now(),
            size_bytes: 0,
            description: description.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checksum: None,
            path: None,
        }
    }

    /// Mark as successful
    pub fn success(mut self, size: u64) -> Self {
        self.status = BackupStatus::Success;
        self.size_bytes = size;
        self
    }

    /// Mark as failed
    pub fn failed(mut self) -> Self {
        self.status = BackupStatus::Failed;
        self
    }

    /// Set checksum
    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }

    /// Set path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    /// Age of backup
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.created
    }

    /// Human-readable age
    pub fn age_display(&self) -> String {
        let age = self.age();
        if age.num_days() > 0 {
            format!("{} days ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{} hours ago", age.num_hours())
        } else {
            format!("{} minutes ago", age.num_minutes())
        }
    }

    /// Is valid backup?
    pub fn is_valid(&self) -> bool {
        self.status == BackupStatus::Success
    }
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup directory
    pub backup_dir: PathBuf,
    /// Max backups to keep
    pub max_backups: usize,
    /// Auto-backup enabled
    pub auto_backup: bool,
    /// Backup interval in hours
    pub interval_hours: u32,
    /// Backup before changes
    pub backup_before_changes: bool,
    /// Compress backups
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from("/var/lib/anna/backups"),
            max_backups: 10,
            auto_backup: true,
            interval_hours: 24,
            backup_before_changes: true,
            compress: false,
        }
    }
}
