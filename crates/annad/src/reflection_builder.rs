//! Reflection Builder - Constructs ReflectionSummary from system state (6.7.0)
//!
//! Extracts concrete information from:
//! - Historian (disk growth, service failures, resource pressure)
//! - ProactiveAssessment (correlated issues, trends)
//! - Anna's own version info
//!
//! Returns concrete metrics and real error messages, not vague statements.

use crate::historian::{HistoryEvent, Historian};
use crate::intel::proactive_engine::{ProactiveAssessment, IssueSeverity};
use crate::reflection::{ReflectionItem, ReflectionSeverity, ReflectionSummary};
use chrono::{DateTime, Duration, Utc};

/// Version information for Anna
pub struct VersionInfo {
    pub current_version: String,
    pub previous_version: Option<String>,
}

/// Build a reflection summary from system state
///
/// This function analyzes:
/// 1. Anna's own version changes
/// 2. System trends from Historian
/// 3. Correlated issues from ProactiveAssessment
/// 4. Log errors (TODO: add log analysis)
///
/// Returns ReflectionSummary with concrete details, no solutions proposed.
pub fn build_reflection_summary(
    historian: &Historian,
    proactive: Option<&ProactiveAssessment>,
    version_info: &VersionInfo,
) -> ReflectionSummary {
    let mut summary = ReflectionSummary::empty();

    // 1. Check for Anna version upgrades
    if let Some(prev_version) = &version_info.previous_version {
        if prev_version != &version_info.current_version {
            summary.add_item(ReflectionItem::new(
                ReflectionSeverity::Notice,
                "upgrade",
                format!("Anna updated to v{}", version_info.current_version),
                format!(
                    "Previously v{}, now v{}. This may change diagnostics and planner behavior.",
                    prev_version, version_info.current_version
                ),
            ));
        }
    }

    // 2. Extract trends from Historian (last 7 days)
    if let Ok(recent_events) = historian.load_recent(Duration::days(7)) {
        extract_historian_trends(&mut summary, &recent_events);
    }

    // 3. Extract issues from ProactiveAssessment
    if let Some(assessment) = proactive {
        extract_proactive_issues(&mut summary, assessment);
    }

    // 4. TODO: Extract log errors (requires journal integration)

    summary
}

