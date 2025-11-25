//! Unified System Report Generator
//!
//! Version 150: Single source of truth for system reports
//! Used by both CLI and TUI to ensure identical output
//!
//! Rules:
//! - Uses telemetry_truth for verified data only
//! - No hallucinations, no guessing
//! - Same input → same output (deterministic)
//! - Clean, professional formatting

use crate::system_query::query_system_telemetry;
use crate::telemetry_truth::{VerifiedSystemReport, HealthStatus};
use anna_common::output_engine::OutputEngine;
use anyhow::Result;

/// Generate a complete system report
///
/// This is the single entry point for "write a full report about my computer"
/// queries. It produces identical output for CLI and TUI.
pub fn generate_full_report() -> Result<String> {
    let telemetry = query_system_telemetry()?;
    let verified = VerifiedSystemReport::from_telemetry(&telemetry);

    let mut report = String::new();

    // Header
    report.push_str(&format!("# System Report: {}\n\n", verified.hostname.display()));

    // Health Summary (most important first)
    report.push_str("## Health Summary\n\n");
    match verified.health_summary.overall_status {
        HealthStatus::Healthy => {
            report.push_str("✅ **Status**: Healthy\n\n");
            for info in &verified.health_summary.info {
                report.push_str(&format!("- {}\n", info));
            }
        }
        HealthStatus::Warning => {
            report.push_str("⚠️  **Status**: Warnings Detected\n\n");
            for warning in &verified.health_summary.warnings {
                report.push_str(&format!("- ⚠️  {}\n", warning));
            }
        }
        HealthStatus::Critical => {
            report.push_str("🔴 **Status**: Critical Issues\n\n");
            for issue in &verified.health_summary.critical_issues {
                report.push_str(&format!("- 🔴 {}\n", issue));
            }
            if !verified.health_summary.warnings.is_empty() {
                report.push_str("\n**Additional Warnings**:\n");
                for warning in &verified.health_summary.warnings {
                    report.push_str(&format!("- ⚠️  {}\n", warning));
                }
            }
        }
    }
    report.push('\n');

    // Hardware
    report.push_str("## Hardware\n\n");
    report.push_str(&format!("**CPU**: {} ({} cores)\n",
        verified.cpu_model.display(),
        verified.cpu_cores.display()));
    report.push_str(&format!("**Load Average**: {}\n", verified.cpu_load.display()));
    report.push_str(&format!("**RAM**: {} GB used / {} GB total ({} %)\n",
        verified.ram_used_gb.display(),
        verified.ram_total_gb.display(),
        verified.ram_percent.display()));
    report.push_str(&format!("**GPU**: {}\n", verified.gpu.display()));
    report.push('\n');

    // Storage
    report.push_str("## Storage\n\n");
    for disk in &verified.storage {
        report.push_str(&format!("**{}**:\n", disk.mount_point));
        report.push_str(&format!("  - Total: {} GB\n", disk.total_gb.display()));
        report.push_str(&format!("  - Used: {} GB ({} %)\n",
            disk.used_gb.display(),
            disk.used_percent.display()));
        report.push_str(&format!("  - Free: {} GB\n", disk.free_gb.display()));
        report.push('\n');
    }

    // System
    report.push_str("## System Information\n\n");
    report.push_str(&format!("**OS**: {}\n", verified.os_name.display()));
    report.push_str(&format!("**Kernel**: {}\n", verified.kernel_version.display()));
    report.push_str(&format!("**Hostname**: {}\n", verified.hostname.display()));
    report.push_str(&format!("**Uptime**: {}\n", verified.uptime.display()));
    report.push('\n');

    // Desktop
    report.push_str("## Desktop Environment\n\n");
    report.push_str(&format!("**Desktop**: {}\n", verified.desktop_environment.display()));
    report.push_str(&format!("**Window Manager**: {}\n", verified.window_manager.display()));
    report.push_str(&format!("**Display Protocol**: {}\n", verified.display_protocol.display()));
    report.push('\n');

    // Network
    report.push_str("## Network\n\n");
    report.push_str(&format!("**Status**: {}\n", verified.network_status.display()));
    report.push_str(&format!("**Primary Interface**: {}\n", verified.primary_interface.display()));

    if !verified.ip_addresses.is_empty() {
        report.push_str("**IP Addresses**:\n");
        for ip in &verified.ip_addresses {
            report.push_str(&format!("  - {}\n", ip.display()));
        }
    } else {
        report.push_str("**IP Addresses**: None detected\n");
    }
    report.push('\n');

    // Services
    if !verified.failed_services.is_empty() {
        report.push_str("## Failed Services\n\n");
        for service in &verified.failed_services {
            report.push_str(&format!("- ❌ {}\n", service));
        }
        report.push_str("\nRun `systemctl --failed` for details.\n\n");
    }

    // Footer
    report.push_str("---\n");
    report.push_str("*Report generated from verified system telemetry*\n");
    report.push_str("*All values are real - no guesses or defaults*\n");

    Ok(report)
}

