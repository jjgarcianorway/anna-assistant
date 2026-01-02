// v0.0.556: Settings Migration - Type Definitions
// Contains enums, structs, and constants for settings migration

use serde::{Deserialize, Serialize};

use crate::unified_settings::UnifiedSettings;

/// Current settings schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Migration status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    /// No migration needed
    UpToDate,
    /// Migration successful
    Migrated,
    /// Migration failed (with reason)
    Failed,
    /// Unknown schema version
    UnknownVersion,
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UpToDate => write!(f, "Up to date"),
            Self::Migrated => write!(f, "Migrated"),
            Self::Failed => write!(f, "Failed"),
            Self::UnknownVersion => write!(f, "Unknown version"),
        }
    }
}

/// Migration result with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Status of migration
    pub status: MigrationStatus,
    /// From version
    pub from_version: u32,
    /// To version
    pub to_version: u32,
    /// Changes made
    pub changes: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl MigrationResult {
    /// Create a new up-to-date result
    pub fn up_to_date() -> Self {
        Self {
            status: MigrationStatus::UpToDate,
            from_version: CURRENT_SCHEMA_VERSION,
            to_version: CURRENT_SCHEMA_VERSION,
            changes: vec![],
            warnings: vec![],
        }
    }

    /// Create a migrated result
    pub fn migrated(from: u32, to: u32, changes: Vec<String>) -> Self {
        Self {
            status: MigrationStatus::Migrated,
            from_version: from,
            to_version: to,
            changes,
            warnings: vec![],
        }
    }

    /// Create a failed result
    pub fn failed(from: u32, reason: &str) -> Self {
        Self {
            status: MigrationStatus::Failed,
            from_version: from,
            to_version: CURRENT_SCHEMA_VERSION,
            changes: vec![],
            warnings: vec![reason.to_string()],
        }
    }

    /// Was migration successful?
    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            MigrationStatus::UpToDate | MigrationStatus::Migrated
        )
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

/// Settings with version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedSettings {
    /// Schema version
    pub schema_version: u32,
    /// When settings were last modified
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// The actual settings
    pub settings: UnifiedSettings,
    /// Migration history
    pub migrations: Vec<MigrationRecord>,
}

impl Default for VersionedSettings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            last_modified: chrono::Utc::now(),
            settings: UnifiedSettings::default(),
            migrations: vec![],
        }
    }
}

impl VersionedSettings {
    /// Create new versioned settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from existing settings
    pub fn from_settings(settings: UnifiedSettings) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            last_modified: chrono::Utc::now(),
            settings,
            migrations: vec![],
        }
    }

    /// Is this the current schema version?
    pub fn is_current(&self) -> bool {
        self.schema_version == CURRENT_SCHEMA_VERSION
    }

    /// Record a migration
    pub fn record_migration(&mut self, from: u32, to: u32, changes: &[String]) {
        self.migrations.push(MigrationRecord {
            from_version: from,
            to_version: to,
            migrated_at: chrono::Utc::now(),
            changes: changes.to_vec(),
        });
        self.schema_version = to;
        self.last_modified = chrono::Utc::now();
    }
}

/// Record of a migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// From version
    pub from_version: u32,
    /// To version
    pub to_version: u32,
    /// When migration occurred
    pub migrated_at: chrono::DateTime<chrono::Utc>,
    /// Changes made
    pub changes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_status_display() {
        assert_eq!(format!("{}", MigrationStatus::UpToDate), "Up to date");
        assert_eq!(format!("{}", MigrationStatus::Migrated), "Migrated");
        assert_eq!(format!("{}", MigrationStatus::Failed), "Failed");
    }

    #[test]
    fn test_migration_result_up_to_date() {
        let result = MigrationResult::up_to_date();
        assert!(result.is_success());
        assert_eq!(result.from_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_migration_result_migrated() {
        let result = MigrationResult::migrated(0, 1, vec!["Test change".to_string()]);
        assert!(result.is_success());
        assert_eq!(result.from_version, 0);
        assert_eq!(result.to_version, 1);
    }

    #[test]
    fn test_migration_result_failed() {
        let result = MigrationResult::failed(0, "Test failure");
        assert!(!result.is_success());
    }

    #[test]
    fn test_versioned_settings_default() {
        let settings = VersionedSettings::default();
        assert_eq!(settings.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(settings.is_current());
    }

    #[test]
    fn test_versioned_settings_record_migration() {
        let mut settings = VersionedSettings::default();
        settings.schema_version = 0;
        settings.record_migration(0, 1, &["Test".to_string()]);
        assert_eq!(settings.schema_version, 1);
        assert_eq!(settings.migrations.len(), 1);
    }

    #[test]
    fn test_check_schema_version() {
        assert_eq!(super::CURRENT_SCHEMA_VERSION, 1);
    }
}
