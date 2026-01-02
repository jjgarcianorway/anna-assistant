// v0.0.576: Settings Restore Types
// Type definitions for restore functionality

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Restore mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreMode {
    /// Full restore - replace all settings
    Full,
    /// Partial restore - only specific categories
    Partial,
    /// Merge - keep current, add missing from backup
    Merge,
    /// Selective - user picks what to restore
    Selective,
}

impl std::fmt::Display for RestoreMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "Full"),
            Self::Partial => write!(f, "Partial"),
            Self::Merge => write!(f, "Merge"),
            Self::Selective => write!(f, "Selective"),
        }
    }
}

/// Restore status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreStatus {
    /// Restore pending
    Pending,
    /// Validating backup
    Validating,
    /// In progress
    InProgress,
    /// Completed successfully
    Success,
    /// Failed
    Failed,
    /// Rolled back
    RolledBack,
}

impl std::fmt::Display for RestoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Validating => write!(f, "Validating"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Success => write!(f, "Success"),
            Self::Failed => write!(f, "Failed"),
            Self::RolledBack => write!(f, "Rolled Back"),
        }
    }
}

/// Restore validation result
#[derive(Debug, Clone, Default)]
pub struct RestoreValidation {
    /// Is valid
    pub valid: bool,
    /// Warnings
    pub warnings: Vec<String>,
    /// Errors
    pub errors: Vec<String>,
    /// Version compatible
    pub version_compatible: bool,
    /// Categories to restore
    pub categories: Vec<SettingsCategory>,
}

impl RestoreValidation {
    /// Create valid result
    pub fn valid() -> Self {
        Self {
            valid: true,
            version_compatible: true,
            ..Default::default()
        }
    }

    /// Create invalid result
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![error.into()],
            ..Default::default()
        }
    }

    /// Add warning
    pub fn warn(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add error
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.valid = false;
        self
    }

    /// Set categories
    pub fn with_categories(mut self, categories: Vec<SettingsCategory>) -> Self {
        self.categories = categories;
        self
    }
}

/// Restore point - snapshot before restore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePoint {
    /// ID
    pub id: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Settings snapshot
    pub settings: UnifiedSettings,
    /// Description
    pub description: String,
}

impl RestorePoint {
    /// Create new restore point
    pub fn new(id: u64, settings: &UnifiedSettings, description: impl Into<String>) -> Self {
        Self {
            id,
            timestamp: chrono::Utc::now(),
            settings: settings.clone(),
            description: description.into(),
        }
    }
}

/// Restore operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRecord {
    /// Record ID
    pub id: u64,
    /// Backup ID restored from
    pub backup_id: u64,
    /// Restore mode
    pub mode: RestoreMode,
    /// Status
    pub status: RestoreStatus,
    /// Started timestamp
    pub started: chrono::DateTime<chrono::Utc>,
    /// Completed timestamp
    pub completed: Option<chrono::DateTime<chrono::Utc>>,
    /// Categories restored
    pub categories: Vec<SettingsCategory>,
    /// Restore point ID (for rollback)
    pub restore_point_id: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
}

impl RestoreRecord {
    /// Create new record
    pub fn new(id: u64, backup_id: u64, mode: RestoreMode) -> Self {
        Self {
            id,
            backup_id,
            mode,
            status: RestoreStatus::Pending,
            started: chrono::Utc::now(),
            completed: None,
            categories: Vec::new(),
            restore_point_id: None,
            error: None,
        }
    }

    /// Mark as in progress
    pub fn start(&mut self) {
        self.status = RestoreStatus::InProgress;
    }

    /// Mark as successful
    pub fn success(&mut self) {
        self.status = RestoreStatus::Success;
        self.completed = Some(chrono::Utc::now());
    }

    /// Mark as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = RestoreStatus::Failed;
        self.error = Some(error.into());
        self.completed = Some(chrono::Utc::now());
    }

    /// Mark as rolled back
    pub fn rollback(&mut self) {
        self.status = RestoreStatus::RolledBack;
        self.completed = Some(chrono::Utc::now());
    }

    /// Duration of restore
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed.map(|c| c - self.started)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_mode_display() {
        assert_eq!(format!("{}", RestoreMode::Full), "Full");
        assert_eq!(format!("{}", RestoreMode::Merge), "Merge");
    }

    #[test]
    fn test_restore_status_display() {
        assert_eq!(format!("{}", RestoreStatus::Success), "Success");
        assert_eq!(format!("{}", RestoreStatus::Failed), "Failed");
    }

    #[test]
    fn test_restore_validation_valid() {
        let v = RestoreValidation::valid();
        assert!(v.valid);
    }

    #[test]
    fn test_restore_validation_invalid() {
        let v = RestoreValidation::invalid("Test error");
        assert!(!v.valid);
        assert_eq!(v.errors.len(), 1);
    }

    #[test]
    fn test_restore_point_new() {
        let settings = UnifiedSettings::default();
        let point = RestorePoint::new(1, &settings, "Test point");
        assert_eq!(point.id, 1);
    }

    #[test]
    fn test_restore_record_new() {
        let record = RestoreRecord::new(1, 10, RestoreMode::Full);
        assert_eq!(record.id, 1);
        assert_eq!(record.backup_id, 10);
        assert_eq!(record.status, RestoreStatus::Pending);
    }

    #[test]
    fn test_restore_record_lifecycle() {
        let mut record = RestoreRecord::new(1, 10, RestoreMode::Full);
        record.start();
        assert_eq!(record.status, RestoreStatus::InProgress);
        record.success();
        assert_eq!(record.status, RestoreStatus::Success);
        assert!(record.completed.is_some());
    }
}
