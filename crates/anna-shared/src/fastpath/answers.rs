//! Fast path answer generators (v0.0.185).

use crate::health_view::{build_health_summary, has_health_issues};
use crate::snapshot::{
    diff_snapshots, format_deltas_text, has_actionable_deltas, load_last_snapshot, DeltaItem,
    SystemSnapshot, DISK_CRITICAL_THRESHOLD, DISK_WARN_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};
use crate::trace::EvidenceKind;

use super::types::{FastPathAnswer, FastPathClass};

/// Answer system health from snapshot using RelevantHealthSummary (v0.0.40)
/// Only shows actionable issues - short "no issues" when healthy.
pub fn answer_system_health(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    // Use the new RelevantHealthSummary for minimal, actionable output
    let summary = build_health_summary(snapshot, None);

    // Build evidence list based on what was checked
    let mut evidence = vec![EvidenceKind::Memory];
    if !snapshot.disk.is_empty() {
        evidence.push(EvidenceKind::Disk);
    }
    if !snapshot.failed_services.is_empty() || has_health_issues(snapshot) {
        evidence.push(EvidenceKind::FailedUnits);
    }

    let reliability = if summary.nothing_to_report { 90 } else { 85 };

    FastPathAnswer::handled(
        FastPathClass::SystemHealth,
        summary.format(),
        evidence,
        "answered from fresh snapshot (relevant health)",
        reliability,
        false, // no probes run
    )
}

/// Answer disk usage from snapshot
pub fn answer_disk_usage(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    if snapshot.disk.is_empty() {
        return FastPathAnswer::not_handled("no disk data in snapshot");
    }

    let mut lines = vec!["**Disk Usage:**".to_string()];
    for (mount, &pct) in &snapshot.disk {
        let status = if pct >= DISK_CRITICAL_THRESHOLD {
            "CRITICAL"
        } else if pct >= DISK_WARN_THRESHOLD {
            "WARNING"
        } else {
            "OK"
        };
        lines.push(format!("  {} - {}% used [{}]", mount, pct, status));
    }

    FastPathAnswer::handled(
        FastPathClass::DiskUsage,
        lines.join("\n"),
        vec![EvidenceKind::Disk],
        "answered from fresh snapshot",
        88,
        false,
    )
}

/// Answer memory usage from snapshot
pub fn answer_memory_usage(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    if snapshot.memory_total_bytes == 0 {
        return FastPathAnswer::not_handled("no memory data in snapshot");
    }

    let total_gb = snapshot.memory_total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_gb = snapshot.memory_used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    let pct = snapshot.memory_percent();

    let status = if pct >= MEMORY_HIGH_THRESHOLD {
        "HIGH"
    } else {
        "OK"
    };

    let answer = format!(
        "**Memory Usage:**\n  {:.1} GB / {:.1} GB ({}%) [{}]",
        used_gb, total_gb, pct, status
    );

    FastPathAnswer::handled(
        FastPathClass::MemoryUsage,
        answer,
        vec![EvidenceKind::Memory],
        "answered from fresh snapshot",
        88,
        false,
    )
}

/// Answer failed services from snapshot
pub fn answer_failed_services(snapshot: &SystemSnapshot, is_fresh: bool) -> FastPathAnswer {
    if !is_fresh {
        return FastPathAnswer::not_handled("snapshot stale, probes needed");
    }

    let answer = if snapshot.failed_services.is_empty() {
        "No failed services. All systemd units are running normally.".to_string()
    } else {
        let mut lines = vec![format!(
            "**{} Failed Service(s):**",
            snapshot.failed_services.len()
        )];
        for svc in &snapshot.failed_services {
            lines.push(format!("  🔴 {}", svc));
        }
        lines.join("\n")
    };

    FastPathAnswer::handled(
        FastPathClass::FailedServices,
        answer,
        vec![EvidenceKind::FailedUnits],
        "answered from fresh snapshot",
        90,
        false,
    )
}

/// Answer "what changed since last time"
pub fn answer_what_changed(current: &SystemSnapshot) -> FastPathAnswer {
    // Load previous snapshot for comparison
    let prev = match load_last_snapshot() {
        Some(s) => s,
        None => {
            return FastPathAnswer::handled(
                FastPathClass::WhatChanged,
                "No previous snapshot available for comparison. This is the first check."
                    .to_string(),
                vec![],
                "no previous snapshot",
                75,
                false,
            );
        }
    };

    let deltas = diff_snapshots(&prev, current);

    if deltas.is_empty() {
        return FastPathAnswer::handled(
            FastPathClass::WhatChanged,
            "No significant changes since last check.".to_string(),
            vec![],
            "no deltas detected",
            85,
            false,
        );
    }

    // Collect evidence kinds from deltas
    let mut evidence = Vec::new();
    for delta in &deltas {
        match delta {
            DeltaItem::DiskWarning { .. }
            | DeltaItem::DiskCritical { .. }
            | DeltaItem::DiskIncreased { .. } => {
                if !evidence.contains(&EvidenceKind::Disk) {
                    evidence.push(EvidenceKind::Disk);
                }
            }
            DeltaItem::NewFailedService { .. } | DeltaItem::ServiceRecovered { .. } => {
                if !evidence.contains(&EvidenceKind::FailedUnits) {
                    evidence.push(EvidenceKind::FailedUnits);
                }
            }
            DeltaItem::MemoryHigh { .. } | DeltaItem::MemoryIncreased { .. } => {
                if !evidence.contains(&EvidenceKind::Memory) {
                    evidence.push(EvidenceKind::Memory);
                }
            }
        }
    }

    let answer = format_deltas_text(&deltas);
    let reliability = if has_actionable_deltas(&deltas) {
        85
    } else {
        80
    };

    FastPathAnswer::handled(
        FastPathClass::WhatChanged,
        answer,
        evidence,
        &format!("{} changes detected", deltas.len()),
        reliability,
        false,
    )
}
