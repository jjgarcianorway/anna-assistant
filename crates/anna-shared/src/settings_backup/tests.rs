// v0.0.575: Backup Tests (Phase 151)

#[cfg(test)]
mod tests {
    use crate::unified_settings::UnifiedSettings;

    use super::super::manager::BackupManager;
    use super::super::types::{BackupConfig, BackupMeta, BackupStatus, BackupType};
    use super::super::utils::{format_backups, is_backup_query, settings_backup_fun_fact};

    #[test]
    fn test_backup_type_display() {
        assert_eq!(format!("{}", BackupType::Full), "Full");
        assert_eq!(format!("{}", BackupType::Manual), "Manual");
    }

    #[test]
    fn test_backup_status_display() {
        assert_eq!(format!("{}", BackupStatus::Success), "Success");
        assert_eq!(format!("{}", BackupStatus::Failed), "Failed");
    }

    #[test]
    fn test_backup_meta_new() {
        let meta = BackupMeta::new(1, BackupType::Full, "Test backup");
        assert_eq!(meta.id, 1);
        assert_eq!(meta.status, BackupStatus::InProgress);
    }

    #[test]
    fn test_backup_meta_success() {
        let meta = BackupMeta::new(1, BackupType::Full, "Test").success(1000);
        assert_eq!(meta.status, BackupStatus::Success);
        assert_eq!(meta.size_bytes, 1000);
    }

    #[test]
    fn test_backup_meta_is_valid() {
        let meta = BackupMeta::new(1, BackupType::Full, "Test").success(100);
        assert!(meta.is_valid());
    }

    #[test]
    fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert!(config.auto_backup);
        assert_eq!(config.max_backups, 10);
    }

    #[test]
    fn test_backup_manager_new() {
        let manager = BackupManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_backup_manager_create_backup() {
        let mut manager = BackupManager::new();
        let settings = UnifiedSettings::default();
        let result = manager.create_backup(&settings, BackupType::Manual, "Test");
        assert!(result.is_ok());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_backup_manager_manual_backup() {
        let mut manager = BackupManager::new();
        let settings = UnifiedSettings::default();
        let result = manager.manual_backup(&settings, "Manual test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_backup_manager_is_backup_due() {
        let manager = BackupManager::new();
        assert!(manager.is_backup_due()); // No backups yet
    }

    #[test]
    fn test_backup_manager_cleanup() {
        let mut manager = BackupManager::new();
        manager.config.max_backups = 3;
        let settings = UnifiedSettings::default();
        for i in 0..5 {
            manager.create_backup(&settings, BackupType::Manual, &format!("Test {}", i)).ok();
        }
        assert_eq!(manager.count(), 3);
    }

    #[test]
    fn test_format_backups() {
        let manager = BackupManager::new();
        let output = format_backups(&manager);
        assert!(output.contains("Backups"));
    }

    #[test]
    fn test_is_backup_query() {
        assert!(is_backup_query("backup settings"));
        assert!(is_backup_query("restore settings"));
        assert!(!is_backup_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_backup_fun_fact();
        // The fact should contain "backs up" or similar
        assert!(fact.contains("backs up"), "Expected 'backs up' in: {}", fact);
    }
}
