// v0.0.576: Settings Restore Helpers
// Helper functions for restore operations

use super::manager::RestoreManager;

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
            "  [{}] {} - {} (backup #{})\n",
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
