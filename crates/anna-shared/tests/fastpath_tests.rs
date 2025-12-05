//! Tests for fastpath module (v0.0.39)

use anna_shared::fastpath::{
    classify_fast_path, try_fast_path, FastPathAnswer, FastPathClass, FastPathInput, FastPathPolicy,
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

// === Classification tests ===

#[test]
fn test_classify_system_health_basic() {
    assert_eq!(classify_fast_path("how is my computer"), FastPathClass::SystemHealth);
    assert_eq!(classify_fast_path("any errors"), FastPathClass::SystemHealth);
    assert_eq!(classify_fast_path("any problems"), FastPathClass::SystemHealth);
    assert_eq!(classify_fast_path("any warnings"), FastPathClass::SystemHealth);
    assert_eq!(classify_fast_path("status"), FastPathClass::SystemHealth);
    assert_eq!(classify_fast_path("health"), FastPathClass::SystemHealth);
}

#[test]
fn test_classify_system_health_with_greeting() {
    // The exact test case from definition of done
    assert_eq!(
        classify_fast_path("hello anna :) how is my computer? any errors or problems so far?"),
        FastPathClass::SystemHealth
    );
}

#[test]
fn test_classify_disk_usage() {
    assert_eq!(classify_fast_path("disk usage"), FastPathClass::DiskUsage);
    assert_eq!(classify_fast_path("disk space"), FastPathClass::DiskUsage);
    assert_eq!(classify_fast_path("how much disk"), FastPathClass::DiskUsage);
}

#[test]
fn test_classify_memory_usage() {
    assert_eq!(classify_fast_path("memory usage"), FastPathClass::MemoryUsage);
    assert_eq!(classify_fast_path("how much memory"), FastPathClass::MemoryUsage);
    assert_eq!(classify_fast_path("ram usage"), FastPathClass::MemoryUsage);
}

#[test]
fn test_classify_failed_services() {
    assert_eq!(classify_fast_path("failed services"), FastPathClass::FailedServices);
    assert_eq!(classify_fast_path("failed units"), FastPathClass::FailedServices);
}

#[test]
fn test_classify_what_changed() {
    assert_eq!(classify_fast_path("what changed since last time"), FastPathClass::WhatChanged);
    assert_eq!(classify_fast_path("what's new"), FastPathClass::WhatChanged);
}

#[test]
fn test_classify_not_fast_path() {
    assert_eq!(classify_fast_path("install vim"), FastPathClass::NotFastPath);
    assert_eq!(classify_fast_path("edit my vimrc"), FastPathClass::NotFastPath);
    assert_eq!(classify_fast_path("configure nginx"), FastPathClass::NotFastPath);
    assert_eq!(classify_fast_path("why is my network slow"), FastPathClass::NotFastPath);
}

// === Fast path execution tests ===

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

// === Deterministic formatting tests ===

#[test]
fn test_fast_path_answer_deterministic() {
    let policy = FastPathPolicy::default();
    let snapshot = fresh_snapshot();

    // Run twice, should get identical results
    let input1 = FastPathInput {
        request: "disk usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };
    let input2 = FastPathInput {
        request: "disk usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result1 = try_fast_path(&input1);
    let result2 = try_fast_path(&input2);

    assert_eq!(result1.answer_text, result2.answer_text);
    assert_eq!(result1.reliability_hint, result2.reliability_hint);
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

// === Evidence tracking tests ===

#[test]
fn test_evidence_kinds_tracked() {
    let policy = FastPathPolicy::default();
    let snapshot = high_usage_snapshot();
    let input = FastPathInput {
        request: "any errors",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);

    // Should include all relevant evidence kinds
    assert!(result.evidence_used.contains(&EvidenceKind::Memory));
    assert!(result.evidence_used.contains(&EvidenceKind::Disk));
    assert!(result.evidence_used.contains(&EvidenceKind::FailedUnits));
}

// === Class display tests ===

#[test]
fn test_fast_path_class_display() {
    assert_eq!(FastPathClass::SystemHealth.to_string(), "system_health");
    assert_eq!(FastPathClass::DiskUsage.to_string(), "disk_usage");
    assert_eq!(FastPathClass::MemoryUsage.to_string(), "memory_usage");
    assert_eq!(FastPathClass::FailedServices.to_string(), "failed_services");
    assert_eq!(FastPathClass::WhatChanged.to_string(), "what_changed");
    assert_eq!(FastPathClass::NotFastPath.to_string(), "not_fast_path");
}

// === Policy configuration tests ===

#[test]
fn test_custom_snapshot_max_age() {
    let policy = FastPathPolicy {
        snapshot_max_age_secs: 60, // 1 minute
        ..Default::default()
    };

    let mut snapshot = SystemSnapshot::now();
    snapshot.add_disk("/", 45);
    snapshot.set_memory(16_000_000_000, 8_000_000_000);
    // Fresh snapshot should work

    let input = FastPathInput {
        request: "disk usage",
        snapshot: Some(&snapshot),
        facts: None,
        policy: &policy,
    };

    let result = try_fast_path(&input);
    assert!(result.handled);
}

#[test]
fn test_fast_path_answer_not_handled_helper() {
    let answer = FastPathAnswer::not_handled("test reason");
    assert!(!answer.handled);
    assert!(answer.answer_text.is_empty());
    assert!(answer.trace_note.contains("test reason"));
    assert_eq!(answer.class, FastPathClass::NotFastPath);
}
