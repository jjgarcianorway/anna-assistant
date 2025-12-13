//! Backup History Tracking - Phase 79
//!
//! Tracks backups created by Anna when making changes to files/configs.
//! Critical for the undo/rollback functionality mentioned in VISION.md.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Backup status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupStatus {
    Active,
    Restored,
    Expired,
    Deleted,
}

impl BackupStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            BackupStatus::Active => "+",
            BackupStatus::Restored => "R",
            BackupStatus::Expired => "x",
            BackupStatus::Deleted => "-",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BackupStatus::Active => "available",
            BackupStatus::Restored => "was restored",
            BackupStatus::Expired => "expired",
            BackupStatus::Deleted => "deleted",
        }
    }
}

/// Type of backup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackupType {
    ConfigFile,
    SystemFile,
    UserFile,
    Database,
    Package,
    Service,
}

impl BackupType {
    pub fn description(&self) -> &'static str {
        match self {
            BackupType::ConfigFile => "configuration file",
            BackupType::SystemFile => "system file",
            BackupType::UserFile => "user file",
            BackupType::Database => "database",
            BackupType::Package => "package state",
            BackupType::Service => "service config",
        }
    }
}

/// A single backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    /// Unique backup ID
    pub id: String,
    /// Original file path
    pub original_path: String,
    /// Backup file path
    pub backup_path: String,
    /// Type of backup
    pub backup_type: BackupType,
    /// Current status
    pub status: BackupStatus,
    /// Size in bytes
    pub size_bytes: u64,
    /// Timestamp when created
    pub created_at: u64,
    /// Associated ticket/change ID
    pub change_id: Option<String>,
    /// Description of what change was made
    pub change_description: Option<String>,
    /// Expiration timestamp (if any)
    pub expires_at: Option<u64>,
    /// Whether restored
    pub restored_at: Option<u64>,
}

impl BackupRecord {
    /// Check if backup is still available for restore
    pub fn is_restorable(&self) -> bool {
        self.status == BackupStatus::Active
    }

    /// Check if expired
    pub fn is_expired(&self, now: u64) -> bool {
        match self.expires_at {
            Some(exp) => now > exp,
            None => false,
        }
    }

    /// Age in seconds
    pub fn age_secs(&self, now: u64) -> u64 {
        now.saturating_sub(self.created_at)
    }
}

/// Backup history tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupHistory {
    /// All backup records
    pub records: Vec<BackupRecord>,
    /// Count by type
    pub by_type: HashMap<String, u64>,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Number of restores performed
    pub restore_count: u64,
}