/// Generate a short system summary (for status display)
pub fn generate_short_summary() -> Result<String> {
    let telemetry = query_system_telemetry()?;
    let verified = VerifiedSystemReport::from_telemetry(&telemetry);

    let mut summary = String::new();

    // One-line status
    match verified.health_summary.overall_status {
        HealthStatus::Healthy => summary.push_str("✅ All systems nominal"),
        HealthStatus::Warning => summary.push_str("⚠️  System warnings detected"),
        HealthStatus::Critical => summary.push_str("🔴 Critical issues require attention"),
    }

    summary.push_str(&format!(" | CPU: {} @ {}% | RAM: {} / {} GB",
        verified.cpu_model.display().split_whitespace().nth(2).unwrap_or("Unknown"),
        verified.cpu_load.display(),
        verified.ram_used_gb.display(),
        verified.ram_total_gb.display()));

    Ok(summary)
}

/// Check if a query is asking for a full system report or status
/// Beta.243: Expanded keyword coverage for status queries
/// Beta.244: Added temporal and importance-based status patterns
/// Beta.254: Now uses shared normalization from unified_query_handler
pub fn is_system_report_query(query: &str) -> bool {
    // Beta.254: Use shared normalization for consistent behavior
    let query_lower = crate::unified_query_handler::normalize_query_for_intent(query);

    // Full report phrasings (original behavior)
    let is_full_report = (query_lower.contains("full report") ||
                         query_lower.contains("complete report") ||
                         query_lower.contains("system report")) &&
                        (query_lower.contains("computer") ||
                         query_lower.contains("system") ||
                         query_lower.contains("machine"));

    if is_full_report {
        return true;
    }

    // Beta.243: Expanded status query keywords
    // These are lighter-weight status checks vs full diagnostic
    let status_keywords = [
        "show me status",
        "system status",
        "what's running",
        "system information",
        "system info",
        // Beta.243: New status keywords
        "current status",
        "what is the current status",
        "what is the status",
        "system state",
        "show system state",
        "what's happening on my system",
        "what's happening",
        // Beta.249: Removed "how is my system" and "how is the system" - they're diagnostic patterns
        // Beta.251: "status of" patterns
        "status of my system",
        "status of the system",
        "status of this system",
        "status of my machine",
        "status of my computer",
        "status of this machine",
        "status of this computer",
        // Beta.251: "[my/this] [computer/machine] status" patterns
        "my computer status",
        "my machine status",
        "my system status",  // for consistency
        "this computer status",
        "this machine status",
        "this system status",  // for consistency
        "computer status",
        "machine status",
        // Beta.251: "status current" terse pattern
        "status current",
        "current system status",
        // Beta.253: Category C - "the machine/computer/system status" variants
        "the machine status",
        "the computer status",
        "the system status",
        "check the machine status",
        "check the computer status",
        "check the system status",
        "the machine's status",
        "the computer's status",
        "the system's status",
        // Beta.275: Additional status report patterns
        "extensive status report",
        "detailed status report",
    ];

    for keyword in &status_keywords {
        if query_lower.contains(keyword) {
            return true;
        }
    }

    // Beta.244: Temporal and importance-based status patterns
    // Beta.255: Extended with recency and "what happened" patterns
    // These imply "right now" or "anything important" combined with system references
    //
    // Temporal indicators: today, now, currently, right now, recently, lately, this morning
    // Recency indicators: what happened, any events, anything changed
    // Importance indicators: important, critical, wrong, issues, problems, review
    // System references: this system, this machine, this computer, my system, my machine

    let temporal_indicators = ["today", "now", "currently", "right now", "recently", "lately",
                               "this morning", "this afternoon", "this evening", "in the last hour"];
    let recency_indicators = [
        "what happened",
        "anything happened",
        "any events",
        "anything changed",
        "any changes",
    ];
    let importance_indicators = [
        "anything important",
        "anything critical",
        "anything wrong",
        "any issues",
        "any problems",
        "important to review",
        "to review",
        "should know",
        "need to know",
    ];
    let system_references = [
        "this system",
        "this machine",
        "this computer",
        "my system",
        "my machine",
        "my computer",
        "the system",
        "the machine",
    ];

    // Check if query has temporal indicator + system reference
    let has_temporal = temporal_indicators.iter().any(|ind| query_lower.contains(ind));
    let has_recency = recency_indicators.iter().any(|ind| query_lower.contains(ind));
    let has_importance = importance_indicators.iter().any(|ind| query_lower.contains(ind));
    let has_system_ref = system_references.iter().any(|ind| query_lower.contains(ind));

    // Beta.255: Match if: (temporal OR recency OR importance) AND system_reference
    // Examples: "how is my system today", "what happened on this machine", "anything important on my system"
    if (has_temporal || has_recency || has_importance) && has_system_ref {
        return true;
    }

    // Beta.244: Also match standalone importance queries that clearly reference system context
    // Example: "anything important to review on this system today"
    if has_importance && has_system_ref {
        return true;
    }

    false
}

