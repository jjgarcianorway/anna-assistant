// v0.0.575: Backup Manager (Phase 151)

use crate::unified_settings::UnifiedSettings;

use super::types::{BackupConfig, BackupMeta, BackupStatus, BackupType};

/// Backup manager
#[derive(Debug, Clone, Default)]
pub struct BackupManager {
    /// Configuration
    pub config: BackupConfig,
    /// Backup history
    backups: Vec<BackupMeta>,
    /// Next ID
    next_id: u64,
    /// Last backup time
    last_backup: Option<chrono::DateTime<chrono::Utc>>,
}

impl BackupManager {
    /// Create new backup manager
    pub fn new() -> Self {
        Self {
            config: BackupConfig::default(),
            ..Default::default()
        }
    }

    /// Create with custom config
    pub fn with_config(config: BackupConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Create a backup
    pub fn create_backup(
        &mut self,
        settings: &UnifiedSettings,
        backup_type: BackupType,
        description: &str,
    ) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;

        let mut meta = BackupMeta::new(id, backup_type, description);

        // Serialize settings
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let size = content.len() as u64;

        // Generate simple checksum
        let checksum = format!("{:x}", content.len());

        meta = meta.success(size).with_checksum(checksum);
        self.last_backup = Some(chrono::Utc::now());

        self.backups.push(meta);
        self.cleanup_old_backups();

        Ok(id)
    }

    /// Create manual backup
    pub fn manual_backup(&mut self, settings: &UnifiedSettings, description: &str) -> Result<u64, String> {
        self.create_backup(settings, BackupType::Manual, description)
    }

    /// Create pre-change backup
    pub fn pre_change_backup(&mut self, settings: &UnifiedSettings) -> Result<u64, String> {
        if !self.config.backup_before_changes {
            return Err("Pre-change backups disabled".to_string());
        }
        self.create_backup(settings, BackupType::PreChange, "Automatic pre-change backup")
    }

    /// Create scheduled backup
    pub fn scheduled_backup(&mut self, settings: &UnifiedSettings) -> Result<u64, String> {
        self.create_backup(settings, BackupType::Scheduled, "Scheduled automatic backup")
    }

    /// Check if backup is due
    pub fn is_backup_due(&self) -> bool {
        if !self.config.auto_backup {
            return false;
        }

        match self.last_backup {
            Some(last) => {
                let hours_since = (chrono::Utc::now() - last).num_hours();
                hours_since >= self.config.interval_hours as i64
            }
            None => true,
        }
    }

    /// Get backup by ID
    pub fn get(&self, id: u64) -> Option<&BackupMeta> {
        self.backups.iter().find(|b| b.id == id)
    }

    /// List all backups
    pub fn list(&self) -> &[BackupMeta] {
        &self.backups
    }

    /// List valid backups
    pub fn valid_backups(&self) -> Vec<&BackupMeta> {
        self.backups.iter().filter(|b| b.is_valid()).collect()
    }

    /// Get latest backup
    pub fn latest(&self) -> Option<&BackupMeta> {
        self.valid_backups().first().copied()
    }

    /// Get backups by type
    pub fn by_type(&self, backup_type: BackupType) -> Vec<&BackupMeta> {
        self.backups.iter().filter(|b| b.backup_type == backup_type).collect()
    }

    /// Delete a backup
    pub fn delete(&mut self, id: u64) -> Option<BackupMeta> {
        if let Some(pos) = self.backups.iter().position(|b| b.id == id) {
            Some(self.backups.remove(pos))
        } else {
            None
        }
    }

    /// Cleanup old backups
    fn cleanup_old_backups(&mut self) {
        while self.backups.len() > self.config.max_backups {
            // Remove oldest
            self.backups.remove(0);
        }
    }

    /// Total backup size
    pub fn total_size(&self) -> u64 {
        self.backups.iter().map(|b| b.size_bytes).sum()
    }

    /// Count backups
    pub fn count(&self) -> usize {
        self.backups.len()
    }
}
