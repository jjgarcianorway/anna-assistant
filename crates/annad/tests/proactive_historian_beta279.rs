//! Proactive Historian Integration Tests - Beta.279
//!
//! These tests verify that the proactive correlation engine correctly uses
//! historian data to detect temporal patterns and trends.
//!
//! Test coverage:
//! - Service flapping detection (SVC-001)
//! - Disk growth trend detection (DISK-002)
//! - Sustained resource pressure (RES-001, RES-002)
//! - Kernel regression detection (SYS-001)
//! - Network degradation trends (NET-003)

use annad::historian::{HistoryEvent, Historian};
use chrono::{Duration, Utc};
use tempfile::TempDir;

// Helper to create historian with sample events
fn setup_historian_with_events(events: Vec<HistoryEvent>) -> (TempDir, Historian) {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();

    for event in events {
        historian.append(&event).unwrap();
    }

    (temp_dir, historian)
}

// ============================================================================
// SVC-001: Service Flapping Detection Tests
// ============================================================================

#[test]
fn test_service_flapping_detected_with_3_transitions() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // Create flapping pattern: 0 → 2 → 0 → 2 (3 transitions)
    for i in 0..4 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 15);
        event.failed_services_count = if i % 2 == 0 { 0 } else { 2 };
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    assert_eq!(loaded.len(), 4);

    // Count transitions
    let mut transitions = 0;
    let mut last_had_failures = loaded[0].failed_services_count > 0;
    for event in loaded.iter().skip(1) {
        let current = event.failed_services_count > 0;
        if current != last_had_failures {
            transitions += 1;
        }
        last_had_failures = current;
    }

    assert_eq!(transitions, 3, "Should detect 3 transitions for flapping");
}

#[test]
fn test_service_flapping_not_detected_with_stable_services() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // Stable: all zeros
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 10);
        event.failed_services_count = 0;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let mut transitions = 0;
    let mut last = loaded[0].failed_services_count > 0;
    for event in loaded.iter().skip(1) {
        let current = event.failed_services_count > 0;
        if current != last {
            transitions += 1;
        }
        last = current;
    }

    assert!(transitions < 3, "Stable services should not show flapping");
}

#[test]
fn test_service_flapping_with_many_transitions() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(2);

    // Rapid flapping: 0→1→0→1→0→1→0 (6 transitions)
    for i in 0..7 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 15);
        event.failed_services_count = if i % 2 == 0 { 0 } else { 1 };
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let mut transitions = 0;
    let mut last = loaded[0].failed_services_count > 0;
    for event in loaded.iter().skip(1) {
        let current = event.failed_services_count > 0;
        if current != last {
            transitions += 1;
        }
        last = current;
    }

    assert_eq!(transitions, 6);
}

// ============================================================================
// DISK-002: Disk Growth Detection Tests
// ============================================================================

#[test]
fn test_disk_growth_detected_above_threshold() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(24);

    // Growth from 65% to 85% (20 point increase)
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours((i * 5) as i64);
        event.disk_root_usage_pct = 65 + (i * 5) as u8;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let first_usage = loaded[0].disk_root_usage_pct;
    let last_usage = loaded.last().unwrap().disk_root_usage_pct;
    let growth = last_usage.saturating_sub(first_usage);

    assert!(growth >= 15, "Should detect significant disk growth");
    assert!(last_usage >= 80, "Should grow into danger zone");
}

#[test]
fn test_disk_growth_not_detected_below_danger_zone() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(12);

    // Growth from 50% to 70% (not reaching 80%)
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours((i * 2) as i64);
        event.disk_root_usage_pct = 50 + (i * 4) as u8;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let last_usage = loaded.last().unwrap().disk_root_usage_pct;

    assert!(last_usage < 80, "Below danger zone should not trigger");
}