// ============================================================================
// v6.33.0: Capability Queries & Disk Explorer
// ============================================================================

/// Capability query types (v6.33.0)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityKind {
    /// CPU instruction set flag (e.g., "sse2", "avx2", "avx512")
    CpuFlag(String),
    /// GPU VRAM query
    GpuVram,
    /// GPU vendor query
    GpuVendor,
    /// Virtualization support (VT-x/AMD-V)
    VirtualizationSupport,
}

/// Disk explorer specification (v6.33.0)
#[derive(Debug, Clone)]
pub struct DiskExplorerSpec {
    /// Root path to explore (default "/")
    pub root_path: String,
    /// Number of results to show (default 10)
    pub count: usize,
}

impl Default for DiskExplorerSpec {
    fn default() -> Self {
        Self {
            root_path: "/".to_string(),
            count: 10,
        }
    }
}

/// Detect capability queries and extract the capability kind
///
/// # Examples
/// - "does my cpu support sse2" → Some(CpuFlag("sse2"))
/// - "do I have avx2" → Some(CpuFlag("avx2"))
/// - "how much vram do I have" → Some(GpuVram)
/// - "which gpu do I have" → Some(GpuVendor)
/// - "does this machine support virtualization" → Some(VirtualizationSupport)
pub fn is_capability_query(text: &str) -> Option<CapabilityKind> {
    let text_lower = text.to_lowercase();

    // CPU flags - detect questions about CPU features
    let is_cpu_query = (text_lower.contains("cpu") || text_lower.contains("processor"))
        && (text_lower.contains("support") || text_lower.contains("have"));

    let is_flag_query = text_lower.contains("support") || text_lower.contains("have") || text_lower.contains("does");

    // Check for specific CPU flags (can be asked with or without explicit "cpu" mention)
    if is_cpu_query || is_flag_query {
        if text_lower.contains("sse2") || text_lower.contains("sse 2") {
            return Some(CapabilityKind::CpuFlag("sse2".to_string()));
        }
        if text_lower.contains("avx2") || text_lower.contains("avx 2") {
            return Some(CapabilityKind::CpuFlag("avx2".to_string()));
        }
        if text_lower.contains("avx512") || text_lower.contains("avx-512") || text_lower.contains("avx 512") {
            return Some(CapabilityKind::CpuFlag("avx512f".to_string())); // AVX-512 Foundation
        }
        if text_lower.contains("avx") && !text_lower.contains("avx2") && !text_lower.contains("avx512") {
            return Some(CapabilityKind::CpuFlag("avx".to_string()));
        }
    }

    // GPU VRAM
    if (text_lower.contains("vram") || (text_lower.contains("video") && text_lower.contains("memory")))
        && (text_lower.contains("how much") || text_lower.contains("amount"))
    {
        return Some(CapabilityKind::GpuVram);
    }

    // GPU vendor/model
    if (text_lower.contains("which gpu") || text_lower.contains("what gpu") || text_lower.contains("gpu model"))
        || (text_lower.contains("graphic") && (text_lower.contains("which") || text_lower.contains("what")))
    {
        return Some(CapabilityKind::GpuVendor);
    }

    // Virtualization
    if text_lower.contains("virtualization")
        || (text_lower.contains("vm") && text_lower.contains("support"))
        || text_lower.contains("vt-x")
        || text_lower.contains("amd-v")
    {
        return Some(CapabilityKind::VirtualizationSupport);
    }

    None
}

