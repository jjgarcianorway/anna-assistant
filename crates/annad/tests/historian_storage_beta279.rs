//! Historian Storage Tests - Beta.279
//!
//! Tests for the historian module's storage, retrieval, and retention logic.
//! Complements proactive_historian_beta279.rs which tests correlation integration.

use annad::historian::{HistoryEvent, Historian};
use chrono::{Duration, Utc};
use tempfile::TempDir;

#[test]
fn test_historian_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let historian = Historian::new(temp_dir.path()).unwrap();

    // Should create history file parent directory
    assert!(temp_dir.path().exists());

    // Fresh historian should have no events
    let events = historian.load_all().unwrap();
    assert_eq!(events.len(), 0);
}

#[test]
fn test_append_and_load_single_event() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let mut event = HistoryEvent::new();
    event.hostname = "test-host".to_string();
    event.disk_root_usage_pct = 75;
    event.failed_services_count = 2;

    historian.append(&event).unwrap();

    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].hostname, "test-host");
    assert_eq!(loaded[0].disk_root_usage_pct, 75);
    assert_eq!(loaded[0].failed_services_count, 2);
}

#[test]
fn test_append_multiple_events_chronological_order() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let base_time = Utc::now() - Duration::hours(3);

    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours(i);
        event.hostname = format!("host-{}", i);
        historian.append(&event).unwrap();
    }

    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded.len(), 5);

    // Verify chronological order
    for i in 0..4 {
        assert!(loaded[i].timestamp_utc <= loaded[i + 1].timestamp_utc);
    }

    assert_eq!(loaded[0].hostname, "host-0");
    assert_eq!(loaded[4].hostname, "host-4");
}

#[test]
fn test_load_recent_filters_by_time_window() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let now = Utc::now();

    // Add events at different times
    let mut old_event = HistoryEvent::new();
    old_event.timestamp_utc = now - Duration::hours(5);
    old_event.hostname = "old".to_string();
    historian.append(&old_event).unwrap();

    let mut recent_event1 = HistoryEvent::new();
    recent_event1.timestamp_utc = now - Duration::minutes(30);
    recent_event1.hostname = "recent1".to_string();
    historian.append(&recent_event1).unwrap();

    let mut recent_event2 = HistoryEvent::new();
    recent_event2.timestamp_utc = now - Duration::minutes(15);
    recent_event2.hostname = "recent2".to_string();
    historian.append(&recent_event2).unwrap();

    // Load only last hour
    let recent = historian.load_recent(Duration::hours(1)).unwrap();

    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].hostname, "recent1");
    assert_eq!(recent[1].hostname, "recent2");
}

#[test]
fn test_retention_rotation() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();
    historian.set_max_entries(10); // Set low limit for testing

    // Append 15 events
    for i in 0..15 {
        let mut event = HistoryEvent::new();
        event.hostname = format!("host-{}", i);
        historian.append(&event).unwrap();
    }

    // Should have rotated to keep only last 10
    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded.len(), 10);

    // Should keep newest entries (host-5 through host-14)
    assert_eq!(loaded[0].hostname, "host-5");
    assert_eq!(loaded[9].hostname, "host-14");
}

#[test]
fn test_kernel_change_detection() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    // First event with kernel 6.5.1
    let mut event1 = HistoryEvent::new();
    event1.kernel_version = "6.5.1".to_string();
    event1.kernel_changed = false;
    historian.append(&event1).unwrap();

    // Second event with same kernel
    let mut event2 = HistoryEvent::new();
    event2.kernel_version = "6.5.1".to_string();
    event2.kernel_changed = false;
    historian.append(&event2).unwrap();

    // Third event with new kernel - kernel_changed should be true
    let mut event3 = HistoryEvent::new();
    event3.kernel_version = "6.6.0".to_string();
    event3.kernel_changed = true; // This would be set by build_history_event
    historian.append(&event3).unwrap();

    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded.len(), 3);
    assert!(!loaded[0].kernel_changed);
    assert!(!loaded[1].kernel_changed);
    assert!(loaded[2].kernel_changed);
}

#[test]
fn test_last_kernel_version_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    assert!(historian.last_kernel_version().is_none());

    let mut event = HistoryEvent::new();
    event.kernel_version = "6.5.1".to_string();
    historian.append(&event).unwrap();

    assert_eq!(historian.last_kernel_version(), Some("6.5.1"));
}

#[test]
fn test_last_boot_id_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    assert!(historian.last_boot_id().is_none());

    let mut event = HistoryEvent::new();
    event.boot_id = "boot-123".to_string();
    historian.append(&event).unwrap();

    assert_eq!(historian.last_boot_id(), Some("boot-123"));
}

