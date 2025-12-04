//! Evidence Coverage v0.0.57 - Deterministic coverage scoring
//!
//! Ensures answers are backed by the RIGHT evidence:
//! - Compares requested targets against evidence types returned
//! - Computes coverage percentage (0-100)
//! - Lists missing facets
//! - Triggers automatic retry for low coverage on read-only queries
//!
//! This is NOT hardcoding answers - it's hardcoding what evidence FIELDS
//! are required to be truthful about specific query types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::system_query_router::QueryTarget;
use crate::tools::ToolResult;

// ============================================================================
// Target Facet Definitions
// ============================================================================

/// Required evidence facets for each query target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetFacets {
    /// Target being queried
    pub target: QueryTarget,
    /// Required fields in evidence (must have ALL)
    #[serde(skip)]
    pub required_fields: Vec<&'static str>,
    /// Optional fields (nice to have)
    #[serde(skip)]
    pub optional_fields: Vec<&'static str>,
    /// Tools that provide these facets
    #[serde(skip)]
    pub providing_tools: Vec<&'static str>,
}

/// Get required facets for a query target
pub fn get_target_facets(target: QueryTarget) -> TargetFacets {
    match target {
        QueryTarget::Cpu => TargetFacets {
            target,
            required_fields: vec!["cpu_model", "cores"],
            optional_fields: vec!["threads", "frequency", "cache"],
            providing_tools: vec!["hw_snapshot_cpu", "hw_snapshot_summary"],
        },
        QueryTarget::Memory => TargetFacets {
            target,
            required_fields: vec!["mem_total"],
            optional_fields: vec!["mem_available", "mem_used", "swap_total"],
            providing_tools: vec!["memory_info", "mem_summary"],
        },
        QueryTarget::DiskFree => TargetFacets {
            target,
            required_fields: vec!["root_fs_free", "root_fs_total"],
            optional_fields: vec!["home_free", "mount_points"],
            providing_tools: vec!["mount_usage", "disk_usage"],
        },
        QueryTarget::KernelVersion => TargetFacets {
            target,
            required_fields: vec!["kernel_release"],
            optional_fields: vec!["kernel_version", "arch"],
            providing_tools: vec!["kernel_version", "uname_summary"],
        },
        QueryTarget::NetworkStatus => TargetFacets {
            target,
            required_fields: vec!["link_state", "has_ip"],
            optional_fields: vec!["default_route", "dns_servers", "nm_state"],
            providing_tools: vec!["network_status", "nm_summary", "link_state_summary",
                                  "ip_route_summary", "dns_summary"],
        },
        QueryTarget::AudioStatus => TargetFacets {
            target,
            required_fields: vec!["pipewire_running"],
            optional_fields: vec!["wireplumber_running", "default_sink", "devices"],
            providing_tools: vec!["audio_status", "audio_services_summary", "pactl_summary"],
        },
        QueryTarget::ServicesStatus => TargetFacets {
            target,
            required_fields: vec!["service_active"],
            optional_fields: vec!["service_enabled", "last_error"],
            providing_tools: vec!["service_status", "systemd_service_probe_v1"],
        },
        QueryTarget::Hardware => TargetFacets {
            target,
            required_fields: vec!["cpu_model", "mem_total"],
            optional_fields: vec!["gpu", "storage", "network"],
            providing_tools: vec!["hw_snapshot_summary"],
        },
        QueryTarget::Software => TargetFacets {
            target,
            required_fields: vec!["package_count"],
            optional_fields: vec!["services", "commands"],
            providing_tools: vec!["sw_snapshot_summary"],
        },
        QueryTarget::Alerts => TargetFacets {
            target,
            required_fields: vec!["alert_count"],
            optional_fields: vec!["critical", "warning", "info", "active_alerts"],
            providing_tools: vec!["proactive_alerts_summary", "failed_units_summary",
                                  "disk_pressure_summary", "thermal_status_summary"],
        },
        QueryTarget::Unknown => TargetFacets {
            target,
            required_fields: vec![],
            optional_fields: vec![],
            providing_tools: vec![],
        },
    }
}

