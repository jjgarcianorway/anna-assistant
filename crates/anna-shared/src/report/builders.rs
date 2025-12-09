//! Report builder functions (v0.0.189).

use crate::parsers::ServiceState;
use crate::reliability::{
    ReliabilityExplanation, DISK_CRITICAL_THRESHOLD, DISK_WARNING_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};
use crate::trace::{ExecutionTrace, ProbeStats};

use super::helpers::{format_bytes, sanitize_mount};
use super::types::{HealthItem, HealthSeverity, ReportEvidence, SystemInventory, SystemReport};

impl SystemReport {
    /// Build report from typed evidence (pure function, deterministic)
    pub fn from_evidence(
        evidence: &ReportEvidence,
        trace: Option<&ExecutionTrace>,
        reliability_score: u8,
        reliability_explanation: Option<ReliabilityExplanation>,
    ) -> Self {
        let inventory = build_inventory(evidence);
        let mut health_checks = build_health_checks(evidence);

        // Sort health checks: severity desc, then id asc (stable ordering)
        health_checks.sort_by(|a, b| b.severity.cmp(&a.severity).then_with(|| a.id.cmp(&b.id)));

        let (probe_stats, evidence_kinds, execution_trace_summary) = match trace {
            Some(t) => (
                t.probe_stats.clone(),
                t.evidence_kinds.clone(),
                t.to_string(),
            ),
            None => (
                ProbeStats::default(),
                vec![],
                "no trace available".to_string(),
            ),
        };

        let executive_summary = build_executive_summary(&health_checks, &inventory);

        Self {
            executive_summary,
            inventory,
            health_checks,
            probe_stats,
            evidence_kinds,
            execution_trace_summary,
            reliability_score,
            reliability_explanation,
        }
    }
}

/// Build system inventory from evidence
pub fn build_inventory(evidence: &ReportEvidence) -> SystemInventory {
    let (cpu_model, cpu_cores) = evidence.cpu.as_ref().map_or((None, None), |cpu| {
        (Some(cpu.model_name.clone()), Some(cpu.cpu_count))
    });

    let memory_total_bytes = evidence.memory.as_ref().map(|m| m.total_bytes);

    let disk_count = evidence
        .block_devices
        .iter()
        .filter(|d| d.device_type == crate::parsers::BlockDeviceType::Disk)
        .count();
    let part_count = evidence
        .block_devices
        .iter()
        .filter(|d| d.device_type == crate::parsers::BlockDeviceType::Part)
        .count();

    let block_device_summary = if disk_count == 0 && part_count == 0 {
        "unknown".to_string()
    } else {
        format!(
            "{} disk{}, {} partition{}",
            disk_count,
            if disk_count == 1 { "" } else { "s" },
            part_count,
            if part_count == 1 { "" } else { "s" }
        )
    };

    SystemInventory {
        cpu_model,
        cpu_cores,
        memory_total_bytes,
        block_device_count: evidence.block_devices.len(),
        block_device_summary,
    }
}

/// Build health checks from evidence using pinned thresholds
pub fn build_health_checks(evidence: &ReportEvidence) -> Vec<HealthItem> {
    let mut checks = Vec::new();

    // Disk health checks
    for disk in &evidence.disks {
        let id = format!("disk_{}", sanitize_mount(&disk.mount));
        let (severity, title) = if disk.percent_used >= DISK_CRITICAL_THRESHOLD {
            (
                HealthSeverity::Critical,
                format!("{} disk usage critical", disk.mount),
            )
        } else if disk.percent_used >= DISK_WARNING_THRESHOLD {
            (
                HealthSeverity::Warning,
                format!("{} disk usage elevated", disk.mount),
            )
        } else {
            (
                HealthSeverity::Ok,
                format!("{} disk usage normal", disk.mount),
            )
        };

        let claim = format!(
            "{}% used ({} / {})",
            disk.percent_used,
            format_bytes(disk.used_bytes),
            format_bytes(disk.size_bytes)
        );

        checks.push(HealthItem {
            id,
            severity,
            title,
            claim,
        });
    }

    // Memory health check
    if let Some(mem) = &evidence.memory {
        let usage_ratio = mem.used_bytes as f32 / mem.total_bytes as f32;
        let percent = (usage_ratio * 100.0).round() as u8;

        let (severity, title) = if usage_ratio >= MEMORY_HIGH_THRESHOLD {
            (HealthSeverity::Warning, "Memory usage high".to_string())
        } else {
            (HealthSeverity::Ok, "Memory usage normal".to_string())
        };

        let claim = format!(
            "{}% used ({} / {})",
            percent,
            format_bytes(mem.used_bytes),
            format_bytes(mem.total_bytes)
        );

        checks.push(HealthItem {
            id: "memory".to_string(),
            severity,
            title,
            claim,
        });
    }

    // Failed services health check
    let failed: Vec<_> = evidence
        .failed_services
        .iter()
        .filter(|s| s.state == ServiceState::Failed)
        .collect();

    if !failed.is_empty() {
        let names: Vec<_> = failed.iter().map(|s| s.name.as_str()).collect();
        checks.push(HealthItem {
            id: "services".to_string(),
            severity: HealthSeverity::Warning,
            title: "Failed services detected".to_string(),
            claim: format!("{} failed: {}", failed.len(), names.join(", ")),
        });
    } else if !evidence.failed_services.is_empty() {
        // We have service data but no failures
        checks.push(HealthItem {
            id: "services".to_string(),
            severity: HealthSeverity::Ok,
            title: "All services healthy".to_string(),
            claim: "no failed services".to_string(),
        });
    }

    checks
}

/// Build executive summary from health checks
pub fn build_executive_summary(checks: &[HealthItem], inventory: &SystemInventory) -> Vec<String> {
    let mut summary = Vec::new();

    let critical_count = checks
        .iter()
        .filter(|c| c.severity == HealthSeverity::Critical)
        .count();
    let warning_count = checks
        .iter()
        .filter(|c| c.severity == HealthSeverity::Warning)
        .count();

    if critical_count == 0 && warning_count == 0 {
        summary.push("System healthy, no critical issues detected".to_string());
    } else {
        let mut issues = Vec::new();
        if critical_count > 0 {
            issues.push(format!("{} critical", critical_count));
        }
        if warning_count > 0 {
            issues.push(format!(
                "{} warning{}",
                warning_count,
                if warning_count == 1 { "" } else { "s" }
            ));
        }
        summary.push(format!(
            "{} issue{} detected: {}",
            critical_count + warning_count,
            if critical_count + warning_count == 1 {
                ""
            } else {
                "s"
            },
            issues.join(", ")
        ));
    }

    // Add hardware summary
    if let Some(cores) = inventory.cpu_cores {
        if let Some(mem) = inventory.memory_total_bytes {
            summary.push(format!(
                "Hardware: {} cores, {} RAM",
                cores,
                format_bytes(mem)
            ));
        }
    }

    summary
}