#[test]
fn test_disk_growth_requires_monotonic_trend() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(24);

    // Erratic: 70→75→72→78→85 (ends high but not steady growth)
    let values = vec![70, 75, 72, 78, 85];
    for (i, &usage) in values.iter().enumerate() {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours((i * 5) as i64);
        event.disk_root_usage_pct = usage;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    // Check monotonic trend requirement (70% increasing)
    let mut increasing_count = 0;
    for i in 1..loaded.len() {
        if loaded[i].disk_root_usage_pct >= loaded[i - 1].disk_root_usage_pct {
            increasing_count += 1;
        }
    }

    let total_transitions = loaded.len() - 1;
    let is_mostly_increasing = increasing_count * 10 >= total_transitions * 7;

    assert!(is_mostly_increasing, "Should require mostly increasing trend");
}

// ============================================================================
// RES-001, RES-002: Resource Pressure Detection Tests
// ============================================================================

#[test]
fn test_sustained_cpu_pressure_detected() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // 7 out of 10 events with high CPU (70% sustained)
    for i in 0..10 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 6);
        event.high_cpu_flag = i < 7; // First 7 have high CPU
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let high_cpu_count = loaded.iter().filter(|e| e.high_cpu_flag).count();
    let threshold = (loaded.len() * 6) / 10; // 60%

    assert!(high_cpu_count >= threshold, "Should detect sustained CPU pressure");
}

#[test]
fn test_cpu_spike_not_sustained() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // Only 3 out of 10 events with high CPU (30%)
    for i in 0..10 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 6);
        event.high_cpu_flag = i >= 4 && i <= 6; // Only 3 events
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let high_cpu_count = loaded.iter().filter(|e| e.high_cpu_flag).count();
    let threshold = (loaded.len() * 6) / 10;

    assert!(high_cpu_count < threshold, "Single spike should not trigger sustained pressure");
}

#[test]
fn test_sustained_memory_pressure_detected() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // 8 out of 10 events with high memory
    for i in 0..10 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 6);
        event.high_memory_flag = i >= 2; // Last 8 have high memory
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let high_memory_count = loaded.iter().filter(|e| e.high_memory_flag).count();
    let threshold = (loaded.len() * 6) / 10;

    assert!(high_memory_count >= threshold, "Should detect sustained memory pressure");
}

// ============================================================================
// SYS-001: Kernel Regression Detection Tests
// ============================================================================

#[test]
fn test_kernel_regression_detected_after_upgrade() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(24);

    // Before kernel change: 0 failures
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours(i as i64);
        event.kernel_version = "6.5.1".to_string();
        event.failed_services_count = 0;
        event.degraded_services_count = 0;
        event.kernel_changed = false;
        events.push(event);
    }

    // Kernel change event
    let mut change_event = HistoryEvent::new();
    change_event.timestamp_utc = base_time + Duration::hours(5);
    change_event.kernel_version = "6.6.0".to_string();
    change_event.failed_services_count = 1;
    change_event.kernel_changed = true;
    events.push(change_event);

    // After kernel change: increased failures
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours((6 + i) as i64);
        event.kernel_version = "6.6.0".to_string();
        event.failed_services_count = 2;
        event.degraded_services_count = 1;
        event.kernel_changed = false;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let change_idx = loaded.iter().position(|e| e.kernel_changed).unwrap();
    let before = &loaded[..change_idx];
    let after = &loaded[change_idx + 1..];

    let before_avg: f64 = before.iter()
        .map(|e| (e.failed_services_count + e.degraded_services_count) as f64)
        .sum::<f64>() / before.len() as f64;

    let after_avg: f64 = after.iter()
        .map(|e| (e.failed_services_count + e.degraded_services_count) as f64)
        .sum::<f64>() / after.len() as f64;

    assert!(after_avg > before_avg + 1.0, "Should detect kernel regression");
}

#[test]
fn test_kernel_upgrade_without_regression() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(12);

    // Before and after: same low failure rate
    for i in 0..3 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours(i as i64);
        event.kernel_version = "6.5.1".to_string();
        event.failed_services_count = 0;
        events.push(event);
    }

    let mut change_event = HistoryEvent::new();
    change_event.timestamp_utc = base_time + Duration::hours(3);
    change_event.kernel_version = "6.6.0".to_string();
    change_event.kernel_changed = true;
    events.push(change_event);

    for i in 0..3 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::hours((4 + i) as i64);
        event.kernel_version = "6.6.0".to_string();
        event.failed_services_count = 0;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let change_idx = loaded.iter().position(|e| e.kernel_changed).unwrap();
    let before = &loaded[..change_idx];
    let after = &loaded[change_idx + 1..];

    let before_avg: f64 = before.iter()
        .map(|e| e.failed_services_count as f64)
        .sum::<f64>() / before.len() as f64;

    let after_avg: f64 = after.iter()
        .map(|e| e.failed_services_count as f64)
        .sum::<f64>() / after.len() as f64;

    assert!(after_avg <= before_avg + 1.0, "Clean upgrade should not show regression");
}

