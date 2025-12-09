//! Health summary builder functions (v0.0.210).

use crate::snapshot::{
    SystemSnapshot, DISK_CRITICAL_THRESHOLD, DISK_WARN_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};

use super::summary::RelevantHealthSummary;
use super::types::{HealthCategory, HealthChange, HealthItem};

/// Build a relevant health summary from a snapshot
pub fn build_health_summary(
    snapshot: &SystemSnapshot,
    prev_snapshot: Option<&SystemSnapshot>,
) -> RelevantHealthSummary {
    let mut summary = RelevantHealthSummary::default();

    // Check disk usage
    for (mount, &pct) in &snapshot.disk {
        if pct >= DISK_CRITICAL_THRESHOLD {
            summary.add_critical(HealthItem::critical(
                HealthCategory::Disk,
                format!("Disk {} is CRITICAL at {}% used", mount, pct),
                pct as u32,
            ));
        } else if pct >= DISK_WARN_THRESHOLD {
            summary.add_warning(HealthItem::warning(
                HealthCategory::Disk,
                format!("Disk {} is at {}% used", mount, pct),
                pct as u32,
            ));
        }
    }

    // Check memory
    let mem_pct = snapshot.memory_percent();
    if mem_pct >= MEMORY_HIGH_THRESHOLD {
        summary.add_warning(HealthItem::warning(
            HealthCategory::Memory,
            format!("Memory usage is high at {}%", mem_pct),
            mem_pct as u32,
        ));
    }

    // Check failed services
    if !snapshot.failed_services.is_empty() {
        for svc in &snapshot.failed_services {
            summary.add_critical(HealthItem::critical(
                HealthCategory::Services,
                format!("Service {} is failed", svc),
                0, // services sorted alphabetically
            ));
        }
    }

    // Check for changes since last snapshot
    if let Some(prev) = prev_snapshot {
        // New failed services
        for svc in &snapshot.failed_services {
            if !prev.failed_services.contains(svc) {
                summary.add_change(HealthChange {
                    description: format!("Service {} started failing", svc),
                    positive: false,
                });
            }
        }
        // Recovered services
        for svc in &prev.failed_services {
            if !snapshot.failed_services.contains(svc) {
                summary.add_change(HealthChange {
                    description: format!("Service {} recovered", svc),
                    positive: true,
                });
            }
        }
        // Disk usage increased significantly
        for (mount, &curr_pct) in &snapshot.disk {
            if let Some(&prev_pct) = prev.disk.get(mount) {
                if curr_pct >= prev_pct + 5 && curr_pct >= DISK_WARN_THRESHOLD {
                    summary.add_change(HealthChange {
                        description: format!(
                            "Disk {} increased from {}% to {}%",
                            mount, prev_pct, curr_pct
                        ),
                        positive: false,
                    });
                }
            }
        }
    }

    // Set nothing_to_report flag
    summary.nothing_to_report = summary.critical.is_empty() && summary.warnings.is_empty();

    // Sort deterministically
    summary.sort();

    summary
}

/// Quick check if snapshot has any issues worth reporting
pub fn has_health_issues(snapshot: &SystemSnapshot) -> bool {
    // Check disk
    for &pct in snapshot.disk.values() {
        if pct >= DISK_WARN_THRESHOLD {
            return true;
        }
    }
    // Check memory
    if snapshot.memory_percent() >= MEMORY_HIGH_THRESHOLD {
        return true;
    }
    // Check services
    !snapshot.failed_services.is_empty()
}
