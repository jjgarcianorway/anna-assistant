//! Deterministic system report generation.
//!
//! All output derived from typed evidence atoms only.
//! No LLM speculation - purely threshold-driven health checks.

use crate::parsers::{BlockDevice, CpuInfo, DiskUsage, MemoryInfo, ServiceState, ServiceStatus};
use crate::reliability::{
    ReliabilityExplanation, DISK_CRITICAL_THRESHOLD, DISK_WARNING_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};
use crate::trace::{EvidenceKind, ExecutionTrace, ProbeStats};
use serde::{Deserialize, Serialize};

/// Severity level for health items (stable ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    Ok = 0,
    Warning = 1,
    Critical = 2,
}

impl std::fmt::Display for HealthSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A single health check result with supporting evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthItem {
    /// Unique identifier (e.g., "disk_root", "memory", "services")
    pub id: String,
    /// Severity level
    pub severity: HealthSeverity,
    /// Human-readable title (e.g., "Root disk usage critical")
    pub title: String,
    /// Evidence claim (e.g., "95% used (47.5 GB / 50 GB)")
    pub claim: String,
}

/// System inventory (hardware summary)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInventory {
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<u32>,
    pub memory_total_bytes: Option<u64>,
    pub block_device_count: usize,
    /// Summary like "2 disks, 3 partitions"
    pub block_device_summary: String,
}

/// Complete report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReport {
    /// 2-3 bullet points summary
    pub executive_summary: Vec<String>,
    /// Hardware inventory
    pub inventory: SystemInventory,
    /// Health checks sorted by (severity desc, id asc)
    pub health_checks: Vec<HealthItem>,
    /// Probe execution stats
    pub probe_stats: ProbeStats,
    /// Evidence kinds collected
    pub evidence_kinds: Vec<EvidenceKind>,
    /// One-line execution trace summary
    pub execution_trace_summary: String,
    /// Reliability score
    pub reliability_score: u8,
    /// Explanation if score < 80
    pub reliability_explanation: Option<ReliabilityExplanation>,
}