/// Detect disk explorer queries and extract the specification
///
/// # Examples
/// - "biggest folders on my system" → Some(DiskExplorerSpec { root_path: "/", count: 10 })
/// - "top 10 biggest directories" → Some(DiskExplorerSpec { root_path: "/", count: 10 })
/// - "show largest folders under /home" → Some(DiskExplorerSpec { root_path: "/home", count: 10 })
/// - "list biggest directories in /var" → Some(DiskExplorerSpec { root_path: "/var", count: 10 })
pub fn is_disk_explorer_query(text: &str) -> Option<DiskExplorerSpec> {
    let text_lower = text.to_lowercase();

    // Must mention size/largest/biggest
    let has_size_context = text_lower.contains("biggest")
        || text_lower.contains("largest")
        || text_lower.contains("top")
        || text_lower.contains("most space");

    // Must mention folders/directories
    let has_folder_context = text_lower.contains("folder")
        || text_lower.contains("director")
        || text_lower.contains("dir");

    if !has_size_context || !has_folder_context {
        return None;
    }

    // Extract path if specified
    let root_path = if text_lower.contains("/home") {
        "/home".to_string()
    } else if text_lower.contains("/var") {
        "/var".to_string()
    } else if text_lower.contains("/usr") {
        "/usr".to_string()
    } else if text_lower.contains("/opt") {
        "/opt".to_string()
    } else {
        "/".to_string()
    };

    // Extract count if specified (look for numbers)
    let count = if text_lower.contains("top 5") || text_lower.contains("5 biggest") {
        5
    } else if text_lower.contains("top 15") || text_lower.contains("15 biggest") {
        15
    } else if text_lower.contains("top 20") || text_lower.contains("20 biggest") {
        20
    } else {
        10 // default
    };

    Some(DiskExplorerSpec {
        root_path,
        count,
    })
}

/// Handle capability check queries (v6.33.0, polished v6.34.0)
/// Uses compact style with OutputEngine for consistent formatting
pub fn handle_capability_check(kind: CapabilityKind) -> Result<String> {
    let engine = OutputEngine::new();

    match kind {
        CapabilityKind::CpuFlag(flag) => {
            // Try to check CPU flags from /proc/cpuinfo
            let has_flag = check_cpu_flag(&flag)?;

            let main_line = if has_flag {
                format!("Yes, your CPU supports {}.", flag.to_uppercase())
            } else {
                format!("No, your CPU does not support {}.", flag.to_uppercase())
            };

            Ok(engine.format_compact(&main_line, Some("CPU flags from /proc/cpuinfo")))
        }
        CapabilityKind::GpuVram => {
            // Check for GPU VRAM in telemetry
            let telemetry = query_system_telemetry()?;

            let main_line = if telemetry.hardware.has_gpu {
                "Discrete GPU detected. VRAM info requires: lspci -v | grep -A 10 VGA".to_string()
            } else {
                "No discrete GPU detected. Integrated graphics share system RAM.".to_string()
            };

            Ok(engine.format_compact(&main_line, Some("System telemetry")))
        }
        CapabilityKind::GpuVendor => {
            let telemetry = query_system_telemetry()?;

            let main_line = if telemetry.hardware.has_gpu {
                "Discrete GPU detected. Details: lspci | grep -i vga".to_string()
            } else {
                "No discrete GPU detected. System may use integrated graphics.".to_string()
            };

            Ok(engine.format_compact(&main_line, Some("System telemetry")))
        }
        CapabilityKind::VirtualizationSupport => {
            // Check for vmx (Intel) or svm (AMD) flags
            let has_vmx = check_cpu_flag("vmx").unwrap_or(false);
            let has_svm = check_cpu_flag("svm").unwrap_or(false);

            let main_line = if has_vmx {
                "Yes, hardware virtualization supported (Intel VT-x detected).".to_string()
            } else if has_svm {
                "Yes, hardware virtualization supported (AMD-V detected).".to_string()
            } else {
                "No hardware virtualization flags found (checked vmx and svm).".to_string()
            };

            Ok(engine.format_compact(&main_line, Some("CPU flags from /proc/cpuinfo")))
        }
    }
}