// ============================================================================
// Evidence Coverage Result
// ============================================================================

/// Result of evidence coverage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCoverage {
    /// Target being analyzed
    pub target: QueryTarget,
    /// Coverage percentage (0-100)
    pub coverage_percent: u8,
    /// Required fields that are present
    pub present_fields: Vec<String>,
    /// Required fields that are missing
    pub missing_fields: Vec<String>,
    /// Optional fields that are present
    pub optional_present: Vec<String>,
    /// Tools that provided evidence
    pub tools_used: Vec<String>,
    /// Whether coverage is sufficient (>= 90%)
    pub is_sufficient: bool,
    /// Suggested tools to fill gaps
    pub suggested_tools: Vec<String>,
}

/// Coverage threshold for "sufficient" evidence
pub const COVERAGE_SUFFICIENT_THRESHOLD: u8 = 90;

/// Coverage threshold below which Junior must penalize
pub const COVERAGE_PENALTY_THRESHOLD: u8 = 50;

// ============================================================================
// Coverage Analysis Functions
// ============================================================================

/// Analyze evidence coverage for a query target
pub fn analyze_coverage(target: QueryTarget, evidence: &[ToolResult]) -> EvidenceCoverage {
    let facets = get_target_facets(target);

    // Collect all evidence data and tool names
    let mut all_evidence_text = String::new();
    let mut tools_used = Vec::new();

    for e in evidence {
        if e.success {
            all_evidence_text.push_str(&e.human_summary.to_lowercase());
            all_evidence_text.push(' ');
            if let Some(data_str) = e.data.as_str() {
                all_evidence_text.push_str(&data_str.to_lowercase());
            } else {
                all_evidence_text.push_str(&e.data.to_string().to_lowercase());
            }
            all_evidence_text.push(' ');
            tools_used.push(e.tool_name.clone());
        }
    }

    // Check which required fields are present
    let mut present_fields = Vec::new();
    let mut missing_fields = Vec::new();

    for &field in &facets.required_fields {
        if check_field_present(field, &all_evidence_text, target) {
            present_fields.push(field.to_string());
        } else {
            missing_fields.push(field.to_string());
        }
    }

    // Check optional fields
    let mut optional_present = Vec::new();
    for &field in &facets.optional_fields {
        if check_field_present(field, &all_evidence_text, target) {
            optional_present.push(field.to_string());
        }
    }

    // Calculate coverage percentage
    let total_required = facets.required_fields.len();
    let coverage_percent = if total_required == 0 {
        100 // No requirements means full coverage
    } else {
        ((present_fields.len() * 100) / total_required) as u8
    };

    // Determine suggested tools to fill gaps
    let mut suggested_tools = Vec::new();
    if !missing_fields.is_empty() {
        for &tool in &facets.providing_tools {
            if !tools_used.iter().any(|t| t == tool) {
                suggested_tools.push(tool.to_string());
            }
        }
    }

    tools_used.sort();
    tools_used.dedup();

    EvidenceCoverage {
        target,
        coverage_percent,
        present_fields,
        missing_fields,
        optional_present,
        tools_used,
        is_sufficient: coverage_percent >= COVERAGE_SUFFICIENT_THRESHOLD,
        suggested_tools,
    }
}