/// Extract trends from recent history events
fn extract_historian_trends(summary: &mut ReflectionSummary, events: &[HistoryEvent]) {
    if events.len() < 2 {
        return; // Need at least 2 events to detect trends
    }

    let latest = &events[events.len() - 1];
    let day_ago_idx = events
        .iter()
        .rposition(|e| {
            (latest.timestamp_utc - e.timestamp_utc).num_hours() >= 24
        })
        .unwrap_or(0);

    if day_ago_idx >= events.len() - 1 {
        return; // Not enough time-separated data
    }

    let day_ago = &events[day_ago_idx];

    // Disk usage trends
    check_disk_trends(summary, latest, day_ago);

    // Service failure trends
    check_service_trends(summary, latest, day_ago);

    // Network degradation trends
    check_network_trends(summary, latest, day_ago);

    // Resource pressure trends
    check_resource_trends(summary, latest, day_ago);

    // Kernel changes
    if latest.kernel_changed {
        summary.add_item(
            ReflectionItem::new(
                ReflectionSeverity::Notice,
                "kernel",
                format!("Kernel updated to {}", latest.kernel_version),
                "Kernel version changed since last boot. Monitor for regressions.",
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }
}

/// Check for disk usage trends
fn check_disk_trends(summary: &mut ReflectionSummary, latest: &HistoryEvent, day_ago: &HistoryEvent) {
    // Root partition
    if latest.disk_root_usage_pct > 85 && latest.disk_root_usage_pct > day_ago.disk_root_usage_pct {
        let severity = if latest.disk_root_usage_pct > 95 {
            ReflectionSeverity::Critical
        } else if latest.disk_root_usage_pct > 90 {
            ReflectionSeverity::Warning
        } else {
            ReflectionSeverity::Notice
        };

        summary.add_item(
            ReflectionItem::new(
                severity,
                "disk",
                format!("/ is at {}% usage", latest.disk_root_usage_pct),
                format!(
                    "Disk usage on / increased from {}% to {}% over 24h",
                    day_ago.disk_root_usage_pct, latest.disk_root_usage_pct
                ),
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }

    // Other partitions
    if latest.disk_other_max_usage_pct > 85
        && latest.disk_other_max_usage_pct > day_ago.disk_other_max_usage_pct
    {
        let severity = if latest.disk_other_max_usage_pct > 95 {
            ReflectionSeverity::Critical
        } else if latest.disk_other_max_usage_pct > 90 {
            ReflectionSeverity::Warning
        } else {
            ReflectionSeverity::Notice
        };

        summary.add_item(
            ReflectionItem::new(
                severity,
                "disk",
                format!("Other partition at {}% usage", latest.disk_other_max_usage_pct),
                format!(
                    "Highest disk usage on other partitions increased from {}% to {}% over 24h",
                    day_ago.disk_other_max_usage_pct, latest.disk_other_max_usage_pct
                ),
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }
}

/// Check for service failure trends
fn check_service_trends(summary: &mut ReflectionSummary, latest: &HistoryEvent, day_ago: &HistoryEvent) {
    if latest.failed_services_count > 0 && latest.failed_services_count > day_ago.failed_services_count {
        let severity = if latest.failed_services_count > 5 {
            ReflectionSeverity::Critical
        } else if latest.failed_services_count > 2 {
            ReflectionSeverity::Warning
        } else {
            ReflectionSeverity::Notice
        };

        summary.add_item(
            ReflectionItem::new(
                severity,
                "services",
                format!("{} failed services detected", latest.failed_services_count),
                format!(
                    "Failed service count increased from {} to {} over 24h",
                    day_ago.failed_services_count, latest.failed_services_count
                ),
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }
}

/// Check for network degradation trends
fn check_network_trends(summary: &mut ReflectionSummary, latest: &HistoryEvent, day_ago: &HistoryEvent) {
    // Packet loss increase
    if latest.network_packet_loss_pct > 2 && latest.network_packet_loss_pct > day_ago.network_packet_loss_pct {
        let severity = if latest.network_packet_loss_pct > 10 {
            ReflectionSeverity::Critical
        } else if latest.network_packet_loss_pct > 5 {
            ReflectionSeverity::Warning
        } else {
            ReflectionSeverity::Notice
        };

        summary.add_item(
            ReflectionItem::new(
                severity,
                "network",
                format!("Packet loss at {}%", latest.network_packet_loss_pct),
                format!(
                    "Average packet loss increased from {}% to {}% over 24h",
                    day_ago.network_packet_loss_pct, latest.network_packet_loss_pct
                ),
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }

    // Latency increase
    if latest.network_latency_ms > 100 && latest.network_latency_ms > day_ago.network_latency_ms + 50 {
        let severity = if latest.network_latency_ms > 500 {
            ReflectionSeverity::Warning
        } else {
            ReflectionSeverity::Notice
        };

        summary.add_item(
            ReflectionItem::new(
                severity,
                "network",
                format!("Latency increased to {}ms", latest.network_latency_ms),
                format!(
                    "Network latency increased from {}ms to {}ms over 24h",
                    day_ago.network_latency_ms, latest.network_latency_ms
                ),
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }
}

/// Check for resource pressure trends
fn check_resource_trends(summary: &mut ReflectionSummary, latest: &HistoryEvent, _day_ago: &HistoryEvent) {
    if latest.high_cpu_flag {
        summary.add_item(
            ReflectionItem::new(
                ReflectionSeverity::Warning,
                "cpu",
                "High CPU usage detected",
                "CPU usage sustained above 80% threshold",
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }

    if latest.high_memory_flag {
        summary.add_item(
            ReflectionItem::new(
                ReflectionSeverity::Warning,
                "memory",
                "High memory usage detected",
                "Memory usage sustained above 85% threshold",
            )
            .with_timestamp(latest.timestamp_utc),
        );
    }
}

/// Extract issues from ProactiveAssessment
fn extract_proactive_issues(summary: &mut ReflectionSummary, assessment: &ProactiveAssessment) {
    // Only report critical and warning level issues in reflection
    // Info-level issues are too noisy for this context
    for issue in &assessment.correlated_issues {
        let severity = match issue.severity {
            IssueSeverity::Critical => ReflectionSeverity::Critical,
            IssueSeverity::Warning => ReflectionSeverity::Warning,
            IssueSeverity::Trend => ReflectionSeverity::Notice,
            IssueSeverity::Info => continue, // Skip info-level in reflection
        };

        summary.add_item(
            ReflectionItem::new(
                severity,
                "health",
                issue.summary.clone(),
                issue.details.clone(),
            )
            .with_timestamp(issue.last_seen),
        );
    }

    // Report significant trends
    for trend in &assessment.trends {
        if trend.projected_severity == IssueSeverity::Info {
            continue; // Skip low-severity trends
        }

        let severity = match trend.projected_severity {
            IssueSeverity::Critical => ReflectionSeverity::Critical,
            IssueSeverity::Warning => ReflectionSeverity::Warning,
            IssueSeverity::Trend => ReflectionSeverity::Notice,
            IssueSeverity::Info => continue,
        };

        summary.add_item(
            ReflectionItem::new(
                severity,
                "trend",
                format!("{} trending", trend.subject),
                format!(
                    "{:?} trend observed for {} minutes. {}",
                    trend.trend_type, trend.duration_minutes, trend.recommendation
                ),
            )
            .with_timestamp(trend.first_detected),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intel::proactive_engine::{CorrelatedIssue, RootCause, TrendObservation};
    use chrono::Duration;

    #[test]
    fn test_version_upgrade_detected() {
        let historian = Historian::new_with_retention("/tmp/test_history.jsonl", 100);
        let version_info = VersionInfo {
            current_version: "6.7.0".to_string(),
            previous_version: Some("6.5.0".to_string()),
        };

        let summary = build_reflection_summary(&historian, None, &version_info);

        assert!(summary.has_items());
        let upgrade_items: Vec<_> = summary
            .items
            .iter()
            .filter(|i| i.category == "upgrade")
            .collect();
        assert_eq!(upgrade_items.len(), 1);
        assert!(upgrade_items[0].title.contains("6.7.0"));
        assert!(upgrade_items[0].details.contains("6.5.0"));
    }

    #[test]
    fn test_no_upgrade_if_same_version() {
        let historian = Historian::new_with_retention("/tmp/test_history.jsonl", 100);
        let version_info = VersionInfo {
            current_version: "6.7.0".to_string(),
            previous_version: Some("6.7.0".to_string()),
        };

        let summary = build_reflection_summary(&historian, None, &version_info);

        let upgrade_items: Vec<_> = summary
            .items
            .iter()
            .filter(|i| i.category == "upgrade")
            .collect();
        assert_eq!(upgrade_items.len(), 0);
    }

    #[test]
    fn test_disk_trend_detection() {
        // Create synthetic history with disk growth
        let now = Utc::now();
        let day_ago = now - Duration::hours(24);

        let mut old_event = HistoryEvent::new();
        old_event.timestamp_utc = day_ago;
        old_event.disk_root_usage_pct = 76;

        let mut new_event = HistoryEvent::new();
        new_event.timestamp_utc = now;
        new_event.disk_root_usage_pct = 91;

        let events = vec![old_event, new_event];
        let mut summary = ReflectionSummary::empty();
        extract_historian_trends(&mut summary, &events);

        let disk_items: Vec<_> = summary
            .items
            .iter()
            .filter(|i| i.category == "disk")
            .collect();
        assert_eq!(disk_items.len(), 1);
        assert!(disk_items[0].title.contains("91%"));
        assert!(disk_items[0].details.contains("76%"));
        assert_eq!(disk_items[0].severity, ReflectionSeverity::Warning);
    }

    #[test]
    fn test_service_failure_trend() {
        let now = Utc::now();
        let day_ago = now - Duration::hours(24);

        let mut old_event = HistoryEvent::new();
        old_event.timestamp_utc = day_ago;
        old_event.failed_services_count = 0;

        let mut new_event = HistoryEvent::new();
        new_event.timestamp_utc = now;
        new_event.failed_services_count = 3;

        let events = vec![old_event, new_event];
        let mut summary = ReflectionSummary::empty();
        extract_historian_trends(&mut summary, &events);

        let service_items: Vec<_> = summary
            .items
            .iter()
            .filter(|i| i.category == "services")
            .collect();
        assert_eq!(service_items.len(), 1);
        assert!(service_items[0].title.contains("3 failed"));
    }

    #[test]
    fn test_network_degradation_trend() {
        let now = Utc::now();
        let day_ago = now - Duration::hours(24);

        let mut old_event = HistoryEvent::new();
        old_event.timestamp_utc = day_ago;
        old_event.network_packet_loss_pct = 0;

        let mut new_event = HistoryEvent::new();
        new_event.timestamp_utc = now;
        new_event.network_packet_loss_pct = 5;

        let events = vec![old_event, new_event];
        let mut summary = ReflectionSummary::empty();
        extract_historian_trends(&mut summary, &events);

        let network_items: Vec<_> = summary
            .items
            .iter()
            .filter(|i| i.category == "network")
            .collect();
        assert_eq!(network_items.len(), 1);
        assert!(network_items[0].details.contains("5%"));
    }

    #[test]
    fn test_kernel_change_reflection() {
        let now = Utc::now();

        let mut event = HistoryEvent::new();
        event.timestamp_utc = now;
        event.kernel_changed = true;
        event.kernel_version = "6.6.10-arch1-1".to_string();

        let events = vec![event.clone(), event];
        let mut summary = ReflectionSummary::empty();
        extract_historian_trends(&mut summary, &events);

        let kernel_items: Vec<_> = summary
            .items
            .iter()
            .filter(|i| i.category == "kernel")
            .collect();
        assert_eq!(kernel_items.len(), 1);
        assert!(kernel_items[0].title.contains("6.6.10"));
    }
}