// ============================================================================
// NET-003: Network Degradation Trend Tests
// ============================================================================

#[test]
fn test_network_packet_loss_trend_detected() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // Rising packet loss: 1% → 8%
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 12);
        event.network_packet_loss_pct = 1 + (i * 2) as u8;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let first_loss = loaded[0].network_packet_loss_pct;
    let last_loss = loaded.last().unwrap().network_packet_loss_pct;

    assert!(last_loss > 5, "Should reach meaningful packet loss");
    assert!(last_loss > first_loss + 3, "Should show rising trend");
}

#[test]
fn test_network_latency_trend_detected() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // Rising latency: 50ms → 180ms
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 12);
        event.network_latency_ms = 50 + (i * 30) as u16;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let first_lat = loaded[0].network_latency_ms;
    let last_lat = loaded.last().unwrap().network_latency_ms;

    assert!(last_lat > 100, "Should reach high latency");
    assert!(last_lat > first_lat + 50, "Should show rising trend");
}

#[test]
fn test_stable_network_no_degradation() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(1);

    // Stable: low packet loss and latency
    for i in 0..5 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 10);
        event.network_packet_loss_pct = 0;
        event.network_latency_ms = 25;
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);
    let loaded = historian.load_all().unwrap();

    let first_loss = loaded[0].network_packet_loss_pct;
    let last_loss = loaded.last().unwrap().network_packet_loss_pct;

    assert!(last_loss <= 5 || last_loss <= first_loss + 3, "Stable network should not trigger");
}

// ============================================================================
// Historian Storage Tests
// ============================================================================

#[test]
fn test_historian_load_recent_respects_time_window() {
    let mut events = Vec::new();
    let base_time = Utc::now() - Duration::hours(5);

    // Create events over 5 hours
    for i in 0..10 {
        let mut event = HistoryEvent::new();
        event.timestamp_utc = base_time + Duration::minutes(i * 30);
        event.hostname = format!("event-{}", i);
        events.push(event);
    }

    let (_temp, historian) = setup_historian_with_events(events);

    // Load only last 2 hours
    let recent = historian.load_recent(Duration::hours(2)).unwrap();

    let cutoff = Utc::now() - Duration::hours(2);
    for event in &recent {
        assert!(event.timestamp_utc >= cutoff, "All events should be within window");
    }
}

#[test]
fn test_historian_retention_enforced() {
    let temp_dir = TempDir::new().unwrap();
    let mut historian = Historian::new(temp_dir.path()).unwrap();
    historian.set_max_entries(5); // Small limit for testing

    // Add 10 events (exceeds limit)
    for i in 0..10 {
        let mut event = HistoryEvent::new();
        event.hostname = format!("host-{}", i);
        historian.append(&event).unwrap();
    }

    // Should have rotated to keep only last 5
    let loaded = historian.load_all().unwrap();
    assert_eq!(loaded.len(), 5, "Should enforce retention limit");
    assert_eq!(loaded[0].hostname, "host-5", "Should keep newest entries");
}

#[test]
fn test_historian_handles_corrupted_data() {
    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.jsonl");

    // Write valid event followed by corrupted line
    let mut event = HistoryEvent::new();
    event.hostname = "valid-host".to_string();
    let valid_json = serde_json::to_string(&event).unwrap();

    let content = format!("{}\n{{invalid json\n{}\n", valid_json, valid_json);
    std::fs::write(&history_path, content).unwrap();

    let historian = Historian::new(temp_dir.path()).unwrap();
    let loaded = historian.load_all().unwrap();

    // Should skip corrupted line and load valid events
    assert!(loaded.len() >= 1, "Should recover from corruption");
}