/// Evidence collection for report generation
#[derive(Debug, Clone, Default)]
pub struct ReportEvidence {
    pub memory: Option<MemoryInfo>,
    pub disks: Vec<DiskUsage>,
    pub block_devices: Vec<BlockDevice>,
    pub cpu: Option<CpuInfo>,
    pub failed_services: Vec<ServiceStatus>,
}

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
        health_checks.sort_by(|a, b| {
            b.severity.cmp(&a.severity).then_with(|| a.id.cmp(&b.id))
        });

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
fn build_inventory(evidence: &ReportEvidence) -> SystemInventory {
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
fn build_health_checks(evidence: &ReportEvidence) -> Vec<HealthItem> {
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
fn build_executive_summary(checks: &[HealthItem], inventory: &SystemInventory) -> Vec<String> {
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
            issues.push(format!("{} warning{}", warning_count, if warning_count == 1 { "" } else { "s" }));
        }
        summary.push(format!("{} issue{} detected: {}",
            critical_count + warning_count,
            if critical_count + warning_count == 1 { "" } else { "s" },
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

/// Sanitize mount point for use in ID
fn sanitize_mount(mount: &str) -> String {
    if mount == "/" {
        "root".to_string()
    } else {
        mount.trim_start_matches('/').replace('/', "_")
    }
}

/// Format bytes as human-readable
fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// =============================================================================
// Report formatting
// =============================================================================

/// Format report as plain text (deterministic, no timestamps)
pub fn format_text(report: &SystemReport) -> String {
    let mut out = String::new();

    out.push_str("SYSTEM REPORT\n");
    out.push_str("=============\n\n");

    // Executive summary
    out.push_str("EXECUTIVE SUMMARY\n");
    for item in &report.executive_summary {
        out.push_str(&format!("  * {}\n", item));
    }
    out.push('\n');

    // Inventory
    out.push_str("INVENTORY\n");
    if let Some(model) = &report.inventory.cpu_model {
        out.push_str(&format!(
            "  CPU: {} ({} cores)\n",
            model,
            report.inventory.cpu_cores.unwrap_or(0)
        ));
    }
    if let Some(mem) = report.inventory.memory_total_bytes {
        out.push_str(&format!("  Memory: {}\n", format_bytes(mem)));
    }
    out.push_str(&format!(
        "  Storage: {}\n",
        report.inventory.block_device_summary
    ));
    out.push('\n');

    // Health checks
    out.push_str("HEALTH CHECKS\n");
    for check in &report.health_checks {
        out.push_str(&format!(
            "  [{}] {}\n         {}\n",
            check.severity, check.title, check.claim
        ));
    }
    out.push('\n');

    // Execution
    out.push_str("EXECUTION\n");
    out.push_str(&format!("  Probes: {}\n", report.probe_stats));
    let evidence_str: Vec<_> = report.evidence_kinds.iter().map(|k| k.to_string()).collect();
    out.push_str(&format!("  Evidence: {}\n", if evidence_str.is_empty() { "none".to_string() } else { evidence_str.join(", ") }));
    out.push_str(&format!("  Path: {}\n", report.execution_trace_summary));
    out.push('\n');

    // Reliability
    out.push_str("RELIABILITY\n");
    out.push_str(&format!("  Score: {}%\n", report.reliability_score));
    if let Some(explanation) = &report.reliability_explanation {
        out.push_str(&format!("  {}\n", explanation.summary));
    }

    out
}

/// Format report as markdown (deterministic, no timestamps)
pub fn format_markdown(report: &SystemReport) -> String {
    let mut out = String::new();

    out.push_str("# System Report\n\n");

    // Executive summary
    out.push_str("## Executive Summary\n\n");
    for item in &report.executive_summary {
        out.push_str(&format!("- {}\n", item));
    }
    out.push('\n');

    // Inventory
    out.push_str("## Inventory\n\n");
    out.push_str("| Component | Value |\n");
    out.push_str("|-----------|-------|\n");
    if let Some(model) = &report.inventory.cpu_model {
        out.push_str(&format!(
            "| CPU | {} ({} cores) |\n",
            model,
            report.inventory.cpu_cores.unwrap_or(0)
        ));
    }
    if let Some(mem) = report.inventory.memory_total_bytes {
        out.push_str(&format!("| Memory | {} |\n", format_bytes(mem)));
    }
    out.push_str(&format!(
        "| Storage | {} |\n",
        report.inventory.block_device_summary
    ));
    out.push('\n');

    // Health checks
    out.push_str("## Health Checks\n\n");
    out.push_str("| Status | Check | Evidence |\n");
    out.push_str("|--------|-------|----------|\n");
    for check in &report.health_checks {
        out.push_str(&format!(
            "| **{}** | {} | {} |\n",
            check.severity, check.title, check.claim
        ));
    }
    out.push('\n');

    // Execution
    out.push_str("## Execution\n\n");
    out.push_str(&format!("- **Probes**: {}\n", report.probe_stats));
    let evidence_str: Vec<_> = report.evidence_kinds.iter().map(|k| k.to_string()).collect();
    out.push_str(&format!("- **Evidence**: {}\n", if evidence_str.is_empty() { "none".to_string() } else { evidence_str.join(", ") }));
    out.push_str(&format!("- **Path**: {}\n", report.execution_trace_summary));
    out.push('\n');

    // Reliability
    out.push_str("## Reliability\n\n");
    out.push_str(&format!("**Score**: {}%\n\n", report.reliability_score));
    if let Some(explanation) = &report.reliability_explanation {
        out.push_str(&format!("{}\n", explanation.summary));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_bytes(32 * 1024 * 1024 * 1024), "32.0 GB");
    }

    #[test]
    fn test_sanitize_mount() {
        assert_eq!(sanitize_mount("/"), "root");
        assert_eq!(sanitize_mount("/var"), "var");
        assert_eq!(sanitize_mount("/var/log"), "var_log");
    }

    #[test]
    fn test_health_severity_ordering() {
        assert!(HealthSeverity::Critical > HealthSeverity::Warning);
        assert!(HealthSeverity::Warning > HealthSeverity::Ok);
    }
}
