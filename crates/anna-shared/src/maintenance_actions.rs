//! Proactive maintenance actions (v0.0.286).
//!
//! Turns health observations into concrete, actionable maintenance suggestions.
//! These are specific actions Anna can help execute, not just passive tips.
//!
//! v0.0.286: Initial implementation.

use crate::roster::{person_for, Tier};
use crate::snapshot::SystemSnapshot;
use crate::system_telemetry::TelemetryStore;
use crate::teams::Team;
use serde::{Deserialize, Serialize};

/// A concrete maintenance action that can be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceAction {
    /// Unique action identifier
    pub id: String,
    /// Brief title
    pub title: String,
    /// What this action does
    pub description: String,
    /// Command or query to ask Anna
    pub anna_query: String,
    /// Urgency level (1=critical, 5=optional)
    pub urgency: u8,
    /// Category for grouping
    pub category: ActionCategory,
    /// Estimated impact (helpful context)
    pub estimated_impact: Option<String>,
}

/// Categories of maintenance actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCategory {
    DiskCleanup,
    MemoryOptimize,
    ServiceRepair,
    SecurityAudit,
    PerformanceTune,
    SystemUpdate,
}

impl std::fmt::Display for ActionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DiskCleanup => write!(f, "disk"),
            Self::MemoryOptimize => write!(f, "memory"),
            Self::ServiceRepair => write!(f, "services"),
            Self::SecurityAudit => write!(f, "security"),
            Self::PerformanceTune => write!(f, "performance"),
            Self::SystemUpdate => write!(f, "updates"),
        }
    }
}

/// Generate maintenance actions from current system state
pub fn generate_maintenance_actions(
    snapshot: &SystemSnapshot,
    telemetry: Option<&TelemetryStore>,
) -> Vec<MaintenanceAction> {
    let mut actions = Vec::new();

    // Disk-based actions
    actions.extend(disk_actions(snapshot));

    // Memory-based actions
    actions.extend(memory_actions(snapshot));

    // Service-based actions
    actions.extend(service_actions(snapshot));

    // Telemetry-based actions
    if let Some(store) = telemetry {
        actions.extend(telemetry_actions(store));
    }

    // Sort by urgency
    actions.sort_by_key(|a| a.urgency);

    // Limit to most urgent
    actions.truncate(5);

    actions
}

/// Disk cleanup actions
fn disk_actions(snapshot: &SystemSnapshot) -> Vec<MaintenanceAction> {
    let mut actions = Vec::new();

    for (mount, use_percent) in &snapshot.disk {
        if *use_percent >= 90 {
            actions.push(MaintenanceAction {
                id: format!("disk-clean-{}", mount.replace('/', "_")),
                title: format!("Clean up {}", mount),
                description: format!("{} is {}% full - critically low space.", mount, use_percent),
                anna_query: format!("Help me clean up {}", mount),
                urgency: 1,
                category: ActionCategory::DiskCleanup,
                estimated_impact: Some("Could free several GB".to_string()),
            });
        } else if *use_percent >= 80 {
            actions.push(MaintenanceAction {
                id: format!("disk-review-{}", mount.replace('/', "_")),
                title: format!("Review space on {}", mount),
                description: format!("{} is {}% full - getting tight.", mount, use_percent),
                anna_query: format!("What's using space on {}?", mount),
                urgency: 3,
                category: ActionCategory::DiskCleanup,
                estimated_impact: Some("Identify space hogs".to_string()),
            });
        }
    }

    // Check for common cleanup opportunities
    if actions.is_empty() {
        // Still suggest periodic cleanup
        actions.push(MaintenanceAction {
            id: "disk-routine-cleanup".to_string(),
            title: "Routine disk cleanup".to_string(),
            description: "Clear package caches, old logs, and temp files.".to_string(),
            anna_query: "Clean up system caches and old logs".to_string(),
            urgency: 5,
            category: ActionCategory::DiskCleanup,
            estimated_impact: Some("Usually frees 100MB-1GB".to_string()),
        });
    }

    actions
}

/// Memory optimization actions
fn memory_actions(snapshot: &SystemSnapshot) -> Vec<MaintenanceAction> {
    let mut actions = Vec::new();

    if snapshot.memory_total_bytes > 0 {
        let used_percent =
            (snapshot.memory_used_bytes as f64 / snapshot.memory_total_bytes as f64 * 100.0) as u8;

        if used_percent >= 90 {
            actions.push(MaintenanceAction {
                id: "memory-critical".to_string(),
                title: "Free up memory".to_string(),
                description: format!("Memory at {}% - system may be swapping.", used_percent),
                anna_query: "What's using all my memory?".to_string(),
                urgency: 1,
                category: ActionCategory::MemoryOptimize,
                estimated_impact: Some("Identify memory-heavy processes".to_string()),
            });
        } else if used_percent >= 75 {
            actions.push(MaintenanceAction {
                id: "memory-review".to_string(),
                title: "Review memory usage".to_string(),
                description: format!("Memory at {}% - worth monitoring.", used_percent),
                anna_query: "Show me memory usage by process".to_string(),
                urgency: 4,
                category: ActionCategory::MemoryOptimize,
                estimated_impact: None,
            });
        }
    }

    actions
}

