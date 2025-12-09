//! Proactive health tips from system state (v0.0.244).
//!
//! Generates idle tips based on actual system health - disk usage,
//! memory pressure, failed services, etc. These tips surface during
//! REPL idle time to help users before they even ask.
//!
//! v0.0.244: Initial implementation.

use crate::idle_tips::{IdleTip, TipCategory};
use crate::roster::{person_for, Tier};
use crate::snapshot::{DeltaItem, SystemSnapshot};
use crate::teams::Team;

/// Generate health-based tips from current system state and deltas
pub fn generate_health_tips(
    snapshot: &SystemSnapshot,
    deltas: &[DeltaItem],
    _failed_services: &[String],
) -> Vec<IdleTip> {
    let mut tips = Vec::new();

    // Process health deltas into proactive tips
    for delta in deltas {
        if let Some(tip) = tip_from_delta(delta) {
            tips.push(tip);
        }
    }

    // Check current disk state for warnings (disk is BTreeMap<String, u8>)
    for (mount, use_percent) in &snapshot.disk {
        if *use_percent >= 90 {
            let person = person_for(Team::Storage, Tier::Senior);
            tips.push(
                IdleTip::new(
                    format!("health-disk-critical-{}", mount),
                    TipCategory::Storage,
                    format!(
                        "{} here from Storage. {} is at {}% - that's critically full. \
                         We should clean up or expand storage soon.",
                        person.display_name, mount, use_percent
                    ),
                )
                .with_action(format!(
                    "Ask me: \"what's using space on {}\" or \"help me clean up {}\"",
                    mount, mount
                ))
                .with_priority(95),
            );
        } else if *use_percent >= 80 {
            let person = person_for(Team::Storage, Tier::Junior);
            tips.push(
                IdleTip::new(
                    format!("health-disk-warning-{}", mount),
                    TipCategory::Storage,
                    format!(
                        "Hi! {} from Storage here. {} is at {}%. Not critical yet, \
                         but worth keeping an eye on.",
                        person.display_name, mount, use_percent
                    ),
                )
                .with_action("Ask me: \"show disk usage\" for details")
                .with_priority(70),
            );
        }
    }

    // Check memory state (memory_total_bytes and memory_used_bytes)
    if snapshot.memory_total_bytes > 0 {
        let used_percent = (snapshot.memory_used_bytes as f64
            / snapshot.memory_total_bytes as f64
            * 100.0) as u8;

        if used_percent >= 90 {
            let person = person_for(Team::Performance, Tier::Senior);
            tips.push(
                IdleTip::new(
                    "health-memory-critical",
                    TipCategory::Performance,
                    format!(
                        "{} checking in. Memory is at {}% - system might be swapping. \
                         Consider closing some applications.",
                        person.display_name, used_percent
                    ),
                )
                .with_action("Ask me: \"what's using memory\" to investigate")
                .with_priority(90),
            );
        } else if used_percent >= 80 {
            let person = person_for(Team::Performance, Tier::Junior);
            tips.push(
                IdleTip::new(
                    "health-memory-warning",
                    TipCategory::Performance,
                    format!(
                        "*glances at htop* {} here! Memory at {}%. Still fine, \
                         but we're getting up there.",
                        person.display_name, used_percent
                    ),
                )
                .with_priority(60),
            );
        }
    }

    // Check for failed services
    if !snapshot.failed_services.is_empty() {
        let person = person_for(Team::Services, Tier::Senior);
        let count = snapshot.failed_services.len();
        let services_list = if count <= 3 {
            snapshot.failed_services.join(", ")
        } else {
            format!(
                "{} and {} more",
                snapshot.failed_services[..2].join(", "),
                count - 2
            )
        };

        tips.push(
            IdleTip::new(
                format!("health-failed-services-{}", count),
                TipCategory::Services,
                format!(
                    "{} here. {} service{} in failed state: {}. \
                     Might want to check on {}.",
                    person.display_name,
                    count,
                    if count == 1 { " is" } else { "s are" },
                    services_list,
                    if count == 1 { "it" } else { "them" }
                ),
            )
            .with_action("Ask me: \"why did [service] fail\" to investigate")
            .with_priority(85),
        );
    }

    tips
}

