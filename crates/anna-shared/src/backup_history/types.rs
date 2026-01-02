//! Backup types and records

use serde::{Deserialize, Serialize};

/// Backup status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupStatus {
    Active,
    Restored,
    Expired,
    Deleted,
}

impl BackupStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            BackupStatus::Active => "+",
            BackupStatus::Restored => "R",
            BackupStatus::Expired => "x",
            BackupStatus::Deleted => "-",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BackupStatus::Active => "available",
            BackupStatus::Restored => "was restored",
            BackupStatus::Expired => "expired",
            BackupStatus::Deleted => "deleted",
        }
    }
}

/// Type of backup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackupType {
    ConfigFile,
    SystemFile,
    UserFile,
    Database,
    Package,
    Service,
}

impl BackupType {
    pub fn description(&self) -> &'static str {
        match self {
            BackupType::ConfigFile => "configuration file",
            BackupType::SystemFile => "system file",
            BackupType::UserFile => "user file",
            BackupType::Database => "database",
            BackupType::Package => "package state",
            BackupType::Service => "service config",
        }
    }
}

/// A single backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    /// Unique backup ID
    pub id: String,
    /// Original file path
    pub original_path: String,
    /// Backup file path
    pub backup_path: String,
    /// Type of backup
    pub backup_type: BackupType,
    /// Current status
    pub status: BackupStatus,
    /// Size in bytes
    pub size_bytes: u64,
    /// Timestamp when created
    pub created_at: u64,
    /// Associated ticket/change ID
    pub change_id: Option<String>,
    /// Description of what change was made
    pub change_description: Option<String>,
    /// Expiration timestamp (if any)
    pub expires_at: Option<u64>,
    /// Whether restored
    pub restored_at: Option<u64>,
}

impl BackupRecord {
    /// Check if backup is still available for restore
    pub fn is_restorable(&self) -> bool {
        self.status == BackupStatus::Active
    }

    /// Check if expired
    pub fn is_expired(&self, now: u64) -> bool {
        match self.expires_at {
            Some(exp) => now > exp,
            None => false,
        }
    }

    /// Age in seconds
    pub fn age_secs(&self, now: u64) -> u64 {
        now.saturating_sub(self.created_at)
    }
}