/// Check if a specific field is present in evidence
fn check_field_present(field: &str, evidence: &str, target: QueryTarget) -> bool {
    match field {
        // CPU fields
        "cpu_model" => {
            evidence.contains("amd") || evidence.contains("intel") ||
            evidence.contains("ryzen") || evidence.contains("core") ||
            evidence.contains("processor") || evidence.contains("cpu:")
        }
        "cores" => {
            evidence.contains("core") || evidence.contains("cores")
        }
        "threads" => evidence.contains("thread"),
        "frequency" => {
            evidence.contains("ghz") || evidence.contains("mhz") ||
            evidence.contains("frequency")
        }

        // Memory fields
        "mem_total" => {
            evidence.contains("total") || evidence.contains("memtotal") ||
            evidence.contains("gib") || evidence.contains("mib") ||
            (evidence.contains("memory") && (evidence.contains("gb") || evidence.contains("mb")))
        }
        "mem_available" => {
            evidence.contains("available") || evidence.contains("memavailable") ||
            evidence.contains("free")
        }
        "mem_used" => evidence.contains("used"),
        "swap_total" => evidence.contains("swap"),

        // Disk fields
        "root_fs_free" => {
            (evidence.contains("free") || evidence.contains("avail")) &&
            (evidence.contains("/") || evidence.contains("root") || evidence.contains("disk"))
        }
        "root_fs_total" => {
            (evidence.contains("total") || evidence.contains("size")) &&
            (evidence.contains("/") || evidence.contains("root") || evidence.contains("disk"))
        }
        "home_free" => evidence.contains("/home"),
        "mount_points" => evidence.contains("mount") || evidence.contains("/"),

        // Kernel fields
        "kernel_release" => {
            // Look for kernel version patterns like 6.x.x or 5.x.x
            evidence.contains("kernel") ||
            evidence.contains("linux") ||
            has_version_pattern(evidence)
        }
        "kernel_version" => evidence.contains("version"),
        "arch" => {
            evidence.contains("x86_64") || evidence.contains("aarch64") ||
            evidence.contains("architecture")
        }

        // Network fields
        "link_state" => {
            evidence.contains("up") || evidence.contains("down") ||
            evidence.contains("state") || evidence.contains("link") ||
            evidence.contains("carrier")
        }
        "has_ip" => {
            evidence.contains("ip") || evidence.contains("inet") ||
            evidence.contains("address") || evidence.contains("192.") ||
            evidence.contains("10.") || evidence.contains("172.")
        }
        "default_route" => {
            evidence.contains("route") || evidence.contains("gateway") ||
            evidence.contains("default")
        }
        "dns_servers" => {
            evidence.contains("dns") || evidence.contains("nameserver") ||
            evidence.contains("resolver")
        }
        "nm_state" => {
            evidence.contains("networkmanager") || evidence.contains("nm") ||
            evidence.contains("connected") || evidence.contains("iwd")
        }

        // Audio fields
        "pipewire_running" => {
            evidence.contains("pipewire") || evidence.contains("audio")
        }
        "wireplumber_running" => evidence.contains("wireplumber"),
        "default_sink" => evidence.contains("sink") || evidence.contains("output"),
        "devices" => evidence.contains("device"),

        // Service fields
        "service_active" => {
            evidence.contains("active") || evidence.contains("running") ||
            evidence.contains("inactive") || evidence.contains("stopped")
        }
        "service_enabled" => {
            evidence.contains("enabled") || evidence.contains("disabled")
        }
        "last_error" => evidence.contains("error") || evidence.contains("failed"),

        // Software fields
        "package_count" => evidence.contains("package"),
        "services" => evidence.contains("service"),
        "commands" => evidence.contains("command") || evidence.contains("binary"),

        // Default
        _ => false,
    }
}

/// Check if evidence contains a kernel version pattern (e.g., 6.7.1-arch1-1)
fn has_version_pattern(evidence: &str) -> bool {
    // Simple check for version-like patterns
    let patterns = ["6.", "5.", "4.", "-arch"];
    patterns.iter().any(|p| evidence.contains(p))
}

// ============================================================================
// Coverage-Based Tool Selection
// ============================================================================

