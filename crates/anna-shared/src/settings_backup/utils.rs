// v0.0.575: Backup Utilities (Phase 151)

use super::manager::BackupManager;

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
