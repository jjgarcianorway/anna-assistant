// v0.0.556: Settings Migration Module
// Handles migrating settings between versions

pub mod migrator;
pub mod types;
pub mod utils;

// Re-export all public items to preserve the original API
pub use migrator::{migrate_and_save, SettingsMigrator};
pub use types::{
    MigrationRecord, MigrationResult, MigrationStatus, VersionedSettings, CURRENT_SCHEMA_VERSION,
};
pub use utils::{check_schema_version, format_migration_status, settings_migration_fun_fact};