/// Get tools needed to fill coverage gaps
pub fn get_gap_filling_tools(coverage: &EvidenceCoverage) -> Vec<&'static str> {
    if coverage.is_sufficient {
        return Vec::new();
    }

    let facets = get_target_facets(coverage.target);

    // Return tools that weren't used and could provide missing fields
    facets.providing_tools
        .into_iter()
        .filter(|&tool| !coverage.tools_used.iter().any(|t| t == tool))
        .collect()
}

/// Determine if evidence mismatch exists (wrong evidence for target)
pub fn check_evidence_mismatch(target: QueryTarget, evidence: &[ToolResult]) -> Option<String> {
    let coverage = analyze_coverage(target, evidence);

    // Check if we have evidence but it's for the wrong target
    if !evidence.is_empty() && coverage.coverage_percent < COVERAGE_PENALTY_THRESHOLD {
        // Check what the evidence actually contains
        let all_text: String = evidence.iter()
            .filter(|e| e.success)
            .map(|e| e.human_summary.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        // Detect common mismatches
        match target {
            QueryTarget::DiskFree => {
                if all_text.contains("cpu") || all_text.contains("processor") {
                    return Some("Evidence contains CPU info instead of disk info".to_string());
                }
                if all_text.contains("memory") && !all_text.contains("disk") {
                    return Some("Evidence contains memory info instead of disk info".to_string());
                }
            }
            QueryTarget::Memory => {
                if all_text.contains("cpu") && !all_text.contains("mem") {
                    return Some("Evidence contains CPU info instead of memory info".to_string());
                }
                if all_text.contains("disk") && !all_text.contains("mem") {
                    return Some("Evidence contains disk info instead of memory info".to_string());
                }
            }
            QueryTarget::KernelVersion => {
                if all_text.contains("cpu") && !all_text.contains("kernel") {
                    return Some("Evidence contains CPU info instead of kernel info".to_string());
                }
            }
            QueryTarget::NetworkStatus => {
                if all_text.contains("cpu") && !all_text.contains("network") {
                    return Some("Evidence contains CPU info instead of network info".to_string());
                }
            }
            _ => {}
        }

        // Generic mismatch
        if coverage.missing_fields.len() == coverage.present_fields.len() + coverage.missing_fields.len() {
            return Some(format!(
                "Evidence missing required fields: {}",
                coverage.missing_fields.join(", ")
            ));
        }
    }

    None
}

// ============================================================================
// Junior Scoring Adjustments
// ============================================================================

/// Calculate score penalty based on evidence coverage
pub fn calculate_coverage_penalty(coverage: &EvidenceCoverage) -> i32 {
    if coverage.is_sufficient {
        return 0;
    }

    // Penalty scales with how much is missing
    let missing_ratio = if coverage.present_fields.len() + coverage.missing_fields.len() > 0 {
        (coverage.missing_fields.len() * 100) /
        (coverage.present_fields.len() + coverage.missing_fields.len())
    } else {
        0
    };

    match missing_ratio {
        0..=10 => 0,      // Minor gap
        11..=30 => -10,   // Some missing
        31..=50 => -20,   // Significant gap
        51..=70 => -30,   // Major gap
        _ => -50,         // Critical gap
    }
}

/// Get maximum allowed score based on evidence coverage
pub fn get_max_score_for_coverage(coverage: &EvidenceCoverage, has_mismatch: bool) -> u8 {
    if has_mismatch {
        // Wrong evidence entirely - cap at 20%
        return 20;
    }

    if coverage.coverage_percent < 50 {
        // Major missing evidence - cap at 50%
        return 50;
    }

    if coverage.coverage_percent < 90 {
        // Some missing evidence - cap at 70%
        return 70;
    }

    // Full coverage - no cap
    100
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_result(tool: &str, summary: &str) -> ToolResult {
        ToolResult {
            tool_name: tool.to_string(),
            evidence_id: "E1".to_string(),
            data: serde_json::json!({}),
            human_summary: summary.to_string(),
            success: true,
            error: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    #[test]
    fn test_cpu_coverage_full() {
        let evidence = vec![
            make_result("hw_snapshot_cpu", "CPU: AMD Ryzen 7 5800X, 8 cores, 16 threads"),
        ];
        let coverage = analyze_coverage(QueryTarget::Cpu, &evidence);
        assert!(coverage.is_sufficient);
        assert!(coverage.missing_fields.is_empty());
    }

    #[test]
    fn test_disk_coverage_with_cpu_evidence() {
        let evidence = vec![
            make_result("hw_snapshot_cpu", "CPU: AMD Ryzen 7 5800X, 8 cores"),
        ];
        let coverage = analyze_coverage(QueryTarget::DiskFree, &evidence);
        assert!(!coverage.is_sufficient);
        assert!(!coverage.missing_fields.is_empty());
        assert!(coverage.missing_fields.contains(&"root_fs_free".to_string()));
    }

    #[test]
    fn test_disk_coverage_correct() {
        let evidence = vec![
            make_result("mount_usage", "Disk /: 433 GiB free of 500 GiB total (13% used)"),
        ];
        let coverage = analyze_coverage(QueryTarget::DiskFree, &evidence);
        assert!(coverage.is_sufficient);
    }

    #[test]
    fn test_memory_coverage() {
        let evidence = vec![
            make_result("memory_info", "Memory: 32 GiB total, 24 GiB available, 8 GiB used"),
        ];
        let coverage = analyze_coverage(QueryTarget::Memory, &evidence);
        assert!(coverage.is_sufficient);
    }

    #[test]
    fn test_kernel_coverage() {
        let evidence = vec![
            make_result("kernel_version", "Linux kernel 6.7.1-arch1-1"),
        ];
        let coverage = analyze_coverage(QueryTarget::KernelVersion, &evidence);
        assert!(coverage.is_sufficient);
    }

    #[test]
    fn test_network_coverage_partial() {
        let evidence = vec![
            make_result("link_state_summary", "eth0: UP, carrier present"),
        ];
        let coverage = analyze_coverage(QueryTarget::NetworkStatus, &evidence);
        // Only has link_state, missing has_ip
        assert!(coverage.present_fields.contains(&"link_state".to_string()));
    }

    #[test]
    fn test_mismatch_detection() {
        let evidence = vec![
            make_result("hw_snapshot_cpu", "CPU: AMD Ryzen 7 5800X, 8 cores"),
        ];
        let mismatch = check_evidence_mismatch(QueryTarget::DiskFree, &evidence);
        assert!(mismatch.is_some());
        assert!(mismatch.unwrap().contains("CPU"));
    }

    #[test]
    fn test_gap_filling_tools() {
        let evidence = vec![
            make_result("hw_snapshot_cpu", "CPU info only"),
        ];
        let coverage = analyze_coverage(QueryTarget::DiskFree, &evidence);
        let tools = get_gap_filling_tools(&coverage);
        assert!(tools.contains(&"mount_usage"));
    }

    #[test]
    fn test_max_score_mismatch() {
        let coverage = EvidenceCoverage {
            target: QueryTarget::DiskFree,
            coverage_percent: 0,
            present_fields: vec![],
            missing_fields: vec!["root_fs_free".to_string()],
            optional_present: vec![],
            tools_used: vec![],
            is_sufficient: false,
            suggested_tools: vec![],
        };
        let max = get_max_score_for_coverage(&coverage, true);
        assert_eq!(max, 20);
    }

    #[test]
    fn test_max_score_partial() {
        let coverage = EvidenceCoverage {
            target: QueryTarget::NetworkStatus,
            coverage_percent: 75,
            present_fields: vec!["link_state".to_string()],
            missing_fields: vec!["has_ip".to_string()],
            optional_present: vec![],
            tools_used: vec![],
            is_sufficient: false,
            suggested_tools: vec![],
        };
        let max = get_max_score_for_coverage(&coverage, false);
        assert_eq!(max, 70);
    }
}