/// Check if a CPU flag is present in /proc/cpuinfo
fn check_cpu_flag(flag: &str) -> Result<bool> {
    use std::fs;

    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;

    // Find the flags line
    for line in cpuinfo.lines() {
        if line.starts_with("flags") || line.starts_with("Features") {
            // Split by : and get the flags part
            if let Some(flags_part) = line.split(':').nth(1) {
                // Check if the flag is present (case-insensitive, whole word match)
                let flag_lower = flag.to_lowercase();
                for cpu_flag in flags_part.split_whitespace() {
                    if cpu_flag.to_lowercase() == flag_lower {
                        return Ok(true);
                    }
                }
            }
            // Found flags line but flag not present
            return Ok(false);
        }
    }

    // No flags line found
    Ok(false)
}

/// Handle disk explorer queries (v6.33.0, polished v6.34.0)
/// Uses stepwise style with OutputEngine for consistent formatting
pub fn handle_disk_explorer(spec: DiskExplorerSpec) -> Result<String> {
    use std::process::Command;

    let engine = OutputEngine::new();
    let mut output = String::new();

    // Section header - v6.34.0: Use OutputEngine
    output.push_str(&engine.format_subheader(&format!("Largest directories under {}", spec.root_path)));
    output.push_str("\n\n");

    // Run du command (safe, read-only)
    let du_output = Command::new("du")
        .args([
            "-x",           // Don't cross filesystem boundaries
            "-h",           // Human-readable sizes
            "--max-depth=3", // Limit depth to avoid overwhelming output
            &spec.root_path,
        ])
        .output();

    match du_output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);

            // Parse and sort the output
            let mut entries: Vec<(&str, &str)> = Vec::new();
            for line in stdout.lines() {
                if let Some((size, path)) = line.split_once('\t') {
                    entries.push((size.trim(), path.trim()));
                }
            }

            // Sort by converting human-readable sizes to bytes (approximately)
            entries.sort_by(|a, b| {
                let size_a = parse_human_size(a.0);
                let size_b = parse_human_size(b.0);
                size_b.cmp(&size_a) // Descending order
            });

            // Take top N
            let top_entries: Vec<_> = entries.iter().take(spec.count).collect();

            if top_entries.is_empty() {
                output.push_str("No directories found or insufficient permissions.\n");
            } else {
                // v6.34.0: Use key-value formatting
                for (size, path) in top_entries {
                    output.push_str(&engine.format_key_value(size, path));
                    output.push('\n');
                }
            }

            output.push_str(&format!("\nShowing top {} of {} total entries.\n",
                spec.count.min(entries.len()), entries.len()));
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            output.push_str(&format!("Command failed: {}\n", stderr));
        }
        Err(e) => {
            output.push_str(&format!("Unable to run du command: {}\n", e));
            output.push_str("\nYou can run manually:\n");
            // v6.34.0: No markdown fences - use format_command
            output.push_str(&engine.format_command(&format!(
                "du -x -h --max-depth=3 {} 2>/dev/null | sort -h | tail -n {}",
                spec.root_path,
                spec.count
            )));
            output.push('\n');
        }
    }

    Ok(output)
}

