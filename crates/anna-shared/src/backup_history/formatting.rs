//! Backup history formatting and display utilities

use super::storage::BackupHistory;
use super::types::BackupType;

/// Format size in human-readable form
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format backup history for display
pub fn format_backup_history(history: &BackupHistory) -> String {
    let mut lines = vec!["=== Backup History ===".to_string()];
    lines.push(String::new());

    if history.records.is_empty() {
        lines.push("No backups created yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total backups: {}", history.total_count()));
    lines.push(format!("Active backups: {}", history.active_count()));
    lines.push(format!("Total size: {}", format_size(history.total_size_bytes)));
    lines.push(format!(
        "Active size: {}",
        format_size(history.active_size_bytes())
    ));
    lines.push(format!("Restores performed: {}", history.restore_count));

    // By type
    if !history.by_type.is_empty() {
        lines.push(String::new());
        lines.push("By type:".to_string());
        for (type_name, count) in &history.by_type {
            lines.push(format!("  {}: {}", type_name, count));
        }
    }

    // Recent backups
    let recent = history.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent backups:".to_string());
        for backup in recent {
            let status = backup.status.symbol();
            lines.push(format!(
                "  [{}] {} ({})",
                status,
                backup.original_path,
                format_size(backup.size_bytes)
            ));
        }
    }

    lines.join("\n")
}

/// Format backup history compact
pub fn format_backup_history_compact(history: &BackupHistory) -> String {
    format!(
        "Backups: {} ({} active, {}) | Restores: {}",
        history.total_count(),
        history.active_count(),
        format_size(history.active_size_bytes()),
        history.restore_count
    )
}

/// Format backup history one-line
pub fn format_backup_history_oneline(history: &BackupHistory) -> String {
    format!(
        "{} backups ({})",
        history.active_count(),
        format_size(history.active_size_bytes())
    )
}

/// Generate fun fact about backups
pub fn backup_fun_fact(history: &BackupHistory) -> String {
    if history.records.is_empty() {
        return "No backups yet - Anna hasn't made any changes!".to_string();
    }

    let facts = [
        format!(
            "Anna has created {} backups totaling {}.",
            history.total_count(),
            format_size(history.total_size_bytes)
        ),
        format!(
            "{} backups are still available for restore.",
            history.active_count()
        ),
        format!(
            "Anna has performed {} successful restores.",
            history.restore_count
        ),
        {
            let config_count = history.by_backup_type(BackupType::ConfigFile).len();
            format!("{} configuration files have been backed up.", config_count)
        },
    ];

    facts[history.total_count() % facts.len()].clone()
}
