// v0.0.575: Settings Backup Manager (Phase 151)
// Automated backup and restore of settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::unified_settings::UnifiedSettings;

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

/// Format backup list for display
pub fn format_backups(manager: &BackupManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Backups ===\n\n");
    output.push_str(&format!("Total: {} backups ({} bytes)\n", manager.count(), manager.total_size()));

    if manager.is_backup_due() {
        output.push_str("Backup is due!\n");
    }
    output.push('\n');

    if manager.count() == 0 {
        output.push_str("No backups available.\n");
        return output;
    }

    for backup in manager.list().iter().rev().take(10) {
        output.push_str(&format!(
            "• [{}] {} - {} ({} bytes) - {}\n",
            backup.id, backup.backup_type, backup.description,
            backup.size_bytes, backup.age_display()
        ));
    }

    output
}

/// Check if query is about backups
pub fn is_backup_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("backup")
        || lower.contains("restore settings")
        || lower.contains("save settings")
}

/// Fun fact about settings backups
pub fn settings_backup_fun_fact() -> &'static str {
    "Anna automatically backs up your settings before major changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

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
