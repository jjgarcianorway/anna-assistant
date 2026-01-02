//! Backup history utility functions

/// Check if query is about backups
pub fn is_backup_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "backup",
        "backups",
        "restore",
        "undo",
        "rollback",
        "revert",
        "saved copies",
        "previous version",
    ];
    keywords.iter().any(|k| q.contains(k))
}