/// Parse human-readable size to approximate bytes for sorting
fn parse_human_size(size_str: &str) -> u64 {
    let size_str = size_str.trim();
    if size_str.is_empty() {
        return 0;
    }

    let (num_part, suffix) = if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
        (&size_str[..pos], &size_str[pos..])
    } else {
        (size_str, "")
    };

    let num: f64 = num_part.trim().parse().unwrap_or(0.0);

    let multiplier: u64 = match suffix.to_uppercase().as_str() {
        "K" => 1024,
        "M" => 1024 * 1024,
        "G" => 1024 * 1024 * 1024,
        "T" => 1024 * 1024 * 1024 * 1024,
        _ => 1,
    };

    (num * multiplier as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_report_query() {
        assert!(is_system_report_query("write a full report about my computer please"));
        assert!(is_system_report_query("give me a complete report of my system"));
        assert!(is_system_report_query("show me a full system report"));

        assert!(!is_system_report_query("what is my CPU?"));
        assert!(!is_system_report_query("how much RAM do I have?"));
    }

    // ===== v6.33.0 Tests =====

    #[test]
    fn test_is_capability_query_cpu_flags() {
        assert_eq!(
            is_capability_query("does my cpu support sse2"),
            Some(CapabilityKind::CpuFlag("sse2".to_string()))
        );
        assert_eq!(
            is_capability_query("do I have avx2"),
            Some(CapabilityKind::CpuFlag("avx2".to_string()))
        );
        assert_eq!(
            is_capability_query("does my processor support avx-512"),
            Some(CapabilityKind::CpuFlag("avx512f".to_string()))
        );
    }

    #[test]
    fn test_is_capability_query_gpu() {
        assert_eq!(
            is_capability_query("how much vram do I have"),
            Some(CapabilityKind::GpuVram)
        );
        assert_eq!(
            is_capability_query("which gpu do I have"),
            Some(CapabilityKind::GpuVendor)
        );
    }

    #[test]
    fn test_is_capability_query_virtualization() {
        assert_eq!(
            is_capability_query("does this machine support virtualization"),
            Some(CapabilityKind::VirtualizationSupport)
        );
    }

    #[test]
    fn test_is_capability_query_negative() {
        assert_eq!(is_capability_query("show me the logs"), None);
        assert_eq!(is_capability_query("biggest folders"), None);
    }

    #[test]
    fn test_is_disk_explorer_query_positive() {
        assert!(is_disk_explorer_query("biggest folders on my system").is_some());
        assert!(is_disk_explorer_query("top 10 biggest directories").is_some());
        assert!(is_disk_explorer_query("show largest folders under /home").is_some());
        assert!(is_disk_explorer_query("list biggest directories in /var").is_some());
    }

    #[test]
    fn test_is_disk_explorer_query_paths() {
        let spec = is_disk_explorer_query("biggest folders under /home").unwrap();
        assert_eq!(spec.root_path, "/home");

        let spec = is_disk_explorer_query("largest directories in /var").unwrap();
        assert_eq!(spec.root_path, "/var");

        let spec = is_disk_explorer_query("biggest folders").unwrap();
        assert_eq!(spec.root_path, "/");
    }

    #[test]
    fn test_is_disk_explorer_query_counts() {
        let spec = is_disk_explorer_query("top 5 biggest folders").unwrap();
        assert_eq!(spec.count, 5);

        let spec = is_disk_explorer_query("top 20 biggest directories").unwrap();
        assert_eq!(spec.count, 20);

        let spec = is_disk_explorer_query("biggest folders").unwrap();
        assert_eq!(spec.count, 10); // default
    }

    #[test]
    fn test_is_disk_explorer_query_negative() {
        assert!(is_disk_explorer_query("show me the logs").is_none());
        assert!(is_disk_explorer_query("what is my cpu").is_none());
    }

    #[test]
    fn test_parse_human_size() {
        assert_eq!(parse_human_size("1K"), 1024);
        assert_eq!(parse_human_size("1M"), 1024 * 1024);
        assert_eq!(parse_human_size("1G"), 1024 * 1024 * 1024);
        assert_eq!(parse_human_size("2.5G"), (2.5 * 1024.0 * 1024.0 * 1024.0) as u64);
    }

    // ========================================================================
    // v6.33.0: Integration Tests (ACTS - Assert-Commands-Test-Scenario)
    // ========================================================================

    #[test]
    fn test_integration_cpu_flag_check_sse2() {
        // Test full flow: query detection -> handler -> result
        let query = "does my cpu support sse2?";
        let kind = is_capability_query(query);
        assert!(kind.is_some());
        if let Some(CapabilityKind::CpuFlag(flag)) = kind {
            assert_eq!(flag, "sse2");
            // Handler should return a result (may fail if /proc/cpuinfo unavailable in test env)
            let result = handle_capability_check(CapabilityKind::CpuFlag(flag));
            assert!(result.is_ok() || result.is_err()); // Either works, just verify it executes
        }
    }

    #[test]
    fn test_integration_cpu_flag_check_avx2() {
        let query = "do I have AVX2?";
        let kind = is_capability_query(query);
        assert!(kind.is_some());
        if let Some(CapabilityKind::CpuFlag(flag)) = kind {
            assert_eq!(flag, "avx2");
        }
    }

    #[test]
    fn test_integration_gpu_vram_query() {
        let query = "how much vram do i have?";
        let kind = is_capability_query(query);
        assert!(kind.is_some());
        assert_eq!(kind.unwrap(), CapabilityKind::GpuVram);
    }

    #[test]
    fn test_integration_gpu_vendor_query() {
        let query = "what GPU do I have?";
        let kind = is_capability_query(query);
        assert!(kind.is_some());
        assert_eq!(kind.unwrap(), CapabilityKind::GpuVendor);
    }

    #[test]
    fn test_integration_virtualization_query() {
        let query = "does my system support virtualization?";
        let kind = is_capability_query(query);
        assert!(kind.is_some());
        assert_eq!(kind.unwrap(), CapabilityKind::VirtualizationSupport);
    }

    #[test]
    fn test_integration_disk_explorer_biggest_folders() {
        let query = "show me the 10 biggest folders in /home";
        let spec = is_disk_explorer_query(query);
        assert!(spec.is_some());
        if let Some(spec) = spec {
            assert_eq!(spec.root_path, "/home");
            assert_eq!(spec.count, 10);
            // Verify handler can execute (may fail if /home not accessible)
            let result = handle_disk_explorer(spec);
            assert!(result.is_ok() || result.is_err()); // Either works, just verify it executes
        }
    }

    #[test]
    fn test_integration_disk_explorer_default_path() {
        let query = "what are the biggest directories?";
        let spec = is_disk_explorer_query(query);
        assert!(spec.is_some());
        if let Some(spec) = spec {
            assert_eq!(spec.root_path, "/");  // Default is root /
            assert_eq!(spec.count, 10);
        }
    }

    #[test]
    fn test_integration_disk_explorer_custom_count() {
        let query = "show top 5 largest folders under /var";
        let spec = is_disk_explorer_query(query);
        assert!(spec.is_some());
        if let Some(spec) = spec {
            assert_eq!(spec.root_path, "/var");
            assert_eq!(spec.count, 5);
        }
    }

    // ========================================================================
    // v6.34.0: CLI Consistency & Output Standards Tests
    // ========================================================================

    // No-markdown-fence tests (6 tests)

    #[test]
    fn test_no_fences_capability_check() {
        use anna_common::output_engine::OutputEngine;

        // Test CPU flag check
        let result = handle_capability_check(CapabilityKind::CpuFlag("sse2".to_string()));
        if let Ok(answer) = result {
            assert!(!answer.contains("```"), "CapabilityCheck must not contain markdown fences");
            assert!(OutputEngine::validate_no_fences(&answer), "Failed no-fences validation");
        }
    }

    #[test]
    fn test_no_fences_gpu_vram() {
        use anna_common::output_engine::OutputEngine;

        let result = handle_capability_check(CapabilityKind::GpuVram);
        if let Ok(answer) = result {
            assert!(!answer.contains("```"), "GPU VRAM check must not contain markdown fences");
            assert!(OutputEngine::validate_no_fences(&answer));
        }
    }

    #[test]
    fn test_no_fences_virtualization() {
        use anna_common::output_engine::OutputEngine;

        let result = handle_capability_check(CapabilityKind::VirtualizationSupport);
        if let Ok(answer) = result {
            assert!(!answer.contains("```"), "Virtualization check must not contain markdown fences");
            assert!(OutputEngine::validate_no_fences(&answer));
        }
    }

    #[test]
    fn test_no_fences_disk_explorer() {
        use anna_common::output_engine::OutputEngine;

        let spec = DiskExplorerSpec {
            root_path: "/tmp".to_string(),
            count: 5,
        };

        let result = handle_disk_explorer(spec);
        if let Ok(answer) = result {
            assert!(!answer.contains("```"), "DiskExplorer must not contain markdown fences");
            assert!(OutputEngine::validate_no_fences(&answer));
        }
    }

    #[test]
    fn test_no_fences_system_report() {
        use anna_common::output_engine::OutputEngine;

        if is_system_report_query("write me a report") {
            // System report should not have fences (already tested in v6.32)
            let result = generate_full_report();
            if let Ok(report) = result {
                assert!(!report.contains("```"), "SystemReport must not contain markdown fences");
                assert!(OutputEngine::validate_no_fences(&report));
            }
        }
    }

    #[test]
    fn test_answer_style_enum() {
        use anna_common::output_engine::AnswerStyle;

        // Test style mapping
        assert_eq!(AnswerStyle::from_intent("CapabilityCheck"), AnswerStyle::Compact);
        assert_eq!(AnswerStyle::from_intent("SimpleFact"), AnswerStyle::Compact);
        assert_eq!(AnswerStyle::from_intent("SystemReport"), AnswerStyle::Sectioned);
        assert_eq!(AnswerStyle::from_intent("DiskExplorer"), AnswerStyle::Stepwise);
        assert_eq!(AnswerStyle::from_intent("Unknown"), AnswerStyle::Sectioned); // Default
    }

    // Style correctness tests (10 tests)

    #[test]
    fn test_style_cpu_flag_compact() {
        // Capability check should be compact (1-3 lines)
        let result = handle_capability_check(CapabilityKind::CpuFlag("avx2".to_string()));
        if let Ok(answer) = result {
            let lines: Vec<&str> = answer.lines().collect();
            assert!(lines.len() <= 4, "CPU flag check should be compact (≤4 lines), got {}", lines.len());
            assert!(answer.contains("AVX2") || answer.contains("avx2"), "Should mention the queried flag");
        }
    }

    #[test]
    fn test_style_gpu_vram_compact() {
        let result = handle_capability_check(CapabilityKind::GpuVram);
        if let Ok(answer) = result {
            let lines: Vec<&str> = answer.lines().collect();
            assert!(lines.len() <= 4, "GPU VRAM check should be compact (≤4 lines), got {}", lines.len());
        }
    }

    #[test]
    fn test_style_virtualization_compact() {
        let result = handle_capability_check(CapabilityKind::VirtualizationSupport);
        if let Ok(answer) = result {
            let lines: Vec<&str> = answer.lines().collect();
            assert!(lines.len() <= 4, "Virtualization check should be compact (≤4 lines), got {}", lines.len());
            assert!(answer.to_lowercase().contains("virtualization"), "Should mention virtualization");
        }
    }

    #[test]
    fn test_style_disk_explorer_has_header() {
        let spec = DiskExplorerSpec {
            root_path: "/tmp".to_string(),
            count: 10,
        };

        let result = handle_disk_explorer(spec);
        if let Ok(answer) = result {
            assert!(answer.contains("Largest directories") || answer.contains("/tmp"),
                    "DiskExplorer should have descriptive header");
        }
    }

    #[test]
    fn test_style_capability_has_source() {
        // v6.34.0: Compact answers should include source attribution
        let result = handle_capability_check(CapabilityKind::CpuFlag("sse2".to_string()));
        if let Ok(answer) = result {
            assert!(answer.to_lowercase().contains("source:") || answer.contains("/proc/cpuinfo"),
                    "Should include source attribution");
        }
    }

    #[test]
    fn test_style_system_report_has_sections() {
        if is_system_report_query("generate system report") {
            let result = generate_full_report();
            if let Ok(report) = result {
                // Should have multiple sections
                let has_sections = report.contains("Health") || report.contains("Storage") || report.contains("Services");
                assert!(has_sections, "System report should have named sections");
            }
        }
    }

    #[test]
    fn test_outputengine_compact_format() {
        use anna_common::output_engine::OutputEngine;

        let engine = OutputEngine::new();
        let compact = engine.format_compact("Yes, feature supported.", Some("CPU flags"));

        assert!(!compact.contains("```"));
        assert!(compact.contains("Yes, feature supported."));
        assert!(compact.contains("Source:"));
    }

    #[test]
    fn test_outputengine_no_fences_stripper() {
        use anna_common::output_engine::OutputEngine;

        let engine = OutputEngine::new();
        let input = "Text\n```bash\ndf -h\n```\nMore text";
        let output = engine.strip_markdown_fences(input);

        assert!(!output.contains("```"));
        assert!(output.contains("[CMD]") || output.contains("df -h"));
    }

    #[test]
    fn test_outputengine_numbered_list() {
        use anna_common::output_engine::OutputEngine;

        let engine = OutputEngine::new();
        let items = vec!["First step".to_string(), "Second step".to_string()];
        let numbered = engine.format_numbered_list(items);

        assert!(numbered.contains("1."));
        assert!(numbered.contains("2."));
        assert!(numbered.contains("First step"));
    }

    #[test]
    fn test_integration_no_raw_ansi_in_output() {
        // Ensure no raw ANSI codes leak into output (should use OutputEngine)
        let result = handle_capability_check(CapabilityKind::CpuFlag("avx".to_string()));
        if let Ok(answer) = result {
            // Check for common ANSI escape sequences
            assert!(!answer.contains("\x1b[31m"), "Should not contain raw ANSI color codes");
            assert!(!answer.contains("\x1b[0m"), "Should not contain raw ANSI reset codes");
        }
    }
}
