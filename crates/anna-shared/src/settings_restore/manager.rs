// v0.0.576: Settings Restore Manager
// Manages restore operations and restore points

use crate::settings_backup::{BackupMeta, BackupStatus};
use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::types::{RestoreMode, RestorePoint, RestoreRecord, RestoreStatus, RestoreValidation};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_backup::BackupType;

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
        let backup = BackupMeta::new(1, BackupType::Manual, "Test").success(100);
        let backup_settings = UnifiedSettings::default();
        let mut current = UnifiedSettings::default();

        let result = manager.restore(&backup, &backup_settings, &mut current, RestoreMode::Full);
        assert!(result.is_ok());
        assert_eq!(manager.count(), 1);
    }
}
