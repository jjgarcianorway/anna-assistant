// v0.0.556: Settings Migration - Utility Functions
// Helper functions for migration operations

use super::types::{MigrationResult, CURRENT_SCHEMA_VERSION};

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
