//! Context-aware greeting insights from system state (v0.0.245).
//!
//! Enriches Anna's greetings with observations about the system state,
//! making her feel more aware and proactive without being annoying.
//!
//! v0.0.245: Initial implementation.

use crate::roster::{person_for, Tier};
use crate::snapshot::{DeltaItem, SystemSnapshot};
use crate::teams::Team;

/// A greeting insight that can be added to Anna's welcome
#[derive(Debug, Clone)]
pub struct GreetingInsight {
    /// The staff member delivering this insight
    pub staff_name: &'static str,
    /// Team the insight is from
    pub team: Team,
    /// The insight message
    pub message: String,
    /// How urgent is this (affects display order)
    pub priority: u8,
    /// Is this good news or concerning?
    pub positive: bool,
}

/// Generate greeting insights from system snapshot
pub fn generate_insights(
    snapshot: &SystemSnapshot,
    deltas: &[DeltaItem],
) -> Vec<GreetingInsight> {
    let mut insights = Vec::new();

    // Check for critical issues first
    add_disk_insights(snapshot, &mut insights);
    add_memory_insights(snapshot, &mut insights);
    add_service_insights(snapshot, &mut insights);
    add_delta_insights(deltas, &mut insights);

    // Sort by priority (highest first)
    insights.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Limit to top 2 insights for greeting (don't overwhelm)
    insights.truncate(2);

    insights
}

fn add_disk_insights(snapshot: &SystemSnapshot, insights: &mut Vec<GreetingInsight>) {
    for (mount, percent) in &snapshot.disk {
        if *percent >= 95 {
            let person = person_for(Team::Storage, Tier::Senior);
            insights.push(GreetingInsight {
                staff_name: person.display_name,
                team: Team::Storage,
                message: format!(
                    "{} is at {}% - pool status is concerning",
                    mount, percent
                ),
                priority: 95,
                positive: false,
            });
        } else if *percent >= 85 {
            let person = person_for(Team::Storage, Tier::Junior);
            insights.push(GreetingInsight {
                staff_name: person.display_name,
                team: Team::Storage,
                message: format!("{} at {}%, keeping an eye on it", mount, percent),
                priority: 70,
                positive: false,
            });
        }
    }

    // Check for good news - lots of free space
    let all_low = snapshot.disk.values().all(|p| *p < 50);
    if all_low && !snapshot.disk.is_empty() {
        let person = person_for(Team::Storage, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Storage,
            message: "plenty of space on all drives!".to_string(),
            priority: 20,
            positive: true,
        });
    }
}

fn add_memory_insights(snapshot: &SystemSnapshot, insights: &mut Vec<GreetingInsight>) {
    if snapshot.memory_total_bytes == 0 {
        return;
    }

    let used_percent = (snapshot.memory_used_bytes as f64
        / snapshot.memory_total_bytes as f64
        * 100.0) as u8;

    if used_percent >= 90 {
        let person = person_for(Team::Performance, Tier::Senior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: format!("memory at {}% - that's 90% efficiency... wait", used_percent),
            priority: 90,
            positive: false,
        });
    } else if used_percent >= 80 {
        let person = person_for(Team::Performance, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: format!("*glances at htop* memory at {}%", used_percent),
            priority: 60,
            positive: false,
        });
    } else if used_percent < 30 {
        let person = person_for(Team::Performance, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: "numbers look good, no bottlenecks!".to_string(),
            priority: 15,
            positive: true,
        });
    }
}

fn add_service_insights(snapshot: &SystemSnapshot, insights: &mut Vec<GreetingInsight>) {
    let count = snapshot.failed_services.len();
    if count > 0 {
        let person = person_for(Team::Services, Tier::Senior);
        let msg = if count == 1 {
            format!("{} is in failed state", snapshot.failed_services[0])
        } else {
            format!("{} services in failed state", count)
        };
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Services,
            message: msg,
            priority: 85,
            positive: false,
        });
    }
}

fn add_delta_insights(deltas: &[DeltaItem], insights: &mut Vec<GreetingInsight>) {
    for delta in deltas {
        match delta {
            DeltaItem::ServiceRecovered { unit } => {
                let person = person_for(Team::Services, Tier::Senior);
                insights.push(GreetingInsight {
                    staff_name: person.display_name,
                    team: Team::Services,
                    message: format!("{} is back up!", unit),
                    priority: 50,
                    positive: true,
                });
            }
            DeltaItem::DiskCritical { mount, curr, .. } => {
                let person = person_for(Team::Storage, Tier::Senior);
                insights.push(GreetingInsight {
                    staff_name: person.display_name,
                    team: Team::Storage,
                    message: format!("{} just hit {}%!", mount, curr),
                    priority: 95,
                    positive: false,
                });
            }
            DeltaItem::NewFailedService { unit } => {
                let person = person_for(Team::Services, Tier::Junior);
                insights.push(GreetingInsight {
                    staff_name: person.display_name,
                    team: Team::Services,
                    message: format!("{} just failed!", unit),
                    priority: 85,
                    positive: false,
                });
            }
            _ => {} // Other deltas handled elsewhere
        }
    }
}

/// Format insights for display in greeting
pub fn format_insights_for_greeting(insights: &[GreetingInsight]) -> Option<String> {
    if insights.is_empty() {
        return None;
    }

    let mut output = String::new();

    if insights.len() == 1 {
        let insight = &insights[0];
        if insight.positive {
            output.push_str(&format!(
                "Heads up from {}: {}",
                insight.staff_name, insight.message
            ));
        } else {
            output.push_str(&format!(
                "Quick note from {}: {}",
                insight.staff_name, insight.message
            ));
        }
    } else {
        output.push_str("A few things to note:\n");
        for insight in insights {
            let icon = if insight.positive { "✓" } else { "•" };
            output.push_str(&format!(
                "  {} {} says: {}\n",
                icon, insight.staff_name, insight.message
            ));
        }
    }

    Some(output)
}

/// Get a one-liner status summary
pub fn quick_status_line(snapshot: &SystemSnapshot) -> String {
    let disk_status = snapshot
        .disk
        .values()
        .max()
        .map(|p| {
            if *p >= 90 {
                "disks critical"
            } else if *p >= 80 {
                "disks busy"
            } else {
                "disks ok"
            }
        })
        .unwrap_or("disks ok");

    let mem_status = if snapshot.memory_total_bytes > 0 {
        let pct = (snapshot.memory_used_bytes as f64 / snapshot.memory_total_bytes as f64 * 100.0)
            as u8;
        if pct >= 90 {
            "memory high"
        } else if pct >= 80 {
            "memory busy"
        } else {
            "memory ok"
        }
    } else {
        "memory ok"
    };

    let svc_status = if snapshot.failed_services.is_empty() {
        "services ok"
    } else {
        "services need attention"
    };

    format!("{} • {} • {}", disk_status, mem_status, svc_status)
}

#[cfg(test)]
mod tests {
    use super::*;
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
            captured_at: 0,
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
            captured_at: 0,
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
            captured_at: 0,
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
}
