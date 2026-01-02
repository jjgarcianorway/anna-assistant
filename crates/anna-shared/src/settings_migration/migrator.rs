// v0.0.556: Settings Migration - Migrator Logic
// Handles the actual migration process between schema versions

use std::collections::HashMap;

use crate::settings_persistence::{SettingsPersistence, SettingsResult};
use crate::unified_settings::UnifiedSettings;

use super::types::{
    MigrationResult, MigrationStatus, VersionedSettings, CURRENT_SCHEMA_VERSION,
};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
