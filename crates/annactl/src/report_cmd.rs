//! Report command handler.
//!
//! Generates a deterministic system health report from probe data.
//! Runs probes directly (not through daemon) for independence.

use anyhow::{anyhow, Result};
use anna_shared::advice::{format_recommendations_markdown, format_recommendations_text, generate_recommendations};
use anna_shared::parsers::{parse_probe_output, ParsedProbeData};
use anna_shared::report::{format_markdown, format_text, ReportEvidence, SystemReport};
use anna_shared::trace::{EvidenceKind, ExecutionTrace, ProbeStats};
use std::process::Command;

/// Probes to run for report generation
const REPORT_PROBES: &[(&str, &str)] = &[
    ("free -b", "memory"),
    ("df -B1", "disk"),
    ("lsblk -b -o NAME,SIZE,TYPE,MOUNTPOINTS", "block"),
    ("lscpu", "cpu"),
    ("systemctl --failed --no-pager", "service"),
];

/// Handle the report command
pub async fn handle_report(format: &str) -> Result<()> {
    // Validate format
    if format != "text" && format != "md" {
        return Err(anyhow!("Invalid format '{}'. Use 'text' or 'md'.", format));
    }

    // Run probes to collect evidence
    let evidence = collect_evidence().await?;

    // Build probe stats
    let probe_stats = ProbeStats {
        planned: REPORT_PROBES.len(),
        succeeded: count_evidence(&evidence),
        failed: 0,
        timed_out: 0,
    };

    // Build evidence kinds from what we collected
    let evidence_kinds = build_evidence_kinds(&evidence);

    // Build execution trace
    let trace = ExecutionTrace::deterministic_route(
        "system_health_report",
        probe_stats,
        evidence_kinds,
    );

    // Generate report
    let report = SystemReport::from_evidence(&evidence, Some(&trace), 100, None);

    // Generate recommendations
    let recommendations = generate_recommendations(&report);

    // Format and print
    let output = match format {
        "md" => {
            let mut out = format_markdown(&report);
            out.push('\n');
            out.push_str(&format_recommendations_markdown(&recommendations));
            out
        }
        _ => {
            let mut out = format_text(&report);
            out.push('\n');
            out.push_str(&format_recommendations_text(&recommendations));
            out
        }
    };

    println!("{}", output);
    Ok(())
}

/// Collect evidence by running probes directly
async fn collect_evidence() -> Result<ReportEvidence> {
    let mut evidence = ReportEvidence::default();

    for (cmd, _probe_type) in REPORT_PROBES {
        match run_probe_direct(cmd) {
            Ok(output) => {
                match parse_probe_output(cmd, &output) {
                    ParsedProbeData::Memory(mem) => evidence.memory = Some(mem),
                    ParsedProbeData::Disk(disks) => evidence.disks = disks,
                    ParsedProbeData::BlockDevices(devices) => evidence.block_devices = devices,
                    ParsedProbeData::Cpu(cpu) => evidence.cpu = Some(cpu),
                    ParsedProbeData::Services(services) => evidence.failed_services = services,
                    _ => {} // Ignore unsupported or error
                }
            }
            Err(e) => {
                eprintln!("Warning: probe '{}' failed: {}", cmd, e);
            }
        }
    }

    Ok(evidence)
}

/// Run a probe command directly via subprocess
fn run_probe_direct(cmd: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| anyhow!("Failed to execute '{}': {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Command '{}' failed with exit code {:?}: {}",
            cmd,
            output.status.code(),
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Count how many evidence types we collected
fn count_evidence(evidence: &ReportEvidence) -> usize {
    let mut count = 0;
    if evidence.memory.is_some() {
        count += 1;
    }
    if !evidence.disks.is_empty() {
        count += 1;
    }
    if !evidence.block_devices.is_empty() {
        count += 1;
    }
    if evidence.cpu.is_some() {
        count += 1;
    }
    // Always count services as collected (even if empty means no failures)
    count += 1;
    count
}

/// Build evidence kinds from collected evidence
fn build_evidence_kinds(evidence: &ReportEvidence) -> Vec<EvidenceKind> {
    let mut kinds = Vec::new();
    if evidence.memory.is_some() {
        kinds.push(EvidenceKind::Memory);
    }
    if !evidence.disks.is_empty() {
        kinds.push(EvidenceKind::Disk);
    }
    if !evidence.block_devices.is_empty() {
        kinds.push(EvidenceKind::BlockDevices);
    }
    if evidence.cpu.is_some() {
        kinds.push(EvidenceKind::Cpu);
    }
    kinds.push(EvidenceKind::Services);
    kinds
}
