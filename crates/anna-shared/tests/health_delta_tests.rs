//! Tests for health delta tracking (v0.0.41/v0.0.42).

use anna_shared::health_delta::{HealthDelta, SnapshotHistory};
use anna_shared::snapshot::SystemSnapshot;

#[test]
fn test_empty_history() {
    let history = SnapshotHistory::new();
    assert!(history.is_empty());
    assert!(history.latest().is_none());
    assert!(history.previous().is_none());
    assert!(history.latest_delta().is_none());
}

#[test]
fn test_history_rotation() {
    let mut history = SnapshotHistory::new();

    // Add more than MAX_HISTORY_SIZE snapshots
    for i in 0..10 {
        let mut snap = SystemSnapshot::now();
        snap.set_memory(1000, (i * 100) as u64);
        history.push(snap);
    }

    assert_eq!(history.len(), 5); // MAX_HISTORY_SIZE
    // Latest should be the last one added (i=9)
    let latest = history.latest().unwrap();
    assert_eq!(latest.memory_used_bytes, 900);
}

#[test]
fn test_health_delta_no_changes() {
    let snap1 = SystemSnapshot::now();
    let snap2 = snap1.clone();

    let delta = HealthDelta::from_snapshots(&snap1, &snap2);
    assert!(!delta.has_changes());
    assert!(!delta.has_actionable());
}

#[test]
fn test_health_delta_memory_change() {
    let mut snap1 = SystemSnapshot::now();
    snap1.set_memory(1000, 500);

    let mut snap2 = SystemSnapshot::now();
    snap2.set_memory(1000, 600);

    let delta = HealthDelta::from_snapshots(&snap1, &snap2);
    assert!(delta.has_changes());
    assert!(delta.changed_fields.contains(&"memory".to_string()));
}

#[test]
fn test_health_summary_healthy() {
    let mut snap = SystemSnapshot::now();
    snap.set_memory(100, 50); // 50%
    snap.add_disk("/", 50);

    let mut history = SnapshotHistory::new();
    history.push(snap);

    let summary = history.health_summary();
    assert!(summary.is_healthy());
}

#[test]
fn test_health_summary_unhealthy() {
    let mut snap = SystemSnapshot::now();
    snap.set_memory(100, 50);
    snap.add_disk("/", 96); // Critical!

    let mut history = SnapshotHistory::new();
    history.push(snap);

    let summary = history.health_summary();
    assert!(!summary.is_healthy());
}

// v0.0.42 IT-style format tests
#[test]
fn test_it_style_healthy() {
    let mut snap = SystemSnapshot::now();
    snap.set_memory(100, 50); // 50%
    snap.add_disk("/", 50);

    let mut history = SnapshotHistory::new();
    history.push(snap);

    let summary = history.health_summary();
    let output = summary.format_it_style();
    assert_eq!(output, "All systems operational");
}

#[test]
fn test_it_style_high_memory() {
    let mut snap = SystemSnapshot::now();
    snap.set_memory(100, 85); // 85%
    snap.add_disk("/", 50);

    let mut history = SnapshotHistory::new();
    history.push(snap);

    let summary = history.health_summary();
    let output = summary.format_it_style();
    assert!(output.contains("Memory: 85%"));
    assert!(output.contains("high"));
}

#[test]
fn test_it_style_critical_disk() {
    let mut snap = SystemSnapshot::now();
    snap.set_memory(100, 50);
    snap.add_disk("/", 96);

    let mut history = SnapshotHistory::new();
    history.push(snap);

    let summary = history.health_summary();
    let output = summary.format_it_style();
    assert!(output.contains("critical"));
    assert!(output.contains("96%"));
}

#[test]
fn test_one_liner_healthy() {
    let mut snap = SystemSnapshot::now();
    snap.set_memory(100, 50);
    snap.add_disk("/", 50);

    let mut history = SnapshotHistory::new();
    history.push(snap);

    let summary = history.health_summary();
    assert_eq!(summary.one_liner(), "Your computer is running smoothly.");
}

#[test]
fn test_issue_and_warning_counts() {
    let mut snap = SystemSnapshot::now();
    snap.set_memory(100, 92); // Issue: >90
    snap.add_disk("/", 96);   // Issue: >95
    snap.add_disk("/home", 85); // Warning: 80-95

    let mut history = SnapshotHistory::new();
    history.push(snap);

    let summary = history.health_summary();
    assert_eq!(summary.issue_count(), 2); // memory + disk /
    assert_eq!(summary.warning_count(), 1); // disk /home
}