/// Convert a health delta to an idle tip
fn tip_from_delta(delta: &DeltaItem) -> Option<IdleTip> {
    match delta {
        DeltaItem::DiskCritical { mount, curr, .. } => {
            let person = person_for(Team::Storage, Tier::Senior);
            Some(
                IdleTip::new(
                    format!("delta-disk-critical-{}", mount),
                    TipCategory::Storage,
                    format!(
                        "{} from Storage. {} just hit {}% - that's critical territory. \
                         Pool status is concerning.",
                        person.display_name, mount, curr
                    ),
                )
                .with_action(format!("Ask me: \"help clean up {}\"", mount))
                .with_priority(95),
            )
        }
        DeltaItem::DiskWarning { mount, curr, .. } => {
            let person = person_for(Team::Storage, Tier::Junior);
            Some(
                IdleTip::new(
                    format!("delta-disk-warning-{}", mount),
                    TipCategory::Storage,
                    format!(
                        "How much free space...? {} here. {} is at {}% now.",
                        person.display_name, mount, curr
                    ),
                )
                .with_priority(75),
            )
        }
        DeltaItem::NewFailedService { unit } => {
            let person = person_for(Team::Services, Tier::Junior);
            Some(
                IdleTip::new(
                    format!("delta-service-failed-{}", unit),
                    TipCategory::Services,
                    format!(
                        "Checking systemd status! {} here - {} just failed. \
                         Want me to look into it?",
                        person.display_name, unit
                    ),
                )
                .with_action(format!("Ask me: \"what's wrong with {}\"", unit))
                .with_priority(85),
            )
        }
        DeltaItem::ServiceRecovered { unit } => {
            let person = person_for(Team::Services, Tier::Senior);
            Some(
                IdleTip::new(
                    format!("delta-service-recovered-{}", unit),
                    TipCategory::Services,
                    format!(
                        "{} here. Good news: {} is back up! \
                         All replicas running.",
                        person.display_name, unit
                    ),
                )
                .with_priority(40), // Lower priority - good news can wait
            )
        }
        DeltaItem::MemoryHigh { curr_percent, .. } => {
            let person = person_for(Team::Performance, Tier::Senior);
            Some(
                IdleTip::new(
                    "delta-memory-high",
                    TipCategory::Performance,
                    format!(
                        "Profiling initiated. {} here - memory hit {}%. \
                         That's 90% efficiency... wait, that's bad here.",
                        person.display_name, curr_percent
                    ),
                )
                .with_action("Ask me: \"what's eating memory\"")
                .with_priority(80),
            )
        }
        DeltaItem::MemoryIncreased {
            prev_percent,
            curr_percent,
        } => {
            // Only tip if increase is significant
            if curr_percent - prev_percent >= 10 {
                let person = person_for(Team::Performance, Tier::Junior);
                Some(
                    IdleTip::new(
                        "delta-memory-increased",
                        TipCategory::Performance,
                        format!(
                            "*glances at htop* {} here! Memory jumped from {}% to {}%. \
                             Something's hungry.",
                            person.display_name, prev_percent, curr_percent
                        ),
                    )
                    .with_priority(60),
                )
            } else {
                None
            }
        }
        DeltaItem::DiskIncreased { mount, prev, curr } => {
            // Only tip if increase is significant
            if curr - prev >= 10 {
                let person = person_for(Team::Storage, Tier::Junior);
                Some(
                    IdleTip::new(
                        format!("delta-disk-increased-{}", mount),
                        TipCategory::Storage,
                        format!(
                            "Storage audit time! {} here. {} grew from {}% to {}%.",
                            person.display_name, mount, prev, curr
                        ),
                    )
                    .with_priority(55),
                )
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_critical_tip() {
        let delta = DeltaItem::DiskCritical {
            mount: "/".to_string(),
            prev: 85,
            curr: 95,
        };
        let tip = tip_from_delta(&delta);
        assert!(tip.is_some());
        let tip = tip.unwrap();
        assert_eq!(tip.category, TipCategory::Storage);
        assert!(tip.priority >= 90);
    }

    #[test]
    fn test_service_failed_tip() {
        let delta = DeltaItem::NewFailedService {
            unit: "nginx.service".to_string(),
        };
        let tip = tip_from_delta(&delta);
        assert!(tip.is_some());
        assert!(tip.unwrap().message.contains("nginx.service"));
    }

    #[test]
    fn test_small_memory_increase_no_tip() {
        let delta = DeltaItem::MemoryIncreased {
            prev_percent: 50,
            curr_percent: 55,
        };
        let tip = tip_from_delta(&delta);
        assert!(tip.is_none()); // Less than 10% increase
    }

    #[test]
    fn test_large_memory_increase_tip() {
        let delta = DeltaItem::MemoryIncreased {
            prev_percent: 50,
            curr_percent: 65,
        };
        let tip = tip_from_delta(&delta);
        assert!(tip.is_some());
    }
}
