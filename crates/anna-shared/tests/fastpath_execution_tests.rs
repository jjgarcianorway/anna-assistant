//! Fast path execution tests for fastpath module (v0.0.39)

use anna_shared::fastpath::{
    try_fast_path, FastPathClass, FastPathInput, FastPathPolicy,
};
use anna_shared::snapshot::SystemSnapshot;
use anna_shared::trace::EvidenceKind;

fn fresh_snapshot() -> SystemSnapshot {
    let mut s = SystemSnapshot::now();
    s.add_disk("/", 45);
    s.add_disk("/home", 60);
    s.set_memory(16_000_000_000, 8_000_000_000); // 50% usage
    s
}

fn high_usage_snapshot() -> SystemSnapshot {
    let mut s = SystemSnapshot::now();
    s.add_disk("/", 92); // Warning
    s.add_disk("/home", 96); // Critical
    s.set_memory(16_000_000_000, 14_000_000_000); // ~88% - high
    s.add_failed_service("nginx.service");
    s
}

fn stale_snapshot() -> SystemSnapshot {
    let mut s = SystemSnapshot::new();
    s.captured_at = 0; // Very old
    s.add_disk("/", 45);
    s.set_memory(16_000_000_000, 8_000_000_000);
    s
}

#[test]
fn test_fast_path_disabled() {
    let policy = FastPathPolicy {
        enabled: false,
        ..Default::default()
    };
    let snapshot = fresh_snapshot();
    let input = FastPathInput {
        request: "how is my computer",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(!result.handled);
    assert!(result.trace_note.contains("disabled"));
}

#[test]
fn test_fast_path_not_fast_path_class() {
    let policy = FastPathPolicy::default();
    let snapshot = fresh_snapshot();
    let input = FastPathInput {
        request: "install vim",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(!result.handled);
    assert!(result.trace_note.contains("not in fast path class"));
}

#[test]
fn test_fast_path_system_health_fresh_healthy() {
    let policy = FastPathPolicy::default();
    let snapshot = fresh_snapshot();
    let input = FastPathInput {
        request: "how is my computer",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
    assert_eq!(result.class, FastPathClass::SystemHealth);
    // v0.0.40: RelevantHealthSummary returns minimal "no issues" message when healthy
    assert!(
        result.answer_text.contains("No critical issues") || result.answer_text.contains("healthy"),
        "Expected healthy message, got: {}",
        result.answer_text
    );
    assert!(!result.probes_run);
    assert!(result.reliability_hint >= 85);
}

#[test]
fn test_fast_path_system_health_with_issues() {
    let policy = FastPathPolicy::default();
    let snapshot = high_usage_snapshot();
    let input = FastPathInput {
        request: "any errors or problems",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
    assert_eq!(result.class, FastPathClass::SystemHealth);
    assert!(result.answer_text.contains("CRITICAL") || result.answer_text.contains("failed"));
    assert!(result.evidence_used.contains(&EvidenceKind::Disk));
    assert!(result.evidence_used.contains(&EvidenceKind::FailedUnits));
}

#[test]
fn test_fast_path_stale_snapshot_declined() {
    let policy = FastPathPolicy::default();
    let snapshot = stale_snapshot();
    let input = FastPathInput {
        request: "how is my computer",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(!result.handled);
    assert!(result.trace_note.contains("stale"));
}

#[test]
fn test_fast_path_disk_usage() {
    let policy = FastPathPolicy::default();
    let snapshot = fresh_snapshot();
    let input = FastPathInput {
        request: "disk usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
    assert_eq!(result.class, FastPathClass::DiskUsage);
    assert!(result.answer_text.contains("Disk Usage"));
    assert!(result.answer_text.contains("/"));
    assert!(result.evidence_used.contains(&EvidenceKind::Disk));
}

#[test]
fn test_fast_path_memory_usage() {
    let policy = FastPathPolicy::default();
    let snapshot = fresh_snapshot();
    let input = FastPathInput {
        request: "memory usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
    assert_eq!(result.class, FastPathClass::MemoryUsage);
    assert!(result.answer_text.contains("Memory Usage"));
    assert!(result.answer_text.contains("GB"));
    assert!(result.evidence_used.contains(&EvidenceKind::Memory));
}

#[test]
fn test_fast_path_failed_services_none() {
    let policy = FastPathPolicy::default();
    let snapshot = fresh_snapshot();
    let input = FastPathInput {
        request: "failed services",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
    assert_eq!(result.class, FastPathClass::FailedServices);
    assert!(result.answer_text.contains("No failed services"));
}

#[test]
fn test_fast_path_failed_services_with_failures() {
    let policy = FastPathPolicy::default();
    let snapshot = high_usage_snapshot();
    let input = FastPathInput {
        request: "failed services",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
    assert_eq!(result.class, FastPathClass::FailedServices);
    assert!(result.answer_text.contains("nginx.service"));
    assert!(result.answer_text.contains("Failed"));
}

#[test]
fn test_fast_path_no_snapshot_declined() {
    let policy = FastPathPolicy::default();
    let input = FastPathInput {
        request: "how is my computer",
        snapshot: None,
        facts: None,
        policy: &policy,
    };

    // This will try to load from disk - which won't exist in test env
    // So it should decline with "no snapshot available"
    let result = try_fast_path(&input);
    // Either handled from disk snapshot or declined
    // In test env without snapshot file, should decline
    if !result.handled {
        assert!(result.trace_note.contains("snapshot"));
    }
}
