// v0.0.556: Settings Migration (Phase 132)
// Handles migrating settings between versions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::settings_persistence::{SettingsError, SettingsPersistence, SettingsResult};
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

/// Settings migrator
#[derive(Debug, Clone)]
pub struct SettingsMigrator {
    /// Create backup before migration
    pub backup_before_migrate: bool,
    /// Dry run mode (don't apply changes)
    pub dry_run: bool,
}

impl Default for SettingsMigrator {
    fn default() -> Self {
        Self {
            backup_before_migrate: true,
            dry_run: false,
        }
    }
}

impl SettingsMigrator {
    /// Create new migrator
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable dry run mode
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Disable backup before migration
    pub fn no_backup(mut self) -> Self {
        self.backup_before_migrate = false;
        self
    }

    /// Migrate versioned settings to current version
    pub fn migrate(&self, settings: &mut VersionedSettings) -> MigrationResult {
        if settings.is_current() {
            return MigrationResult::up_to_date();
        }

        let from_version = settings.schema_version;
        let mut changes = vec![];

        // Apply migrations in order
        let mut current = from_version;
        while current < CURRENT_SCHEMA_VERSION {
            match self.apply_migration(current, settings) {
                Ok(migration_changes) => {
                    changes.extend(migration_changes);
                    current += 1;
                }
                Err(e) => {
                    return MigrationResult::failed(from_version, &e);
                }
            }
        }

        if !self.dry_run {
            settings.record_migration(from_version, CURRENT_SCHEMA_VERSION, &changes);
        }

        MigrationResult::migrated(from_version, CURRENT_SCHEMA_VERSION, changes)
    }

    /// Apply a single version migration
    fn apply_migration(
        &self,
        from_version: u32,
        _settings: &mut VersionedSettings,
    ) -> Result<Vec<String>, String> {
        match from_version {
            0 => {
                // Version 0 -> 1: Initial schema
                Ok(vec!["Initialized settings schema".to_string()])
            }
            _ => {
                // Unknown version
                Err(format!("Unknown migration from version {}", from_version))
            }
        }
    }

    /// Migrate from legacy settings (pre-versioned)
    pub fn migrate_legacy(&self, legacy: HashMap<String, serde_json::Value>) -> VersionedSettings {
        let mut settings = UnifiedSettings::default();
        let mut changes = vec![];

        // Try to extract known fields from legacy format
        if let Some(val) = legacy.get("personality") {
            if let Ok(personality) = serde_json::from_value(val.clone()) {
                settings.personality = personality;
                changes.push("Migrated personality settings".to_string());
            }
        }

        if let Some(val) = legacy.get("risk") {
            if let Ok(risk) = serde_json::from_value(val.clone()) {
                settings.risk = risk;
                changes.push("Migrated risk settings".to_string());
            }
        }

        if let Some(val) = legacy.get("learning") {
            if let Ok(learning) = serde_json::from_value(val.clone()) {
                settings.learning = learning;
                changes.push("Migrated learning settings".to_string());
            }
        }

        let mut versioned = VersionedSettings::from_settings(settings);
        versioned.record_migration(0, CURRENT_SCHEMA_VERSION, &changes);
        versioned
    }

    /// Check if migration is needed
    pub fn needs_migration(settings: &VersionedSettings) -> bool {
        settings.schema_version < CURRENT_SCHEMA_VERSION
    }

    /// Get migration path description
    pub fn migration_path(from: u32, to: u32) -> Vec<String> {
        let mut path = vec![];
        for v in from..to {
            path.push(format!("v{} -> v{}", v, v + 1));
        }
        path
    }
}

/// Migrate persistence and save
pub fn migrate_and_save(persistence: &mut SettingsPersistence) -> SettingsResult<MigrationResult> {
    let mut versioned = VersionedSettings::from_settings(persistence.settings.clone());

    let migrator = SettingsMigrator::new();
    if migrator.backup_before_migrate {
        persistence.create_backup()?;
    }

    let result = migrator.migrate(&mut versioned);

    if result.is_success() {
        persistence.settings = versioned.settings;
        persistence.save()?;
    }

    Ok(result)
}

/// Check current schema version
pub fn check_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

/// Format migration status for display
pub fn format_migration_status(result: &MigrationResult) -> String {
    let mut output = String::new();

    output.push_str(&format!("Migration Status: {}\n", result.status));
    output.push_str(&format!(
        "Version: {} -> {}\n",
        result.from_version, result.to_version
    ));

    if !result.changes.is_empty() {
        output.push_str("\nChanges:\n");
        for change in &result.changes {
            output.push_str(&format!("  - {}\n", change));
        }
    }

    if !result.warnings.is_empty() {
        output.push_str("\nWarnings:\n");
        for warning in &result.warnings {
            output.push_str(&format!("  ! {}\n", warning));
        }
    }

    output
}

/// Fun fact about settings migration
pub fn settings_migration_fun_fact() -> &'static str {
    "Anna automatically migrates your settings when the schema changes - no manual intervention needed!"
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
    fn test_migrator_default() {
        let migrator = SettingsMigrator::default();
        assert!(migrator.backup_before_migrate);
        assert!(!migrator.dry_run);
    }

    #[test]
    fn test_migrator_dry_run() {
        let migrator = SettingsMigrator::new().dry_run();
        assert!(migrator.dry_run);
    }

    #[test]
    fn test_migrate_current_version() {
        let migrator = SettingsMigrator::new();
        let mut settings = VersionedSettings::default();
        let result = migrator.migrate(&mut settings);
        assert!(result.is_success());
        assert_eq!(result.status, MigrationStatus::UpToDate);
    }

    #[test]
    fn test_needs_migration() {
        let current = VersionedSettings::default();
        assert!(!SettingsMigrator::needs_migration(&current));

        let mut old = VersionedSettings::default();
        old.schema_version = 0;
        assert!(SettingsMigrator::needs_migration(&old));
    }

    #[test]
    fn test_migration_path() {
        let path = SettingsMigrator::migration_path(0, 3);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "v0 -> v1");
    }

    #[test]
    fn test_check_schema_version() {
        assert_eq!(check_schema_version(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_format_migration_status() {
        let result = MigrationResult::up_to_date();
        let output = format_migration_status(&result);
        assert!(output.contains("Up to date"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_migration_fun_fact();
        assert!(fact.contains("migrat"));
    }
}