impl BackupHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a backup record
    pub fn add(&mut self, record: BackupRecord) {
        let type_key = format!("{:?}", record.backup_type);
        *self.by_type.entry(type_key).or_insert(0) += 1;
        self.total_size_bytes += record.size_bytes;
        self.records.push(record);
    }

    /// Get backup by ID
    pub fn get(&self, id: &str) -> Option<&BackupRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get mutable backup by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut BackupRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    /// Mark a backup as restored
    pub fn mark_restored(&mut self, id: &str) -> bool {
        if let Some(record) = self.get_mut(id) {
            record.status = BackupStatus::Restored;
            record.restored_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            );
            self.restore_count += 1;
            true
        } else {
            false
        }
    }

    /// Mark a backup as deleted
    pub fn mark_deleted(&mut self, id: &str) -> bool {
        // Find the index and size first to avoid borrow issues
        let found = self.records.iter().position(|r| r.id == id);
        if let Some(idx) = found {
            let size = self.records[idx].size_bytes;
            self.total_size_bytes = self.total_size_bytes.saturating_sub(size);
            self.records[idx].status = BackupStatus::Deleted;
            true
        } else {
            false
        }
    }

    /// Get active (restorable) backups
    pub fn active(&self) -> Vec<&BackupRecord> {
        self.records.iter().filter(|r| r.is_restorable()).collect()
    }

    /// Get recent backups
    pub fn recent(&self, limit: usize) -> Vec<&BackupRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get backups by type
    pub fn by_backup_type(&self, backup_type: BackupType) -> Vec<&BackupRecord> {
        self.records
            .iter()
            .filter(|r| r.backup_type == backup_type)
            .collect()
    }

    /// Get backups for a specific file
    pub fn for_file(&self, path: &str) -> Vec<&BackupRecord> {
        self.records
            .iter()
            .filter(|r| r.original_path == path)
            .collect()
    }

    /// Get backups for a change ID
    pub fn for_change(&self, change_id: &str) -> Vec<&BackupRecord> {
        self.records
            .iter()
            .filter(|r| r.change_id.as_deref() == Some(change_id))
            .collect()
    }

    /// Total backup count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Active backup count
    pub fn active_count(&self) -> usize {
        self.active().len()
    }

    /// Expire old backups
    pub fn expire_old(&mut self, now: u64) -> usize {
        let mut count = 0;
        for record in &mut self.records {
            if record.status == BackupStatus::Active && record.is_expired(now) {
                record.status = BackupStatus::Expired;
                count += 1;
            }
        }
        count
    }

    /// Calculate total active size
    pub fn active_size_bytes(&self) -> u64 {
        self.active().iter().map(|r| r.size_bytes).sum()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_backup(id: &str, path: &str, size: u64) -> BackupRecord {
        BackupRecord {
            id: id.to_string(),
            original_path: path.to_string(),
            backup_path: format!("{}.bak", path),
            backup_type: BackupType::ConfigFile,
            status: BackupStatus::Active,
            size_bytes: size,
            created_at: 1234567890,
            change_id: Some("CHG-001".to_string()),
            change_description: Some("Test change".to_string()),
            expires_at: None,
            restored_at: None,
        }
    }

    #[test]
    fn test_backup_status() {
        assert_eq!(BackupStatus::Active.symbol(), "+");
        assert_eq!(BackupStatus::Restored.description(), "was restored");
    }

    #[test]
    fn test_backup_type() {
        assert_eq!(BackupType::ConfigFile.description(), "configuration file");
    }

    #[test]
    fn test_backup_record_restorable() {
        let backup = make_backup("B001", "/etc/test.conf", 1024);
        assert!(backup.is_restorable());
    }

    #[test]
    fn test_backup_record_expired() {
        let mut backup = make_backup("B001", "/etc/test.conf", 1024);
        backup.expires_at = Some(1000);
        assert!(backup.is_expired(2000));
        assert!(!backup.is_expired(500));
    }

    #[test]
    fn test_backup_history_add() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));

        assert_eq!(history.total_count(), 1);
        assert_eq!(history.total_size_bytes, 1024);
    }

    #[test]
    fn test_backup_history_get() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));

        assert!(history.get("B001").is_some());
        assert!(history.get("B999").is_none());
    }

    #[test]
    fn test_mark_restored() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));

        assert!(history.mark_restored("B001"));
        assert_eq!(history.restore_count, 1);
        assert_eq!(history.get("B001").unwrap().status, BackupStatus::Restored);
    }

    #[test]
    fn test_mark_deleted() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));

        assert!(history.mark_deleted("B001"));
        assert_eq!(history.total_size_bytes, 0);
        assert_eq!(history.get("B001").unwrap().status, BackupStatus::Deleted);
    }

    #[test]
    fn test_active_backups() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test1.conf", 1024));
        history.add(make_backup("B002", "/etc/test2.conf", 2048));
        history.mark_deleted("B001");

        assert_eq!(history.active().len(), 1);
        assert_eq!(history.active_count(), 1);
    }

    #[test]
    fn test_for_file() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));
        history.add(make_backup("B002", "/etc/test.conf", 2048));
        history.add(make_backup("B003", "/etc/other.conf", 512));

        let backups = history.for_file("/etc/test.conf");
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(2048), "2.0 KB");
        assert_eq!(format_size(1572864), "1.5 MB");
        assert_eq!(format_size(1610612736), "1.5 GB");
    }

    #[test]
    fn test_format_backup_history() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));

        let output = format_backup_history(&history);
        assert!(output.contains("Backup History"));
        assert!(output.contains("Total backups: 1"));
    }

    #[test]
    fn test_is_backup_query() {
        assert!(is_backup_query("show my backups"));
        assert!(is_backup_query("can I restore the file?"));
        assert!(is_backup_query("undo the last change"));
        assert!(!is_backup_query("what is my disk space?"));
    }

    #[test]
    fn test_backup_fun_fact() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));

        let fact = backup_fun_fact(&history);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut history = BackupHistory::new();
        history.add(make_backup("B001", "/etc/test.conf", 1024));

        let compact = format_backup_history_compact(&history);
        assert!(compact.contains("Backups: 1"));

        let oneline = format_backup_history_oneline(&history);
        assert!(oneline.contains("1 backups"));
    }
}
