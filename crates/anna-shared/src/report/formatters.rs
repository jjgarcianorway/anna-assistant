//! Report formatting functions (v0.0.189).

use super::helpers::format_bytes;
use super::types::SystemReport;

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
    let evidence_str: Vec<_> = report
        .evidence_kinds
        .iter()
        .map(|k| k.to_string())
        .collect();
    out.push_str(&format!(
        "  Evidence: {}\n",
        if evidence_str.is_empty() {
            "none".to_string()
        } else {
            evidence_str.join(", ")
        }
    ));
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
    let evidence_str: Vec<_> = report
        .evidence_kinds
        .iter()
        .map(|k| k.to_string())
        .collect();
    out.push_str(&format!(
        "- **Evidence**: {}\n",
        if evidence_str.is_empty() {
            "none".to_string()
        } else {
            evidence_str.join(", ")
        }
    ));
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
