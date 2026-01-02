//! Snapshot-based health checks (disk, memory, services).
//!
//! Analyzes the current system snapshot and generates health tips
//! for disk usage, memory pressure, and failed services.

use crate::idle_tips::{IdleTip, TipCategory};
use crate::roster::{person_for, Tier};
use crate::snapshot::{DeltaItem, SystemSnapshot};
use crate::system_telemetry::TelemetryStore;
use crate::teams::Team;

use super::delta_tips::tip_from_delta;
use super::telemetry_tips::generate_telemetry_tips;

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
    tips.extend(check_disk_usage(snapshot));

    // Check memory state (memory_total_bytes and memory_used_bytes)
    tips.extend(check_memory_usage(snapshot));

    // Check for failed services
    tips.extend(check_failed_services(snapshot));

    // v0.0.285: Add telemetry-based trend tips
    if let Some(telemetry) = TelemetryStore::load_if_exists() {
        tips.extend(generate_telemetry_tips(&telemetry));
    }

    tips
}

/// Check disk usage across all mounts
fn check_disk_usage(snapshot: &SystemSnapshot) -> Vec<IdleTip> {
    let mut tips = Vec::new();

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

    tips
}

/// Check memory usage
fn check_memory_usage(snapshot: &SystemSnapshot) -> Vec<IdleTip> {
    let mut tips = Vec::new();

    if snapshot.memory_total_bytes > 0 {
        let used_percent =
            (snapshot.memory_used_bytes as f64 / snapshot.memory_total_bytes as f64 * 100.0) as u8;

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

    tips
}

/// Check for failed services
fn check_failed_services(snapshot: &SystemSnapshot) -> Vec<IdleTip> {
    let mut tips = Vec::new();

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