/// Service repair actions
fn service_actions(snapshot: &SystemSnapshot) -> Vec<MaintenanceAction> {
    let mut actions = Vec::new();

    for service in &snapshot.failed_services {
        let person = person_for(Team::Services, Tier::Senior);
        actions.push(MaintenanceAction {
            id: format!("service-repair-{}", service),
            title: format!("Fix {}", service),
            description: format!("{} says {} is failed.", person.display_name, service),
            anna_query: format!("Why did {} fail and how do I fix it?", service),
            urgency: 2,
            category: ActionCategory::ServiceRepair,
            estimated_impact: Some("Restore service functionality".to_string()),
        });
    }

    actions
}

/// Telemetry-based actions
fn telemetry_actions(store: &TelemetryStore) -> Vec<MaintenanceAction> {
    let mut actions = Vec::new();

    // Health score actions
    let score = store.health_score();
    if score < 60 {
        actions.push(MaintenanceAction {
            id: "health-diagnostic".to_string(),
            title: "Run system diagnostic".to_string(),
            description: format!("Health score {}% - multiple issues detected.", score),
            anna_query: "Diagnose my system health".to_string(),
            urgency: 2,
            category: ActionCategory::PerformanceTune,
            estimated_impact: Some("Comprehensive health report".to_string()),
        });
    }

    // Trend-based actions
    if store.trends.sample_count >= 10 {
        if store.trends.disk_trend > 10.0 {
            actions.push(MaintenanceAction {
                id: "disk-trend-alert".to_string(),
                title: "Address disk growth".to_string(),
                description: format!(
                    "Disk usage growing {:.1}% over time.",
                    store.trends.disk_trend
                ),
                anna_query: "What's causing my disk to fill up over time?".to_string(),
                urgency: 3,
                category: ActionCategory::DiskCleanup,
                estimated_impact: Some("Find growing files/directories".to_string()),
            });
        }

        if store.trends.memory_trend > 20.0 {
            actions.push(MaintenanceAction {
                id: "memory-trend-alert".to_string(),
                title: "Check for memory leak".to_string(),
                description: format!(
                    "Memory trending up {:.1}% - possible leak.",
                    store.trends.memory_trend
                ),
                anna_query: "Check for memory leaks".to_string(),
                urgency: 3,
                category: ActionCategory::MemoryOptimize,
                estimated_impact: Some("Identify leaking processes".to_string()),
            });
        }
    }

    // Security-related from anomalies
    for anomaly in store.recent_anomalies().iter().take(2) {
        use crate::system_telemetry::AnomalyCategory;

        if matches!(anomaly.category, AnomalyCategory::NetworkError) {
            actions.push(MaintenanceAction {
                id: "network-check".to_string(),
                title: "Check network health".to_string(),
                description: anomaly.description.clone(),
                anna_query: "Is my network working properly?".to_string(),
                urgency: 3,
                category: ActionCategory::SecurityAudit,
                estimated_impact: Some("Verify connectivity".to_string()),
            });
            break; // Only one network action
        }
    }

    actions
}

/// Format actions for display
pub fn format_actions_for_display(actions: &[MaintenanceAction]) -> String {
    if actions.is_empty() {
        return String::new();
    }

    let mut output = String::from("Suggested maintenance:\n");

    for (i, action) in actions.iter().take(3).enumerate() {
        let urgency_marker = match action.urgency {
            1 => "[!!]",
            2 => "[! ]",
            3 => "[* ]",
            _ => "[  ]",
        };

        output.push_str(&format!(
            "  {}. {} {}\n",
            i + 1,
            urgency_marker,
            action.title
        ));
        output.push_str(&format!("       Ask: \"{}\"\n", action.anna_query));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_snapshot() -> SystemSnapshot {
        let mut snapshot = SystemSnapshot::default();
        snapshot.disk.insert("/".to_string(), 50);
        snapshot.memory_total_bytes = 8_000_000_000;
        snapshot.memory_used_bytes = 4_000_000_000;
        snapshot
    }

    #[test]
    fn test_disk_actions_critical() {
        let mut snapshot = test_snapshot();
        snapshot.disk.insert("/".to_string(), 95);

        let actions = disk_actions(&snapshot);
        assert!(actions.iter().any(|a| a.urgency == 1));
    }

    #[test]
    fn test_service_actions() {
        let mut snapshot = test_snapshot();
        snapshot.failed_services.push("nginx.service".to_string());

        let actions = service_actions(&snapshot);
        assert!(!actions.is_empty());
        assert!(actions[0].anna_query.contains("nginx"));
    }

    #[test]
    fn test_format_actions() {
        let actions = vec![MaintenanceAction {
            id: "test".to_string(),
            title: "Test action".to_string(),
            description: "Description".to_string(),
            anna_query: "test query".to_string(),
            urgency: 2,
            category: ActionCategory::DiskCleanup,
            estimated_impact: None,
        }];

        let formatted = format_actions_for_display(&actions);
        assert!(formatted.contains("Test action"));
        assert!(formatted.contains("test query"));
    }
}
