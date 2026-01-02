//! Tests for greeting insights.

use super::*;
use crate::teams::Team;
use std::collections::BTreeMap;

#[test]
fn test_critical_disk_insight() {
    let mut disk = BTreeMap::new();
    disk.insert("/".to_string(), 96);

    let snapshot = SystemSnapshot {
        disk,
        failed_services: vec![],
        memory_total_bytes: 16_000_000_000,
        memory_used_bytes: 4_000_000_000,
        ..Default::default()
    };

    let insights = generate_insights(&snapshot, &[]);
    assert!(!insights.is_empty());
    assert!(!insights[0].positive);
    assert!(insights[0].priority >= 90);
}

#[test]
fn test_healthy_system_good_news() {
    let mut disk = BTreeMap::new();
    disk.insert("/".to_string(), 30);

    let snapshot = SystemSnapshot {
        disk,
        failed_services: vec![],
        memory_total_bytes: 16_000_000_000,
        memory_used_bytes: 2_000_000_000,
        ..Default::default()
    };

    let insights = generate_insights(&snapshot, &[]);
    // Should have positive insights about good health
    let positive_count = insights.iter().filter(|i| i.positive).count();
    assert!(positive_count >= 1);
}

#[test]
fn test_quick_status_line() {
    let mut disk = BTreeMap::new();
    disk.insert("/".to_string(), 50);

    let snapshot = SystemSnapshot {
        disk,
        failed_services: vec![],
        memory_total_bytes: 16_000_000_000,
        memory_used_bytes: 4_000_000_000,
        ..Default::default()
    };

    let status = quick_status_line(&snapshot);
    assert!(status.contains("ok"));
}

#[test]
fn test_format_single_insight() {
    let insights = vec![GreetingInsight {
        staff_name: "Lars",
        team: Team::Storage,
        message: "disk at 85%".to_string(),
        priority: 70,
        positive: false,
    }];

    let formatted = format_insights_for_greeting(&insights);
    assert!(formatted.is_some());
    assert!(formatted.unwrap().contains("Lars"));
}

#[test]
fn test_learning_insight_priority_is_low() {
    // Learning insights should have low priority (< 30)
    // so they don't override system issues
    let mut insights = Vec::new();
    add_learning_insights(&mut insights);
    // If we have no learning data, no insight is added
    // If we have data, priority should be low
    for insight in &insights {
        assert!(
            insight.priority < 30,
            "Learning insight should have low priority"
        );
        assert!(insight.positive, "Learning insight should be positive");
    }
}
