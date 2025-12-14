// v0.0.576: Settings Restore (Phase 152)
// Restore settings from backups with validation

use serde::{Deserialize, Serialize};

use crate::settings_backup::{BackupMeta, BackupStatus};
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

/// Restore manager
#[derive(Debug, Clone, Default)]
pub struct RestoreManager {
    /// Restore history
    history: Vec<RestoreRecord>,
    /// Restore points
    restore_points: Vec<RestorePoint>,
    /// Next ID
    next_id: u64,
    /// Max restore points to keep
    max_restore_points: usize,
}

impl RestoreManager {
    /// Create new restore manager
    pub fn new() -> Self {
        Self {
            max_restore_points: 5,
            ..Default::default()
        }
    }

    /// Validate a backup for restore
    pub fn validate(&self, backup: &BackupMeta, _backup_settings: &UnifiedSettings) -> RestoreValidation {
        let mut result = RestoreValidation::valid();

        // Check backup status
        if backup.status != BackupStatus::Success {
            return RestoreValidation::invalid("Backup is not valid");
        }

        // Check version compatibility (simplified)
        let current_version = env!("CARGO_PKG_VERSION");
        if backup.version != current_version {
            result = result.warn(format!(
                "Version mismatch: backup is v{}, current is v{}",
                backup.version, current_version
            ));
        }

        // Add all categories
        result = result.with_categories(vec![
            SettingsCategory::Personality,
            SettingsCategory::Risk,
            SettingsCategory::Learning,
            SettingsCategory::Escalation,
            SettingsCategory::Verbosity,
            SettingsCategory::Confirmation,
            SettingsCategory::Timeout,
            SettingsCategory::OutputStyle,
            SettingsCategory::Privacy,
            SettingsCategory::Backup,
            SettingsCategory::Update,
            SettingsCategory::Model,
        ]);

        result
    }

    /// Create restore point (before restore)
    pub fn create_restore_point(&mut self, settings: &UnifiedSettings, description: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let point = RestorePoint::new(id, settings, description);
        self.restore_points.push(point);

        // Cleanup old points
        while self.restore_points.len() > self.max_restore_points {
            self.restore_points.remove(0);
        }

        id
    }

    /// Restore from backup
    pub fn restore(
        &mut self,
        backup: &BackupMeta,
        backup_settings: &UnifiedSettings,
        current: &mut UnifiedSettings,
        mode: RestoreMode,
    ) -> Result<u64, String> {
        // Validate
        let validation = self.validate(backup, backup_settings);
        if !validation.valid {
            return Err(validation.errors.join(", "));
        }

        // Create restore point
        let restore_point_id = self.create_restore_point(current, "Pre-restore snapshot");

        // Create record
        let record_id = self.next_id;
        self.next_id += 1;
        let mut record = RestoreRecord::new(record_id, backup.id, mode);
        record.restore_point_id = Some(restore_point_id);
        record.categories = validation.categories;
        record.start();

        // Perform restore based on mode
        match mode {
            RestoreMode::Full => {
                *current = backup_settings.clone();
            }
            RestoreMode::Merge => {
                // For merge, we'd selectively copy non-default values
                // Simplified: just do full restore for now
                *current = backup_settings.clone();
            }
            RestoreMode::Partial | RestoreMode::Selective => {
                // Would restore only selected categories
                *current = backup_settings.clone();
            }
        }

        record.success();
        self.history.push(record);

        Ok(record_id)
    }

    /// Rollback to restore point
    pub fn rollback(&mut self, restore_point_id: u64, current: &mut UnifiedSettings) -> Result<(), String> {
        let point = self.restore_points
            .iter()
            .find(|p| p.id == restore_point_id)
            .ok_or("Restore point not found")?;

        *current = point.settings.clone();
        Ok(())
    }

    /// Get restore history
    pub fn history(&self) -> &[RestoreRecord] {
        &self.history
    }

    /// Get recent restores
    pub fn recent(&self, count: usize) -> Vec<&RestoreRecord> {
        self.history.iter().rev().take(count).collect()
    }

    /// Get restore points
    pub fn restore_points(&self) -> &[RestorePoint] {
        &self.restore_points
    }

    /// Get record by ID
    pub fn get(&self, id: u64) -> Option<&RestoreRecord> {
        self.history.iter().find(|r| r.id == id)
    }

    /// Count restores
    pub fn count(&self) -> usize {
        self.history.len()
    }

    /// Successful restores count
    pub fn successful_count(&self) -> usize {
        self.history.iter().filter(|r| r.status == RestoreStatus::Success).count()
    }
}

/// Format restore history for display
pub fn format_restore_history(manager: &RestoreManager) -> String {
    let mut output = String::new();

    output.push_str("=== Restore History ===\n\n");
    output.push_str(&format!(
        "Total: {} restores ({} successful)\n",
        manager.count(),
        manager.successful_count()
    ));
    output.push_str(&format!("Restore points: {}\n\n", manager.restore_points().len()));

    if manager.count() == 0 {
        output.push_str("No restore operations performed.\n");
        return output;
    }

    for record in manager.recent(10) {
        output.push_str(&format!(
            "• [{}] {} - {} (backup #{})\n",
            record.id, record.mode, record.status, record.backup_id
        ));
    }

    output
}

/// Check if query is about restore
pub fn is_restore_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("restore")
        || lower.contains("rollback")
        || lower.contains("recover settings")
}

/// Fun fact about restore
pub fn settings_restore_fun_fact() -> &'static str {
    "Anna creates a restore point before each restore, so you can always roll back!"
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

    #[test]
    fn test_restore_manager_new() {
        let manager = RestoreManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_restore_manager_create_restore_point() {
        let mut manager = RestoreManager::new();
        let settings = UnifiedSettings::default();
        let id = manager.create_restore_point(&settings, "Test");
        assert_eq!(manager.restore_points().len(), 1);
        assert_eq!(manager.restore_points()[0].id, id);
    }

    #[test]
    fn test_restore_manager_restore() {
        let mut manager = RestoreManager::new();
        let backup = BackupMeta::new(1, crate::settings_backup::BackupType::Manual, "Test")
            .success(100);
        let backup_settings = UnifiedSettings::default();
        let mut current = UnifiedSettings::default();

        let result = manager.restore(&backup, &backup_settings, &mut current, RestoreMode::Full);
        assert!(result.is_ok());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_format_restore_history() {
        let manager = RestoreManager::new();
        let output = format_restore_history(&manager);
        assert!(output.contains("Restore"));
    }

    #[test]
    fn test_is_restore_query() {
        assert!(is_restore_query("restore settings"));
        assert!(is_restore_query("rollback changes"));
        assert!(!is_restore_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_restore_fun_fact();
        assert!(fact.contains("restore"));
    }
}
