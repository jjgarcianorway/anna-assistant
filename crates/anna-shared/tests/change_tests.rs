//! Tests for the safe change engine (v0.0.27).

use anna_shared::change::{
    apply_change, plan_ensure_line, rollback, ChangeResult,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_ensure_line_appends_when_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.conf");

    // Create file without the target line
    fs::write(&path, "existing content\n").unwrap();

    let plan = plan_ensure_line(&path, "new line").unwrap();
    assert!(!plan.is_noop);

    let result = apply_change(&plan);
    assert!(result.applied);
    assert!(!result.was_noop);

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("new line"));
}

#[test]
fn test_ensure_line_noop_when_present() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.conf");

    // Create file WITH the target line
    fs::write(&path, "existing content\nnew line\n").unwrap();

    let plan = plan_ensure_line(&path, "new line").unwrap();
    assert!(plan.is_noop);

    let result = apply_change(&plan);
    assert!(!result.applied);
    assert!(result.was_noop);
}

#[test]
fn test_backup_created_before_edit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.conf");
    fs::write(&path, "original content\n").unwrap();

    let plan = plan_ensure_line(&path, "new line").unwrap();
    let result = apply_change(&plan);

    assert!(result.applied);
    assert!(result.backup_path.is_some());
    assert!(result.backup_path.unwrap().exists());
}

#[test]
fn test_rollback_restores_original() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.conf");
    let original = "original content\n";
    fs::write(&path, original).unwrap();

    let plan = plan_ensure_line(&path, "new line").unwrap();
    let apply_result = apply_change(&plan);
    assert!(apply_result.applied);

    // Verify content changed
    let after_change = fs::read_to_string(&path).unwrap();
    assert!(after_change.contains("new line"));

    // Rollback
    let rollback_result = rollback(&plan);
    assert!(rollback_result.applied);

    // Verify original restored
    let after_rollback = fs::read_to_string(&path).unwrap();
    assert_eq!(after_rollback, original);
}

#[test]
fn test_create_file_if_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("new.conf");

    assert!(!path.exists());

    let plan = plan_ensure_line(&path, "first line").unwrap();
    assert!(!plan.target_exists);
    assert!(!plan.is_noop);

    let result = apply_change(&plan);
    assert!(result.applied);
    assert!(path.exists());

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("first line"));
}

#[test]
fn test_change_result_types() {
    let success = ChangeResult::success(PathBuf::from("/backup"));
    assert!(success.applied);
    assert!(!success.was_noop);
    assert!(success.error.is_none());

    let noop = ChangeResult::noop();
    assert!(!noop.applied);
    assert!(noop.was_noop);

    let failed = ChangeResult::failed("test error");
    assert!(!failed.applied);
    assert_eq!(failed.error, Some("test error".to_string()));
}

#[test]
fn test_change_result_diagnostics() {
    let result = ChangeResult::noop().with_diagnostic("test diagnostic");
    assert_eq!(result.diagnostics, vec!["Line already present, no change needed", "test diagnostic"]);
}

#[test]
fn test_plan_summary() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.conf");
    fs::write(&path, "existing\n").unwrap();

    let plan = plan_ensure_line(&path, "new line").unwrap();
    let summary = plan.summary();
    assert!(summary.contains("backup"));
    assert!(!summary.contains("No change needed"));

    // Now test noop summary
    fs::write(&path, "new line\n").unwrap();
    let plan_noop = plan_ensure_line(&path, "new line").unwrap();
    let summary_noop = plan_noop.summary();
    assert!(summary_noop.contains("No change needed"));
}
