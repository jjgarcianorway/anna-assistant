//! Delta-based tip generation.
//!
//! Converts system state deltas (changes) into actionable idle tips.
//! Handles disk changes, memory changes, and service state transitions.

use crate::idle_tips::{IdleTip, TipCategory};
use crate::roster::{person_for, Tier};
use crate::snapshot::DeltaItem;
use crate::teams::Team;

/// Convert a health delta to an idle tip
pub fn tip_from_delta(delta: &DeltaItem) -> Option<IdleTip> {
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
