//! Root cause analysis engine.

use super::types::{RootCauseAnalysis, RootCause, SystemEvent, EventType, DependencyGraph};
use tracing::{debug, info};

/// Analyze a symptom to find root causes
pub fn analyze_symptom(symptom: &str, recent_events: &[SystemEvent], dep_graph: &DependencyGraph) -> RootCauseAnalysis {
    let mut root_causes = Vec::new();

    debug!("Analyzing symptom: {}", symptom);

    // Pattern 1: Service failure due to dependency
    if symptom.contains("failed") {
        if let Some(service_name) = extract_service_name(symptom) {
            let dependencies = dep_graph.find_dependencies(&service_name);

            for dep in dependencies {
                // Check if dependency recently failed
                let dep_failed = recent_events.iter().any(|e| {
                    matches!(e.event_type, EventType::ServiceFailed) && e.component == dep.name
                });

                if dep_failed {
                    root_causes.push(RootCause {
                        description: format!("{} failed because dependency {} is not running", service_name, dep.name),
                        confidence: 0.85,
                        evidence: vec![
                            format!("{} depends on {}", service_name, dep.name),
                            format!("{} failed recently", dep.name),
                        ],
                        recommended_action: format!("Start {} first, then restart {}", dep.name, service_name),
                    });
                }
            }
        }
    }

    // Pattern 2: Resource exhaustion
    let high_cpu = recent_events.iter().any(|e| matches!(e.event_type, EventType::HighCpuUsage));
    let high_memory = recent_events.iter().any(|e| matches!(e.event_type, EventType::HighMemoryUsage));

    if (symptom.contains("slow") || symptom.contains("timeout")) && (high_cpu || high_memory) {
        root_causes.push(RootCause {
            description: "System resource exhaustion causing slowdowns".to_string(),
            confidence: 0.75,
            evidence: vec![
                if high_cpu { "High CPU usage detected" } else { "High memory usage detected" }.to_string(),
                "Performance degradation symptoms present".to_string(),
            ],
            recommended_action: "Identify and stop resource-heavy processes".to_string(),
        });
    }

    // Pattern 3: Configuration change correlation
    let recent_config_changes: Vec<_> = recent_events.iter()
        .filter(|e| matches!(e.event_type, EventType::ConfigChanged))
        .collect();

    if !recent_config_changes.is_empty() {
        for change in recent_config_changes {
            root_causes.push(RootCause {
                description: format!("Recent configuration change in {}", change.component),
                confidence: 0.65,
                evidence: vec![
                    format!("Config changed: {}", change.details),
                    "Symptoms appeared shortly after change".to_string(),
                ],
                recommended_action: "Review and possibly revert recent configuration changes".to_string(),
            });
        }
    }

    // Pattern 4: Package installation correlation
    let recent_packages: Vec<_> = recent_events.iter()
        .filter(|e| matches!(e.event_type, EventType::PackageInstalled | EventType::PackageRemoved))
        .collect();

    if !recent_packages.is_empty() {
        for pkg in recent_packages {
            root_causes.push(RootCause {
                description: format!("Package change: {}", pkg.details),
                confidence: 0.55,
                evidence: vec![
                    format!("Package operation: {}", pkg.details),
                    "Issue appeared after package change".to_string(),
                ],
                recommended_action: "Check package logs and consider rolling back".to_string(),
            });
        }
    }

    // Sort by confidence
    root_causes.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    RootCauseAnalysis {
        symptom: symptom.to_string(),
        root_causes,
        analysis_timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

/// Extract service name from error message
fn extract_service_name(message: &str) -> Option<String> {
    // Try to find service names (ending in .service)
    message.split_whitespace()
        .find(|word| word.ends_with(".service"))
        .map(|s| s.to_string())
}

/// Collect recent system events from logs
pub fn collect_recent_events(hours: u64) -> Vec<SystemEvent> {
    let mut events = Vec::new();

    // Get systemd journal events
    if let Ok(output) = std::process::Command::new("journalctl")
        .args([
            "--since",
            &format!("{} hours ago", hours),
            "-o", "json",
            "-n", "500",
            "--no-pager"
        ])
        .output()
    {
        let logs = String::from_utf8_lossy(&output.stdout);

        for line in logs.lines() {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = entry.get("__REALTIME_TIMESTAMP")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let message = entry.get("MESSAGE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let unit = entry.get("UNIT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("system");

                // Classify event
                let event_type = classify_event(message);

                events.push(SystemEvent {
                    timestamp,
                    event_type,
                    component: unit.to_string(),
                    details: message.chars().take(200).collect(),
                });
            }
        }
    }

    info!("Collected {} recent events", events.len());
    events
}

/// Classify an event based on message content
fn classify_event(message: &str) -> EventType {
    let lower = message.to_lowercase();

    if lower.contains("failed") || lower.contains("failure") {
        EventType::ServiceFailed
    } else if lower.contains("started") || lower.contains("starting") {
        EventType::ServiceStarted
    } else if lower.contains("stopped") || lower.contains("stopping") {
        EventType::ServiceStopped
    } else if lower.contains("high cpu") || lower.contains("cpu usage") {
        EventType::HighCpuUsage
    } else if lower.contains("out of memory") || lower.contains("oom") {
        EventType::HighMemoryUsage
    } else if lower.contains("disk full") || lower.contains("no space") {
        EventType::DiskFull
    } else if lower.contains("network") && (lower.contains("error") || lower.contains("timeout")) {
        EventType::NetworkError
    } else if lower.contains("installed") {
        EventType::PackageInstalled
    } else if lower.contains("removed") {
        EventType::PackageRemoved
    } else if lower.contains("config") {
        EventType::ConfigChanged
    } else {
        EventType::LogError
    }
}
