//! Tests for backup history module

#![cfg(test)]

use super::formatting::*;
use super::storage::BackupHistory;
use super::types::{BackupRecord, BackupStatus, BackupType};
use super::utils::*;

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