#[test]
fn test_corrupted_line_handling() {
    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.jsonl");

    // Write a valid event
    let mut event = HistoryEvent::new();
    event.hostname = "valid-host".to_string();
    let valid_json = serde_json::to_string(&event).unwrap();

    // Write valid, corrupted, valid pattern
    let content = format!("{}\n{{\n{}\n", valid_json, valid_json);
    std::fs::write(&history_path, content).unwrap();

    let historian = Historian::new(temp_dir.path()).unwrap();
    let loaded = historian.load_all().unwrap();

    // Should load 2 valid events, skip corrupted line
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].hostname, "valid-host");
}

#[test]
fn test_empty_lines_skipped() {
    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.jsonl");

    let mut event = HistoryEvent::new();
    event.hostname = "test".to_string();
    let json = serde_json::to_string(&event).unwrap();

    // Write with empty lines
    let content = format!("{}\n\n\n{}\n\n", json, json);
    std::fs::write(&history_path, content).unwrap();

    let historian = Historian::new(temp_dir.path()).unwrap();
    let loaded = historian.load_all().unwrap();

    assert_eq!(loaded.len(), 2);
}

#[test]
fn test_future_schema_version_skipped() {
    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.jsonl");

    // Create event with current schema
    let mut current_event = HistoryEvent::new();
    current_event.hostname = "current".to_string();
    current_event.schema_version = 1;

    // Manually create event with future schema
    let mut future_event = current_event.clone();
    future_event.hostname = "future".to_string();
    future_event.schema_version = 99; // Future version

    let current_json = serde_json::to_string(&current_event).unwrap();
    let future_json = serde_json::to_string(&future_event).unwrap();

    let content = format!("{}\n{}\n{}\n", current_json, future_json, current_json);
    std::fs::write(&history_path, content).unwrap();

    let historian = Historian::new(temp_dir.path()).unwrap();
    let loaded = historian.load_all().unwrap();

    // Should load only current schema events
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].hostname, "current");
    assert_eq!(loaded[1].hostname, "current");
}

#[test]
fn test_rotation_preserves_order() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();
    historian.set_max_entries(3);

    // Add 5 events
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.hostname = format!("host-{}", i);
        historian.append(&event).unwrap();
    }

    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded.len(), 3);

    // Verify chronological order maintained after rotation
    assert_eq!(loaded[0].hostname, "host-2");
    assert_eq!(loaded[1].hostname, "host-3");
    assert_eq!(loaded[2].hostname, "host-4");
}

#[test]
fn test_high_flags_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let mut event = HistoryEvent::new();
    event.high_cpu_flag = true;
    event.high_memory_flag = true;
    historian.append(&event).unwrap();

    let loaded = historian.load_all().unwrap();
    assert!(loaded[0].high_cpu_flag);
    assert!(loaded[0].high_memory_flag);
}

#[test]
fn test_network_metrics_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let mut event = HistoryEvent::new();
    event.network_packet_loss_pct = 15;
    event.network_latency_ms = 250;
    historian.append(&event).unwrap();

    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded[0].network_packet_loss_pct, 15);
    assert_eq!(loaded[0].network_latency_ms, 250);
}

#[test]
fn test_service_counts_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let mut event = HistoryEvent::new();
    event.failed_services_count = 3;
    event.degraded_services_count = 2;
    historian.append(&event).unwrap();

    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded[0].failed_services_count, 3);
    assert_eq!(loaded[0].degraded_services_count, 2);
}

#[test]
fn test_disk_usage_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let mut event = HistoryEvent::new();
    event.disk_root_usage_pct = 85;
    event.disk_other_max_usage_pct = 72;
    historian.append(&event).unwrap();

    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded[0].disk_root_usage_pct, 85);
    assert_eq!(loaded[0].disk_other_max_usage_pct, 72);
}

#[test]
fn test_device_hotplug_flag() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    let mut event = HistoryEvent::new();
    event.device_hotplug_flag = true;
    historian.append(&event).unwrap();

    let loaded = historian.load_all().unwrap();
    assert!(loaded[0].device_hotplug_flag);
}

#[test]
fn test_load_recent_empty_when_no_events() {
    let temp_dir = TempDir::new().unwrap();
    let historian = Historian::new(temp_dir.path()).unwrap();

    let recent = historian.load_recent(Duration::hours(1)).unwrap();
    assert_eq!(recent.len(), 0);
}

#[test]
fn test_append_after_rotation() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();
    historian.set_max_entries(3);

    // Fill to trigger rotation
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.hostname = format!("pre-{}", i);
        historian.append(&event).unwrap();
    }

    // Append after rotation
    let mut new_event = HistoryEvent::new();
    new_event.hostname = "post".to_string();
    historian.append(&new_event).unwrap();

    let loaded = historian.load_all().unwrap();

    // Should still enforce limit
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded[2].hostname, "post");
}
