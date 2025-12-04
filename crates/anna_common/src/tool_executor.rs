//! Tool Executor v0.0.58
//!
//! v0.0.58: Proactive alerts tools (proactive_alerts_summary, disk_pressure_summary, etc.)
//! v0.0.51: Systemd service action tools (probe, preview, apply, rollback)
//! v0.0.50: User file edit tools (file_edit_preview_v1, file_edit_apply_v1, file_edit_rollback_v1)
//! v0.0.49: Doctor network evidence tools (net_interfaces_summary, dns_summary, etc.)
//! v0.0.48: Knowledge search tools (learned_recipe_search, learning_stats)
//! v0.0.47: File evidence tools for mutation support (file_stat, file_preview, file_hash, path_policy_check)
//! v0.0.46: Domain-specific evidence tools to prevent generic summary answers
//!
//! Executes read-only tools from the catalog with structured outputs
//! and human-readable summaries.

use crate::alert_probes::{
    probe_alerts_summary, probe_disk_pressure_summary, probe_failed_units_summary,
    probe_journal_error_burst_summary, probe_thermal_summary,
};
use crate::anomaly_engine::{analyze_slowness, what_changed, AlertQueue};
use crate::daemon_state::StatusSnapshot;
use crate::doctor_network_tools::{
    execute_dns_summary, execute_iw_summary, execute_net_interfaces_summary,
    execute_net_routes_summary, execute_ping_check, execute_recent_network_errors,
};
use crate::file_edit_tools::{
    execute_file_edit_apply_v1, execute_file_edit_preview_v1, execute_file_edit_rollback_v1,
};
use crate::knowledge_packs::{KnowledgeIndex, DEFAULT_TOP_K};
use crate::reliability::{
    calculate_budget_status, check_budget_alerts, DiagnosticsReport, ErrorBudgets, MetricsStore,
};
use crate::snapshots::{HwSnapshot, SwSnapshot};
use crate::source_labels::{classify_question_type, AnswerContext, QaStats, SourcePlan};
use crate::systemd_tools::{
    execute_systemd_service_apply_v1, execute_systemd_service_preview_v1,
    execute_systemd_service_probe_v1, execute_systemd_service_rollback_v1,
};
use crate::tools::{
    unavailable_result, unknown_tool_result, EvidenceCollector, ToolCatalog, ToolRequest,
    ToolResult,
};
use serde_json::json;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Execute a tool and return the result with evidence ID
pub fn execute_tool(request: &ToolRequest, catalog: &ToolCatalog, evidence_id: &str) -> ToolResult {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    // Check if tool exists in catalog
    if !catalog.exists(&request.tool_name) {
        return unknown_tool_result(&request.tool_name, evidence_id);
    }

    match request.tool_name.as_str() {
        "status_snapshot" => execute_status_snapshot(evidence_id, timestamp),
        "sw_snapshot_summary" => execute_sw_snapshot_summary(evidence_id, timestamp),
        "hw_snapshot_summary" => execute_hw_snapshot_summary(evidence_id, timestamp),
        "recent_installs" => execute_recent_installs(&request.parameters, evidence_id, timestamp),
        "journal_warnings" => execute_journal_warnings(&request.parameters, evidence_id, timestamp),
        "boot_time_trend" => execute_boot_time_trend(&request.parameters, evidence_id, timestamp),
        "top_resource_processes" => {
            execute_top_resource_processes(&request.parameters, evidence_id, timestamp)
        }
        "package_info" => execute_package_info(&request.parameters, evidence_id, timestamp),
        "service_status" => execute_service_status(&request.parameters, evidence_id, timestamp),
        "disk_usage" => execute_disk_usage(evidence_id, timestamp),
        // v0.0.12: Anomaly detection tools
        "active_alerts" => execute_active_alerts(evidence_id, timestamp),
        "what_changed" => execute_what_changed(&request.parameters, evidence_id, timestamp),
        "slowness_hypotheses" => {
            execute_slowness_hypotheses(&request.parameters, evidence_id, timestamp)
        }
        // v0.0.19: Knowledge pack tools
        "knowledge_search" => execute_knowledge_search(&request.parameters, evidence_id, timestamp),
        "knowledge_stats" => execute_knowledge_stats(evidence_id, timestamp),
        // v0.0.20: Ask Me Anything tools
        "answer_context" => execute_answer_context(evidence_id, timestamp),
        "source_plan" => execute_source_plan(&request.parameters, evidence_id, timestamp),
        "qa_stats" => execute_qa_stats(evidence_id, timestamp),
        // v0.0.22: Reliability Engineering tools
        "self_diagnostics" => execute_self_diagnostics(evidence_id, timestamp),
        "metrics_summary" => execute_metrics_summary(&request.parameters, evidence_id, timestamp),
        "error_budgets" => execute_error_budgets(evidence_id, timestamp),
        // v0.0.33: Case file retrieval tools
        "last_case_summary" => execute_last_case_summary(evidence_id, timestamp),
        "last_failure_summary" => execute_last_failure_summary(evidence_id, timestamp),
        "list_today_cases" => execute_list_today_cases(evidence_id, timestamp),
        "list_recent_cases" => {
            execute_list_recent_cases(&request.parameters, evidence_id, timestamp)
        }
        // v0.0.45: Direct evidence tools for correctness
        "kernel_version" => execute_kernel_version(evidence_id, timestamp),
        "memory_info" => execute_memory_info(evidence_id, timestamp),
        "network_status" => execute_network_status(evidence_id, timestamp),
        "audio_status" => execute_audio_status(evidence_id, timestamp),
        // v0.0.46: Domain-specific evidence tools
        "uname_summary" => execute_uname_summary(evidence_id, timestamp),
        "mem_summary" => execute_mem_summary(evidence_id, timestamp),
        "mount_usage" => execute_mount_usage(evidence_id, timestamp),
        "nm_summary" => execute_nm_summary(evidence_id, timestamp),
        "ip_route_summary" => execute_ip_route_summary(evidence_id, timestamp),
        "link_state_summary" => execute_link_state_summary(evidence_id, timestamp),
        "audio_services_summary" => execute_audio_services_summary(evidence_id, timestamp),
        "pactl_summary" => execute_pactl_summary(evidence_id, timestamp),
        "boot_time_summary" => execute_boot_time_summary(evidence_id, timestamp),
        "recent_errors_summary" => {
            execute_recent_errors_summary(&request.parameters, evidence_id, timestamp)
        }
        // v0.0.47: File evidence tools for mutations
        "file_stat" => execute_file_stat(&request.parameters, evidence_id, timestamp),
        "file_preview" => execute_file_preview(&request.parameters, evidence_id, timestamp),
        "file_hash" => execute_file_hash(&request.parameters, evidence_id, timestamp),
        "path_policy_check" => {
            execute_path_policy_check(&request.parameters, evidence_id, timestamp)
        }
        // v0.0.48: Knowledge search tools
        "learned_recipe_search" => {
            execute_learned_recipe_search(&request.parameters, evidence_id, timestamp)
        }
        "learning_stats" => execute_learning_stats(evidence_id, timestamp),
        // v0.0.49: Doctor network evidence tools
        "net_interfaces_summary" => execute_net_interfaces_summary(evidence_id, timestamp),
        "net_routes_summary" => execute_net_routes_summary(evidence_id, timestamp),
        "dns_summary" => execute_dns_summary(evidence_id, timestamp),
        "iw_summary" => execute_iw_summary(evidence_id, timestamp),
        "recent_network_errors" => {
            execute_recent_network_errors(&request.parameters, evidence_id, timestamp)
        }
        "ping_check" => execute_ping_check(&request.parameters, evidence_id, timestamp),
        // v0.0.50: User file edit tools
        "file_edit_preview_v1" => {
            execute_file_edit_preview_v1(&request.parameters, evidence_id, timestamp)
        }
        "file_edit_apply_v1" => {
            execute_file_edit_apply_v1(&request.parameters, evidence_id, timestamp)
        }
        "file_edit_rollback_v1" => {
            execute_file_edit_rollback_v1(&request.parameters, evidence_id, timestamp)
        }
        // v0.0.51: Systemd service action tools
        "systemd_service_probe_v1" => {
            execute_systemd_service_probe_v1(&request.parameters, evidence_id, timestamp)
        }
        "systemd_service_preview_v1" => {
            execute_systemd_service_preview_v1(&request.parameters, evidence_id, timestamp)
        }
        "systemd_service_apply_v1" => {
            execute_systemd_service_apply_v1(&request.parameters, evidence_id, timestamp)
        }
        "systemd_service_rollback_v1" => {
            execute_systemd_service_rollback_v1(&request.parameters, evidence_id, timestamp)
        }
        // v0.0.58: Proactive alerts tools
        "proactive_alerts_summary" => execute_proactive_alerts_summary(evidence_id, timestamp),
        "disk_pressure_summary" => execute_disk_pressure_summary(evidence_id, timestamp),
        "failed_units_summary" => execute_failed_units_summary(evidence_id, timestamp),
        "thermal_status_summary" => execute_thermal_status_summary(evidence_id, timestamp),
        "journal_error_burst_summary" => {
            execute_journal_error_burst_summary(evidence_id, timestamp)
        }
        _ => unavailable_result(&request.tool_name, evidence_id),
    }
}

/// Execute all tools in a plan and collect evidence
pub fn execute_tool_plan(
    requests: &[ToolRequest],
    catalog: &ToolCatalog,
    collector: &mut EvidenceCollector,
) -> Vec<ToolResult> {
    let mut results = Vec::new();

    for request in requests {
        let evidence_id = collector.next_id();
        let mut result = execute_tool(request, catalog, &evidence_id);
        result.evidence_id = evidence_id;
        collector.push(result.clone());
        results.push(result);
    }

    results
}

// Individual tool implementations

fn execute_status_snapshot(evidence_id: &str, timestamp: u64) -> ToolResult {
    match StatusSnapshot::load() {
        Some(snapshot) => {
            let uptime_str = format_duration(snapshot.uptime_secs);

            let llm_status = snapshot.llm_bootstrap_phase.as_deref().unwrap_or("unknown");

            let human_summary = format!(
                "Daemon uptime: {}, LLM status: {}, schema v{}",
                uptime_str, llm_status, snapshot.schema_version
            );

            ToolResult {
                tool_name: "status_snapshot".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "schema_version": snapshot.schema_version,
                    "uptime_secs": snapshot.uptime_secs,
                    "llm_bootstrap_phase": snapshot.llm_bootstrap_phase,
                    "llm_translator_model": snapshot.llm_translator_model,
                    "llm_junior_model": snapshot.llm_junior_model,
                    "healthy": snapshot.healthy,
                    "version": snapshot.version,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        None => ToolResult {
            tool_name: "status_snapshot".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Status snapshot not available"}),
            human_summary: "Status snapshot is not available (daemon may not be running)."
                .to_string(),
            success: false,
            error: Some("Snapshot not found".to_string()),
            timestamp,
        },
    }
}

fn execute_sw_snapshot_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    match SwSnapshot::load() {
        Some(snapshot) => {
            let pkg_count = snapshot.packages.total;
            let cmd_count = snapshot.commands.total;
            let svc_count = snapshot.services.total;
            let running_services = snapshot.services.running;
            let failed_services = snapshot.services.failed;

            let human_summary = format!(
                "Found {} packages ({} explicit, {} AUR), {} commands, {} services ({} running, {} failed)",
                pkg_count, snapshot.packages.explicit, snapshot.packages.aur,
                cmd_count, svc_count, running_services, failed_services
            );

            // Extract categories
            let categories: HashMap<String, usize> = snapshot
                .categories
                .iter()
                .map(|c| (c.name.clone(), c.count))
                .collect();

            ToolResult {
                tool_name: "sw_snapshot_summary".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "package_count": pkg_count,
                    "package_explicit": snapshot.packages.explicit,
                    "package_aur": snapshot.packages.aur,
                    "command_count": cmd_count,
                    "service_count": svc_count,
                    "services_running": running_services,
                    "services_failed": failed_services,
                    "failed_services": snapshot.services.failed_services,
                    "categories": categories,
                    "generated_at": snapshot.generated_at.to_rfc3339(),
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        None => ToolResult {
            tool_name: "sw_snapshot_summary".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Software snapshot not available"}),
            human_summary: "Software snapshot is not available.".to_string(),
            success: false,
            error: Some("Snapshot not found".to_string()),
            timestamp,
        },
    }
}

fn execute_hw_snapshot_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    match HwSnapshot::load() {
        Some(snapshot) => {
            let cpu_info = format!(
                "{} ({} cores, {} threads)",
                snapshot.cpu.model, snapshot.cpu.cores, snapshot.cpu.threads
            );

            let memory_gb = snapshot.memory.total_bytes / (1024 * 1024 * 1024);

            let gpu_info = snapshot
                .gpu
                .as_ref()
                .map(|g| g.name.clone())
                .unwrap_or_else(|| "none detected".to_string());

            let storage_count = snapshot.storage.len();
            let network_count = snapshot.network.len();

            let human_summary = format!(
                "CPU: {}, RAM: {} GB, GPU: {}, {} storage devices, {} network interfaces",
                cpu_info, memory_gb, gpu_info, storage_count, network_count
            );

            ToolResult {
                tool_name: "hw_snapshot_summary".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "cpu": {
                        "model": snapshot.cpu.model,
                        "cores": snapshot.cpu.cores,
                        "threads": snapshot.cpu.threads,
                    },
                    "memory": {
                        "total_gb": memory_gb,
                        "total_bytes": snapshot.memory.total_bytes,
                    },
                    "gpu": snapshot.gpu.as_ref().map(|g| &g.name),
                    "storage_count": storage_count,
                    "network_count": network_count,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        None => ToolResult {
            tool_name: "hw_snapshot_summary".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Hardware snapshot not available"}),
            human_summary: "Hardware snapshot is not available.".to_string(),
            success: false,
            error: Some("Snapshot not found".to_string()),
            timestamp,
        },
    }
}

fn execute_recent_installs(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let days = params.get("days").and_then(|v| v.as_i64()).unwrap_or(14) as i64;

    // Read pacman.log and find recent installs
    let pacman_log = std::fs::read_to_string("/var/log/pacman.log").unwrap_or_default();
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

    let mut installs: Vec<(String, String)> = Vec::new();

    for line in pacman_log.lines() {
        if line.contains("installed") && !line.contains("reinstalled") {
            // Parse: [2024-01-15T10:30:00+0000] [ALPM] installed package (version)
            if let Some(date_end) = line.find(']') {
                let date_str = &line[1..date_end];
                if let Some(date_part) = date_str.split('T').next() {
                    if date_part >= cutoff_str.as_str() {
                        // Extract package name
                        if let Some(pkg_start) = line.find("installed ") {
                            let rest = &line[pkg_start + 10..];
                            if let Some(pkg_end) = rest.find(' ') {
                                let pkg_name = rest[..pkg_end].to_string();
                                installs.push((date_part.to_string(), pkg_name));
                            }
                        }
                    }
                }
            }
        }
    }

    let count = installs.len();
    let human_summary = format!(
        "Found {} packages installed in the last {} days",
        count, days
    );

    ToolResult {
        tool_name: "recent_installs".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "days": days,
            "count": count,
            "installs": installs.iter().take(20).collect::<Vec<_>>(), // Limit to 20
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_journal_warnings(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let service = params
        .get("service")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let minutes = params.get("minutes").and_then(|v| v.as_i64()).unwrap_or(60) as u64;

    // Run journalctl with priority filter
    let mut cmd = Command::new("journalctl");
    cmd.arg("--no-pager")
        .arg("-p")
        .arg("warning")
        .arg("--since")
        .arg(format!("{} minutes ago", minutes))
        .arg("-n")
        .arg("50"); // Limit entries

    if let Some(svc) = &service {
        cmd.arg("-u").arg(svc);
    }

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().take(30).collect();
            let total_lines = stdout.lines().count();

            let human_summary = if let Some(svc) = &service {
                format!(
                    "Found {} warnings/errors for {} in the last {} minutes",
                    total_lines, svc, minutes
                )
            } else {
                format!(
                    "Found {} system warnings/errors in the last {} minutes",
                    total_lines, minutes
                )
            };

            ToolResult {
                tool_name: "journal_warnings".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "service": service,
                    "minutes": minutes,
                    "total_count": total_lines,
                    "entries": lines,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "journal_warnings".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": e.to_string()}),
            human_summary: format!("Failed to read journal: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_boot_time_trend(
    _params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    // Use systemd-analyze for boot time
    match Command::new("systemd-analyze").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let first_line = stdout.lines().next().unwrap_or("unknown");

            // Extract boot time from: "Startup finished in 2.5s (kernel) + 5.3s (userspace) = 7.8s"
            let total_time = if let Some(eq_pos) = first_line.rfind("= ") {
                first_line[eq_pos + 2..].trim().to_string()
            } else {
                "unknown".to_string()
            };

            let human_summary = format!("Current boot time: {}", total_time);

            ToolResult {
                tool_name: "boot_time_trend".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "current_boot_time": total_time,
                    "raw": first_line,
                    "note": "Historical trend data not yet implemented"
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => unavailable_result("boot_time_trend", evidence_id),
    }
}

fn execute_top_resource_processes(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    // Use ps to get top CPU/memory consumers
    match Command::new("ps").args(["aux", "--sort=-%cpu"]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().take(11).collect(); // Header + top 10

            let process_count = lines.len().saturating_sub(1);
            let human_summary = format!(
                "Retrieved top {} resource-consuming processes",
                process_count
            );

            ToolResult {
                tool_name: "top_resource_processes".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "processes": lines,
                    "count": process_count,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "top_resource_processes".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": e.to_string()}),
            human_summary: format!("Failed to get process list: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_package_info(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return ToolResult {
            tool_name: "package_info".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Package name required"}),
            human_summary: "Package name is required.".to_string(),
            success: false,
            error: Some("Missing parameter: name".to_string()),
            timestamp,
        };
    }

    match Command::new("pacman").args(["-Qi", name]).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Extract key fields
            let mut info: HashMap<String, String> = HashMap::new();
            for line in stdout.lines() {
                if let Some(colon) = line.find(':') {
                    let key = line[..colon].trim().to_lowercase().replace(' ', "_");
                    let value = line[colon + 1..].trim().to_string();
                    info.insert(key, value);
                }
            }

            let version = info
                .get("version")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let installed_size = info
                .get("installed_size")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let human_summary = format!(
                "Package {}: version {}, size {}",
                name, version, installed_size
            );

            ToolResult {
                tool_name: "package_info".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "name": name,
                    "info": info,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Ok(_) => ToolResult {
            tool_name: "package_info".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Package not found"}),
            human_summary: format!("Package '{}' is not installed.", name),
            success: false,
            error: Some("Package not found".to_string()),
            timestamp,
        },
        Err(e) => ToolResult {
            tool_name: "package_info".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": e.to_string()}),
            human_summary: format!("Failed to query package: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_service_status(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return ToolResult {
            tool_name: "service_status".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Service name required"}),
            human_summary: "Service name is required.".to_string(),
            success: false,
            error: Some("Missing parameter: name".to_string()),
            timestamp,
        };
    }

    // Ensure .service suffix
    let unit_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    match Command::new("systemctl")
        .args(["status", &unit_name])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().take(10).collect();

            // Extract state from output
            let is_active = stdout.contains("Active: active");
            let is_enabled = stdout.contains("enabled;");

            let state = if is_active { "active" } else { "inactive" };
            let enabled = if is_enabled { "enabled" } else { "disabled" };

            let human_summary = format!("Service {}: {} ({})", unit_name, state, enabled);

            ToolResult {
                tool_name: "service_status".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "name": unit_name,
                    "active": is_active,
                    "enabled": is_enabled,
                    "output": lines,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "service_status".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": e.to_string()}),
            human_summary: format!("Failed to check service: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_disk_usage(evidence_id: &str, timestamp: u64) -> ToolResult {
    match Command::new("df")
        .args(["-h", "--output=source,size,used,avail,pcent,target"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            // Parse filesystem entries
            let mut filesystems: Vec<serde_json::Value> = Vec::new();
            let mut root_info: Option<serde_json::Value> = None;

            for line in lines.iter().skip(1) {
                // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    let fs = json!({
                        "source": parts[0],
                        "size": parts[1],
                        "used": parts[2],
                        "available": parts[3],
                        "use_percent": parts[4],
                        "mountpoint": parts[5],
                    });
                    filesystems.push(fs.clone());

                    // Track root filesystem specifically
                    if parts[5] == "/" {
                        root_info = Some(fs);
                    }
                }
            }

            // Get root filesystem free space for summary
            let (root_free, root_used_pct) = if let Some(ref root) = root_info {
                (
                    root.get("available")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    root.get("use_percent")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                )
            } else {
                ("unknown".to_string(), "unknown".to_string())
            };

            let human_summary = format!(
                "Disk: / has {} free ({} used), {} filesystems mounted",
                root_free,
                root_used_pct,
                filesystems.len()
            );

            ToolResult {
                tool_name: "disk_usage".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "filesystems": filesystems,
                    "root_filesystem": root_info,
                    "root_free": root_free,
                    "root_used_percent": root_used_pct,
                    "filesystem_count": filesystems.len(),
                    "raw_lines": lines,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "disk_usage".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": e.to_string()}),
            human_summary: format!("Failed to check disk usage: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

// v0.0.12: Anomaly detection tool implementations

fn execute_active_alerts(evidence_id: &str, timestamp: u64) -> ToolResult {
    let queue = AlertQueue::load();
    let (critical, warning, info) = queue.count_by_severity();
    let active = queue.get_active();

    let human_summary = if active.is_empty() {
        "No active alerts".to_string()
    } else {
        format!(
            "{} active alerts: {} critical, {} warnings, {} info",
            active.len(),
            critical,
            warning,
            info
        )
    };

    let alerts_data: Vec<serde_json::Value> = active
        .iter()
        .map(|a| {
            json!({
                "evidence_id": a.evidence_id,
                "severity": a.severity.as_str(),
                "title": a.title,
                "description": a.description,
                "confidence": a.confidence,
                "occurrence_count": a.occurrence_count,
                "hints": a.hints,
            })
        })
        .collect();

    ToolResult {
        tool_name: "active_alerts".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "total": active.len(),
            "critical": critical,
            "warning": warning,
            "info": info,
            "alerts": alerts_data,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_what_changed(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let days = params.get("days").and_then(|v| v.as_i64()).unwrap_or(7) as u32;

    let result = what_changed(days);
    let human_summary = result.format_summary();

    ToolResult {
        tool_name: "what_changed".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "evidence_id": result.evidence_id,
            "days": days,
            "packages_installed": result.packages_installed.len(),
            "packages_removed": result.packages_removed.len(),
            "packages_upgraded": result.packages_upgraded.len(),
            "services_enabled": result.services_enabled,
            "services_disabled": result.services_disabled,
            "config_changes": result.config_changes.len(),
            "anna_updates": result.anna_updates,
            "has_changes": result.has_changes(),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_slowness_hypotheses(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let days = params.get("days").and_then(|v| v.as_i64()).unwrap_or(7) as u32;

    let result = analyze_slowness(days);

    let human_summary = if result.hypotheses.is_empty() {
        "No specific slowness hypotheses identified".to_string()
    } else {
        let top = &result.hypotheses[0];
        format!(
            "{} hypotheses, top: {} ({:.0}% confidence)",
            result.hypotheses.len(),
            top.title,
            top.confidence * 100.0
        )
    };

    let hypotheses_data: Vec<serde_json::Value> = result
        .hypotheses
        .iter()
        .map(|h| {
            json!({
                "evidence_id": h.evidence_id,
                "title": h.title,
                "explanation": h.explanation,
                "confidence": h.confidence,
                "supporting_evidence": h.supporting_evidence,
                "suggested_diagnostics": h.suggested_diagnostics,
            })
        })
        .collect();

    ToolResult {
        tool_name: "slowness_hypotheses".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "analysis_evidence_id": result.evidence_id,
            "days": days,
            "hypothesis_count": result.hypotheses.len(),
            "hypotheses": hypotheses_data,
            "changes_summary": result.changes_summary,
            "active_anomalies": result.active_anomalies,
            "top_processes": result.top_processes,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Format duration in human-readable form
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

// v0.0.19: Knowledge pack tool implementations

fn execute_knowledge_search(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");

    if query.is_empty() {
        return ToolResult {
            tool_name: "knowledge_search".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Query parameter required"}),
            human_summary: "Search query is required.".to_string(),
            success: false,
            error: Some("Missing parameter: query".to_string()),
            timestamp,
        };
    }

    let top_k = params
        .get("top_k")
        .and_then(|v| v.as_i64())
        .unwrap_or(DEFAULT_TOP_K as i64) as usize;

    // Open the knowledge index
    match KnowledgeIndex::open() {
        Ok(index) => {
            match index.search(query, top_k) {
                Ok(results) => {
                    let result_count = results.len();

                    // Format results for JSON output
                    let results_data: Vec<serde_json::Value> = results
                        .iter()
                        .map(|r| {
                            json!({
                                "evidence_id": r.evidence_id,
                                "title": r.title,
                                "pack_id": r.pack_id,
                                "pack_name": r.pack_name,
                                "source_path": r.source_path,
                                "trust": format!("{:?}", r.trust),
                                "excerpt": r.excerpt,
                                "score": r.score,
                                "matched_keywords": r.matched_keywords,
                            })
                        })
                        .collect();

                    let human_summary = if result_count == 0 {
                        format!("No results found for query: '{}'", query)
                    } else {
                        let top_titles: Vec<&str> =
                            results.iter().take(3).map(|r| r.title.as_str()).collect();
                        format!(
                            "Found {} results for '{}'. Top matches: {}",
                            result_count,
                            query,
                            top_titles.join(", ")
                        )
                    };

                    ToolResult {
                        tool_name: "knowledge_search".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({
                            "query": query,
                            "top_k": top_k,
                            "result_count": result_count,
                            "results": results_data,
                        }),
                        human_summary,
                        success: true,
                        error: None,
                        timestamp,
                    }
                }
                Err(e) => ToolResult {
                    tool_name: "knowledge_search".to_string(),
                    evidence_id: evidence_id.to_string(),
                    data: json!({"error": e.to_string()}),
                    human_summary: format!("Search failed: {}", e),
                    success: false,
                    error: Some(e.to_string()),
                    timestamp,
                },
            }
        }
        Err(e) => ToolResult {
            tool_name: "knowledge_search".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": e.to_string(), "hint": "Knowledge index may not be initialized yet"}),
            human_summary: format!("Failed to open knowledge index: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_knowledge_stats(evidence_id: &str, timestamp: u64) -> ToolResult {
    match KnowledgeIndex::open() {
        Ok(index) => {
            match index.get_stats() {
                Ok(stats) => {
                    // Format index size
                    let size_str = if stats.total_size_bytes < 1024 {
                        format!("{} B", stats.total_size_bytes)
                    } else if stats.total_size_bytes < 1024 * 1024 {
                        format!("{:.1} KB", stats.total_size_bytes as f64 / 1024.0)
                    } else {
                        format!(
                            "{:.1} MB",
                            stats.total_size_bytes as f64 / (1024.0 * 1024.0)
                        )
                    };

                    // Format last index time
                    let last_indexed = match stats.last_indexed_at {
                        Some(t) if t > 0 => {
                            let duration = timestamp.saturating_sub(t);
                            format!("{} ago", format_duration(duration))
                        }
                        _ => "never".to_string(),
                    };

                    let human_summary = format!(
                        "{} packs, {} documents, {} index size, last indexed {}",
                        stats.pack_count, stats.document_count, size_str, last_indexed
                    );

                    ToolResult {
                        tool_name: "knowledge_stats".to_string(),
                        evidence_id: evidence_id.to_string(),
                        data: json!({
                            "pack_count": stats.pack_count,
                            "document_count": stats.document_count,
                            "total_size_bytes": stats.total_size_bytes,
                            "index_size_human": size_str,
                            "last_indexed_at": stats.last_indexed_at,
                            "last_indexed_human": last_indexed,
                            "packs_by_source": stats.packs_by_source,
                            "top_packs": stats.top_packs,
                        }),
                        human_summary,
                        success: true,
                        error: None,
                        timestamp,
                    }
                }
                Err(e) => ToolResult {
                    tool_name: "knowledge_stats".to_string(),
                    evidence_id: evidence_id.to_string(),
                    data: json!({"error": e.to_string()}),
                    human_summary: format!("Failed to get stats: {}", e),
                    success: false,
                    error: Some(e.to_string()),
                    timestamp,
                },
            }
        }
        Err(e) => ToolResult {
            tool_name: "knowledge_stats".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "pack_count": 0,
                "document_count": 0,
                "total_size_bytes": 0,
                "hint": "Knowledge index not initialized"
            }),
            human_summary: "Knowledge index not initialized. Run ingestion first.".to_string(),
            success: true, // Not a failure, just empty
            error: None,
            timestamp,
        },
    }
}

// v0.0.20: Ask Me Anything tool implementations

fn execute_answer_context(evidence_id: &str, timestamp: u64) -> ToolResult {
    let context = AnswerContext::build();

    let human_summary = format!(
        "Distro: {}, Knowledge: {} packs ({} docs)",
        context.distro, context.knowledge_packs_available, context.knowledge_docs_count
    );

    ToolResult {
        tool_name: "answer_context".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "target_user": context.target_user,
            "distro": context.distro,
            "kernel_version": context.kernel_version,
            "relevant_packages": context.relevant_packages,
            "relevant_services": context.relevant_services,
            "knowledge_packs_available": context.knowledge_packs_available,
            "knowledge_docs_count": context.knowledge_docs_count,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_source_plan(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let request = params.get("request").and_then(|v| v.as_str()).unwrap_or("");

    if request.is_empty() {
        return ToolResult {
            tool_name: "source_plan".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Request parameter required"}),
            human_summary: "Request parameter is required.".to_string(),
            success: false,
            error: Some("Missing parameter: request".to_string()),
            timestamp,
        };
    }

    let question_type = classify_question_type(request);
    let plan = SourcePlan::create(request, question_type);

    let human_summary = plan.rationale.clone();

    ToolResult {
        tool_name: "source_plan".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "question_type": plan.question_type.as_str(),
            "primary_sources": plan.primary_sources.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            "knowledge_query": plan.knowledge_query,
            "system_tools": plan.system_tools,
            "rationale": plan.rationale,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_qa_stats(evidence_id: &str, timestamp: u64) -> ToolResult {
    let stats = QaStats::load_today();

    let top_sources: Vec<String> = stats
        .top_source_types(3)
        .iter()
        .map(|(name, count)| format!("{}: {}", name, count))
        .collect();

    let human_summary = if stats.answers_count == 0 {
        "No Q&A answers today yet.".to_string()
    } else {
        format!(
            "{} answers today, avg reliability {}%, top sources: {}",
            stats.answers_count,
            stats.avg_reliability(),
            if top_sources.is_empty() {
                "none".to_string()
            } else {
                top_sources.join(", ")
            }
        )
    };

    ToolResult {
        tool_name: "qa_stats".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "date": stats.date,
            "answers_count": stats.answers_count,
            "avg_reliability": stats.avg_reliability(),
            "total_reliability": stats.total_reliability,
            "knowledge_citations": stats.knowledge_citations,
            "evidence_citations": stats.evidence_citations,
            "reasoning_labels": stats.reasoning_labels,
            "top_source_types": stats.top_source_types(5),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

// v0.0.22: Reliability Engineering tool implementations

fn execute_self_diagnostics(evidence_id: &str, timestamp: u64) -> ToolResult {
    let report = DiagnosticsReport::generate();

    let human_summary = format!(
        "Self-diagnostics report generated [{}]: Overall status: {}, {} sections",
        report.report_evidence_id,
        report.overall_status.as_str(),
        report.sections.len()
    );

    let report_text = report.to_text();

    ToolResult {
        tool_name: "self_diagnostics".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "generated_at": report.generated_at,
            "version": report.version,
            "overall_status": report.overall_status.as_str(),
            "sections": report.sections.iter().map(|s| json!({
                "title": s.title,
                "evidence_id": s.evidence_id,
                "status": s.status.as_str(),
                "content": s.content,
            })).collect::<Vec<_>>(),
            "report_text": report_text,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_metrics_summary(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let days = params.get("days").and_then(|v| v.as_u64()).unwrap_or(1) as u32;

    let metrics = MetricsStore::load();
    let totals = metrics.total_counts(days);

    // Calculate success rates
    let request_success = *totals.get("request_success").unwrap_or(&0);
    let request_failure = *totals.get("request_failure").unwrap_or(&0);
    let request_total = request_success + request_failure;
    let request_rate = if request_total > 0 {
        (request_success as f64 / request_total as f64) * 100.0
    } else {
        100.0
    };

    let tool_success = *totals.get("tool_success").unwrap_or(&0);
    let tool_failure = *totals.get("tool_failure").unwrap_or(&0);
    let tool_total = tool_success + tool_failure;
    let tool_rate = if tool_total > 0 {
        (tool_success as f64 / tool_total as f64) * 100.0
    } else {
        100.0
    };

    let cache_hits = *totals.get("cache_hit").unwrap_or(&0);
    let cache_misses = *totals.get("cache_miss").unwrap_or(&0);
    let cache_total = cache_hits + cache_misses;
    let cache_rate = if cache_total > 0 {
        (cache_hits as f64 / cache_total as f64) * 100.0
    } else {
        0.0
    };

    // Get latency percentiles from today
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let (p50_e2e, p95_e2e) = metrics
        .for_date(&today)
        .map(|m| {
            (
                m.percentile_latency("e2e", 50.0).unwrap_or(0),
                m.percentile_latency("e2e", 95.0).unwrap_or(0),
            )
        })
        .unwrap_or((0, 0));

    let human_summary = format!(
        "Metrics ({} day): request success {:.1}%, tool success {:.1}%, cache hit {:.1}%, p50 {}ms, p95 {}ms",
        days, request_rate, tool_rate, cache_rate, p50_e2e, p95_e2e
    );

    ToolResult {
        tool_name: "metrics_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "days": days,
            "request_success_rate": request_rate,
            "request_total": request_total,
            "tool_success_rate": tool_rate,
            "tool_total": tool_total,
            "cache_hit_rate": cache_rate,
            "cache_total": cache_total,
            "latency_p50_e2e_ms": p50_e2e,
            "latency_p95_e2e_ms": p95_e2e,
            "totals": totals,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_error_budgets(evidence_id: &str, timestamp: u64) -> ToolResult {
    let metrics = MetricsStore::load();
    let budgets = ErrorBudgets::default();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_metrics = metrics
        .for_date(&today)
        .cloned()
        .unwrap_or_else(|| crate::reliability::DailyMetrics::default());

    let statuses = calculate_budget_status(&today_metrics, &budgets);
    let alerts = check_budget_alerts(&statuses);

    let has_issues = statuses.iter().any(|s| {
        s.status == crate::reliability::BudgetState::Warning
            || s.status == crate::reliability::BudgetState::Critical
            || s.status == crate::reliability::BudgetState::Exhausted
    });

    let human_summary = if statuses.is_empty() {
        "No error budget data yet today".to_string()
    } else if has_issues {
        let issues: Vec<String> = statuses
            .iter()
            .filter(|s| s.status != crate::reliability::BudgetState::Ok)
            .map(|s| {
                format!(
                    "{}: {:.1}%/{:.1}%",
                    s.category, s.current_percent, s.budget_percent
                )
            })
            .collect();
        format!("Error budget issues: {}", issues.join(", "))
    } else {
        format!("All {} error budgets healthy", statuses.len())
    };

    ToolResult {
        tool_name: "error_budgets".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "date": today,
            "budgets": statuses.iter().map(|s| json!({
                "category": s.category,
                "budget_percent": s.budget_percent,
                "current_percent": s.current_percent,
                "total_events": s.total_events,
                "failed_events": s.failed_events,
                "status": s.status.as_str(),
            })).collect::<Vec<_>>(),
            "alerts": alerts.iter().map(|a| json!({
                "category": a.category,
                "severity": format!("{:?}", a.severity),
                "message": a.message,
            })).collect::<Vec<_>>(),
            "has_issues": has_issues,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

// =============================================================================
// v0.0.33: Case file retrieval tools
// =============================================================================

fn execute_last_case_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let cases = crate::transcript::list_recent_cases(1);

    if cases.is_empty() {
        return ToolResult {
            tool_name: "last_case_summary".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "No case files found"}),
            human_summary: "No case files found yet".to_string(),
            success: true,
            error: None,
            timestamp,
        };
    }

    let case_path = &cases[0];
    let summary = crate::transcript::load_case_summary(case_path);

    match summary {
        Some(s) => {
            let human_summary = format!(
                "Last case [{}]: {} - {} ({}% reliability)",
                s.request_id,
                truncate_str(&s.user_request, 50),
                s.outcome,
                s.reliability_score
            );

            ToolResult {
                tool_name: "last_case_summary".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "request_id": s.request_id,
                    "timestamp": s.timestamp.to_rfc3339(),
                    "user_request": s.user_request,
                    "intent_type": s.intent_type,
                    "outcome": s.outcome.to_string(),
                    "reliability_score": s.reliability_score,
                    "evidence_count": s.evidence_count,
                    "policy_refs_count": s.policy_refs_count,
                    "duration_ms": s.duration_ms,
                    "error_message": s.error_message,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        None => ToolResult {
            tool_name: "last_case_summary".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"error": "Could not load case summary"}),
            human_summary: "Could not read last case file".to_string(),
            success: false,
            error: Some("Failed to parse case summary".to_string()),
            timestamp,
        },
    }
}

fn execute_last_failure_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let failure_path = crate::transcript::find_last_failure();

    match failure_path {
        Some(path) => {
            let summary = crate::transcript::load_case_summary(&path);
            let transcript_path = path.join("transcript.log");
            let transcript = std::fs::read_to_string(&transcript_path)
                .unwrap_or_else(|_| "(transcript not available)".to_string());

            if let Some(s) = summary {
                let human_summary = format!(
                    "Last failure [{}]: {} - error: {}",
                    s.request_id,
                    truncate_str(&s.user_request, 40),
                    s.error_message.as_deref().unwrap_or("unknown")
                );

                ToolResult {
                    tool_name: "last_failure_summary".to_string(),
                    evidence_id: evidence_id.to_string(),
                    data: json!({
                        "request_id": s.request_id,
                        "timestamp": s.timestamp.to_rfc3339(),
                        "user_request": s.user_request,
                        "outcome": s.outcome.to_string(),
                        "reliability_score": s.reliability_score,
                        "error_message": s.error_message,
                        "transcript_preview": truncate_str(&transcript, 500),
                    }),
                    human_summary,
                    success: true,
                    error: None,
                    timestamp,
                }
            } else {
                ToolResult {
                    tool_name: "last_failure_summary".to_string(),
                    evidence_id: evidence_id.to_string(),
                    data: json!({"error": "Could not load failure details"}),
                    human_summary: "Could not read failure case file".to_string(),
                    success: false,
                    error: Some("Failed to parse failure case".to_string()),
                    timestamp,
                }
            }
        }
        None => ToolResult {
            tool_name: "last_failure_summary".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({"message": "No failures found"}),
            human_summary: "No failures found in recent cases".to_string(),
            success: true,
            error: None,
            timestamp,
        },
    }
}

fn execute_list_today_cases(evidence_id: &str, timestamp: u64) -> ToolResult {
    let cases = crate::transcript::list_today_cases();

    let case_summaries: Vec<_> = cases
        .iter()
        .filter_map(|path| crate::transcript::load_case_summary(path))
        .collect();

    let human_summary = format!(
        "Today: {} cases, {} successful",
        case_summaries.len(),
        case_summaries
            .iter()
            .filter(|s| s.outcome == crate::transcript::CaseOutcome::Success)
            .count()
    );

    ToolResult {
        tool_name: "list_today_cases".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "count": case_summaries.len(),
            "cases": case_summaries.iter().map(|s| json!({
                "request_id": s.request_id,
                "timestamp": s.timestamp.to_rfc3339(),
                "request_preview": truncate_str(&s.user_request, 30),
                "outcome": s.outcome.to_string(),
                "reliability_score": s.reliability_score,
            })).collect::<Vec<_>>(),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_list_recent_cases(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let cases = crate::transcript::list_recent_cases(limit);

    let case_summaries: Vec<_> = cases
        .iter()
        .filter_map(|path| crate::transcript::load_case_summary(path))
        .collect();

    let human_summary = format!(
        "Recent: {} cases (last {} requested)",
        case_summaries.len(),
        limit
    );

    ToolResult {
        tool_name: "list_recent_cases".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "limit": limit,
            "count": case_summaries.len(),
            "cases": case_summaries.iter().map(|s| json!({
                "request_id": s.request_id,
                "timestamp": s.timestamp.to_rfc3339(),
                "request_preview": truncate_str(&s.user_request, 30),
                "outcome": s.outcome.to_string(),
                "reliability_score": s.reliability_score,
            })).collect::<Vec<_>>(),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Helper to truncate strings for summaries
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// =============================================================================
// v0.0.45: Direct evidence tools for correctness
// =============================================================================

fn execute_kernel_version(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Get kernel version from uname
    let kernel_release = Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let kernel_full = Command::new("uname")
        .arg("-a")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let human_summary = format!("Kernel version: {}", kernel_release);

    ToolResult {
        tool_name: "kernel_version".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "kernel_release": kernel_release,
            "kernel_full": kernel_full,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_memory_info(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Read /proc/meminfo directly
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();

    // Parse key values
    let mut total_kb: u64 = 0;
    let mut free_kb: u64 = 0;
    let mut available_kb: u64 = 0;
    let mut buffers_kb: u64 = 0;
    let mut cached_kb: u64 = 0;
    let mut swap_total_kb: u64 = 0;
    let mut swap_free_kb: u64 = 0;

    for line in meminfo.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let value = parts[1].parse::<u64>().unwrap_or(0);
            match parts[0] {
                "MemTotal:" => total_kb = value,
                "MemFree:" => free_kb = value,
                "MemAvailable:" => available_kb = value,
                "Buffers:" => buffers_kb = value,
                "Cached:" => cached_kb = value,
                "SwapTotal:" => swap_total_kb = value,
                "SwapFree:" => swap_free_kb = value,
                _ => {}
            }
        }
    }

    // Convert to GiB for display
    let total_gib = total_kb as f64 / 1024.0 / 1024.0;
    let available_gib = available_kb as f64 / 1024.0 / 1024.0;
    let used_gib = (total_kb - available_kb) as f64 / 1024.0 / 1024.0;

    let human_summary = format!(
        "Memory: {:.1} GiB total, {:.1} GiB available, {:.1} GiB used",
        total_gib, available_gib, used_gib
    );

    ToolResult {
        tool_name: "memory_info".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "total_kb": total_kb,
            "free_kb": free_kb,
            "available_kb": available_kb,
            "buffers_kb": buffers_kb,
            "cached_kb": cached_kb,
            "swap_total_kb": swap_total_kb,
            "swap_free_kb": swap_free_kb,
            "total_gib": format!("{:.2}", total_gib),
            "available_gib": format!("{:.2}", available_gib),
            "used_gib": format!("{:.2}", used_gib),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_network_status(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Get interface states with ip link
    let ip_link = Command::new("ip")
        .args(["-o", "link"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Get IP addresses with ip addr
    let ip_addr = Command::new("ip")
        .args(["-o", "addr"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Get default route
    let default_route = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "no default route".to_string());

    // Get DNS servers from /etc/resolv.conf
    let dns_servers: Vec<String> = std::fs::read_to_string("/etc/resolv.conf")
        .unwrap_or_default()
        .lines()
        .filter(|l| l.starts_with("nameserver"))
        .filter_map(|l| l.split_whitespace().nth(1).map(|s| s.to_string()))
        .collect();

    // Check NetworkManager status
    let nm_status = Command::new("systemctl")
        .args(["is-active", "NetworkManager"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Check systemd-networkd status
    let networkd_status = Command::new("systemctl")
        .args(["is-active", "systemd-networkd"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Parse interfaces from ip link
    let interfaces: Vec<serde_json::Value> = ip_link
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let name = parts[1].trim();
                let state = if line.contains("state UP") {
                    "UP"
                } else if line.contains("state DOWN") {
                    "DOWN"
                } else {
                    "UNKNOWN"
                };
                Some(json!({ "name": name, "state": state }))
            } else {
                None
            }
        })
        .collect();

    let active_count = interfaces
        .iter()
        .filter(|i| i.get("state").and_then(|v| v.as_str()) == Some("UP"))
        .count();

    let human_summary = format!(
        "Network: {} interfaces ({} up), default route: {}, NetworkManager: {}",
        interfaces.len(),
        active_count,
        if default_route.is_empty() {
            "none"
        } else {
            "yes"
        },
        nm_status
    );

    ToolResult {
        tool_name: "network_status".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "interfaces": interfaces,
            "default_route": default_route,
            "dns_servers": dns_servers,
            "networkmanager_status": nm_status,
            "systemd_networkd_status": networkd_status,
            "ip_link_raw": ip_link.lines().take(10).collect::<Vec<_>>(),
            "ip_addr_raw": ip_addr.lines().take(10).collect::<Vec<_>>(),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

fn execute_audio_status(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Check pipewire status
    let pipewire_status = Command::new("systemctl")
        .args(["--user", "is-active", "pipewire"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Check wireplumber status
    let wireplumber_status = Command::new("systemctl")
        .args(["--user", "is-active", "wireplumber"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Check pulseaudio status (fallback)
    let pulseaudio_status = Command::new("systemctl")
        .args(["--user", "is-active", "pulseaudio"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get audio devices with pactl
    let pactl_sinks = Command::new("pactl")
        .args(["list", "short", "sinks"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let pactl_sources = Command::new("pactl")
        .args(["list", "short", "sources"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Get ALSA cards
    let alsa_cards = std::fs::read_to_string("/proc/asound/cards")
        .unwrap_or_else(|_| "No ALSA cards found".to_string());

    // Parse sinks and sources
    let sinks: Vec<String> = pactl_sinks
        .lines()
        .filter_map(|l| l.split_whitespace().nth(1).map(|s| s.to_string()))
        .collect();

    let sources: Vec<String> = pactl_sources
        .lines()
        .filter_map(|l| l.split_whitespace().nth(1).map(|s| s.to_string()))
        .collect();

    let audio_backend = if pipewire_status == "active" {
        "pipewire"
    } else if pulseaudio_status == "active" {
        "pulseaudio"
    } else {
        "none"
    };

    let human_summary = format!(
        "Audio: {} (pipewire: {}, wireplumber: {}), {} sinks, {} sources",
        audio_backend,
        pipewire_status,
        wireplumber_status,
        sinks.len(),
        sources.len()
    );

    ToolResult {
        tool_name: "audio_status".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "pipewire_status": pipewire_status,
            "wireplumber_status": wireplumber_status,
            "pulseaudio_status": pulseaudio_status,
            "audio_backend": audio_backend,
            "sinks": sinks,
            "sources": sources,
            "sink_count": sinks.len(),
            "source_count": sources.len(),
            "alsa_cards": alsa_cards.lines().take(5).collect::<Vec<_>>(),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

// =============================================================================
// v0.0.46: Domain-Specific Evidence Tools
// These tools provide targeted evidence for specific question domains
// =============================================================================

/// uname_summary - kernel version and architecture
fn execute_uname_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let kernel_release = Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let machine = Command::new("uname")
        .arg("-m")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let kernel_name = Command::new("uname")
        .arg("-s")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let human_summary = format!("Kernel: {} {} ({})", kernel_name, kernel_release, machine);

    ToolResult {
        tool_name: "uname_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "kernel_release": kernel_release,
            "kernel_name": kernel_name,
            "machine": machine,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// mem_summary - memory total and available from /proc/meminfo
fn execute_mem_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();

    let mut mem_total_kb: u64 = 0;
    let mut mem_available_kb: u64 = 0;

    for line in meminfo.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let value = parts[1].parse::<u64>().unwrap_or(0);
            match parts[0] {
                "MemTotal:" => mem_total_kb = value,
                "MemAvailable:" => mem_available_kb = value,
                _ => {}
            }
        }
    }

    let total_gib = mem_total_kb as f64 / 1024.0 / 1024.0;
    let available_gib = mem_available_kb as f64 / 1024.0 / 1024.0;
    let used_gib = total_gib - available_gib;
    let used_percent = if total_gib > 0.0 {
        (used_gib / total_gib) * 100.0
    } else {
        0.0
    };

    let human_summary = format!(
        "Memory: {:.1} GiB total, {:.1} GiB available ({:.0}% used)",
        total_gib, available_gib, used_percent
    );

    ToolResult {
        tool_name: "mem_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "mem_total_kb": mem_total_kb,
            "mem_available_kb": mem_available_kb,
            "mem_total_gib": format!("{:.2}", total_gib),
            "mem_available_gib": format!("{:.2}", available_gib),
            "mem_used_gib": format!("{:.2}", used_gib),
            "mem_used_percent": format!("{:.1}", used_percent),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// mount_usage - disk space for / and key mounts
fn execute_mount_usage(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Use df with block output for precise bytes
    let df_output = Command::new("df")
        .args(["-B1", "--output=source,size,used,avail,pcent,target"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut mounts: Vec<serde_json::Value> = Vec::new();
    let mut root_info: Option<serde_json::Value> = None;

    for line in df_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let source = parts[0];
            let size_bytes: u64 = parts[1].parse().unwrap_or(0);
            let used_bytes: u64 = parts[2].parse().unwrap_or(0);
            let avail_bytes: u64 = parts[3].parse().unwrap_or(0);
            let use_pct = parts[4].trim_end_matches('%');
            let target = parts[5];

            // Format human-readable sizes
            let size_human = format_bytes_human(size_bytes);
            let avail_human = format_bytes_human(avail_bytes);

            let mount = json!({
                "source": source,
                "target": target,
                "size_bytes": size_bytes,
                "used_bytes": used_bytes,
                "avail_bytes": avail_bytes,
                "use_percent": use_pct,
                "size_human": size_human,
                "avail_human": avail_human,
            });

            if target == "/" {
                root_info = Some(mount.clone());
            }

            // Include key mounts only
            if target == "/" || target == "/home" || target == "/boot" || target == "/var" {
                mounts.push(mount);
            }
        }
    }

    let (root_free, root_pct) = if let Some(ref root) = root_info {
        (
            root.get("avail_human")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            root.get("use_percent")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
        )
    } else {
        ("unknown".to_string(), "0".to_string())
    };

    let human_summary = format!("Disk /: {} free ({}% used)", root_free, root_pct);

    ToolResult {
        tool_name: "mount_usage".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "root": root_info,
            "mounts": mounts,
            "root_free_human": root_free,
            "root_used_percent": root_pct,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// Format bytes to human-readable string
fn format_bytes_human(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if bytes >= TIB {
        format!("{:.1} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// nm_summary - NetworkManager status and active connections
fn execute_nm_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Check NetworkManager service state
    let nm_active = Command::new("systemctl")
        .args(["is-active", "NetworkManager"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get active connections via nmcli if available
    let nmcli_output = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "NAME,TYPE,DEVICE,STATE",
            "connection",
            "show",
            "--active",
        ])
        .output()
        .ok();

    let connections: Vec<serde_json::Value> = nmcli_output
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 4 {
                        Some(json!({
                            "name": parts[0],
                            "type": parts[1],
                            "device": parts[2],
                            "state": parts[3],
                        }))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let human_summary = format!(
        "NetworkManager: {}, {} active connections",
        nm_active,
        connections.len()
    );

    ToolResult {
        tool_name: "nm_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "networkmanager_status": nm_active,
            "active_connection_count": connections.len(),
            "connections": connections,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// ip_route_summary - default route and routing table summary
fn execute_ip_route_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Get default route
    let default_route = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // Parse gateway and interface from default route
    let (gateway, interface) = if !default_route.is_empty() {
        let parts: Vec<&str> = default_route.split_whitespace().collect();
        let gw = parts
            .iter()
            .skip_while(|&&p| p != "via")
            .nth(1)
            .map(|s| s.to_string())
            .unwrap_or_default();
        let dev = parts
            .iter()
            .skip_while(|&&p| p != "dev")
            .nth(1)
            .map(|s| s.to_string())
            .unwrap_or_default();
        (gw, dev)
    } else {
        (String::new(), String::new())
    };

    // Count total routes
    let route_count = Command::new("ip")
        .args(["route", "show"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    let has_default = !default_route.is_empty();
    let human_summary = if has_default {
        format!(
            "Default route via {} on {}, {} total routes",
            gateway, interface, route_count
        )
    } else {
        format!("No default route, {} routes", route_count)
    };

    ToolResult {
        tool_name: "ip_route_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "has_default_route": has_default,
            "default_gateway": gateway,
            "default_interface": interface,
            "default_route_raw": default_route,
            "total_routes": route_count,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// link_state_summary - interface up/down and carrier status
fn execute_link_state_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let ip_link = Command::new("ip")
        .args(["-o", "link", "show"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut interfaces: Vec<serde_json::Value> = Vec::new();
    let mut up_count = 0;
    let mut carrier_count = 0;

    for line in ip_link.lines() {
        // Parse: "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> ..."
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() >= 2 {
            let name = parts[1].trim().split('@').next().unwrap_or(parts[1].trim());
            let flags = if let Some(flags_part) = line.split('<').nth(1) {
                flags_part.split('>').next().unwrap_or("")
            } else {
                ""
            };

            let is_up = flags.contains("UP");
            let has_carrier = flags.contains("LOWER_UP");
            let state = if line.contains("state UP") {
                "UP"
            } else if line.contains("state DOWN") {
                "DOWN"
            } else {
                "UNKNOWN"
            };

            // Skip loopback for reporting
            if name != "lo" {
                if is_up {
                    up_count += 1;
                }
                if has_carrier {
                    carrier_count += 1;
                }

                interfaces.push(json!({
                    "name": name,
                    "state": state,
                    "up": is_up,
                    "carrier": has_carrier,
                }));
            }
        }
    }

    let human_summary = format!(
        "Interfaces: {} total, {} up, {} with carrier",
        interfaces.len(),
        up_count,
        carrier_count
    );

    ToolResult {
        tool_name: "link_state_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "interface_count": interfaces.len(),
            "up_count": up_count,
            "carrier_count": carrier_count,
            "interfaces": interfaces,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// audio_services_summary - pipewire and wireplumber service states
fn execute_audio_services_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Check user services
    let pipewire_user = Command::new("systemctl")
        .args(["--user", "is-active", "pipewire"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let wireplumber_user = Command::new("systemctl")
        .args(["--user", "is-active", "wireplumber"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let pipewire_pulse_user = Command::new("systemctl")
        .args(["--user", "is-active", "pipewire-pulse"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Check system pulseaudio (legacy)
    let pulseaudio = Command::new("systemctl")
        .args(["--user", "is-active", "pulseaudio"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "inactive".to_string());

    let audio_working = pipewire_user == "active" && wireplumber_user == "active";

    let human_summary = format!(
        "Audio services: pipewire={}, wireplumber={}, pipewire-pulse={}",
        pipewire_user, wireplumber_user, pipewire_pulse_user
    );

    ToolResult {
        tool_name: "audio_services_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "pipewire_status": pipewire_user,
            "wireplumber_status": wireplumber_user,
            "pipewire_pulse_status": pipewire_pulse_user,
            "pulseaudio_status": pulseaudio,
            "audio_working": audio_working,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// pactl_summary - default audio sink/source names
fn execute_pactl_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    // Check if pactl is available
    let pactl_available = Command::new("which")
        .arg("pactl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !pactl_available {
        return ToolResult {
            tool_name: "pactl_summary".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "available": false,
                "reason": "pactl not installed"
            }),
            human_summary: "pactl not available (pipewire/pulseaudio tools not installed)"
                .to_string(),
            success: true,
            error: None,
            timestamp,
        };
    }

    // Get default sink
    let default_sink = Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get default source
    let default_source = Command::new("pactl")
        .args(["get-default-source"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Count sinks and sources
    let sink_count = Command::new("pactl")
        .args(["list", "short", "sinks"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    let source_count = Command::new("pactl")
        .args(["list", "short", "sources"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    let human_summary = format!(
        "Audio: default sink={}, {} sinks, {} sources",
        default_sink.split('.').last().unwrap_or(&default_sink),
        sink_count,
        source_count
    );

    ToolResult {
        tool_name: "pactl_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "available": true,
            "default_sink": default_sink,
            "default_source": default_source,
            "sink_count": sink_count,
            "source_count": source_count,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// boot_time_summary - boot time from systemd-analyze
fn execute_boot_time_summary(evidence_id: &str, timestamp: u64) -> ToolResult {
    let analyze_output = Command::new("systemd-analyze")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "systemd-analyze not available".to_string());

    // Parse boot time from output like:
    // "Startup finished in 1.234s (kernel) + 5.678s (userspace) = 6.912s"
    let total_time = analyze_output
        .split('=')
        .last()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Extract kernel and userspace times
    let kernel_time = analyze_output
        .split('+')
        .next()
        .and_then(|s| s.split("in").last())
        .map(|s| s.trim().trim_end_matches("(kernel)").trim().to_string())
        .unwrap_or_default();

    let userspace_time = analyze_output
        .split('+')
        .nth(1)
        .map(|s| {
            s.split('=')
                .next()
                .unwrap_or(s)
                .trim()
                .trim_end_matches("(userspace)")
                .trim()
                .to_string()
        })
        .unwrap_or_default();

    let human_summary = format!("Boot time: {}", total_time);

    ToolResult {
        tool_name: "boot_time_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "total_time": total_time,
            "kernel_time": kernel_time,
            "userspace_time": userspace_time,
            "raw_output": analyze_output,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

/// recent_errors_summary - journalctl warnings/errors summarized by service
fn execute_recent_errors_summary(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let minutes = params.get("minutes").and_then(|v| v.as_u64()).unwrap_or(30) as u32;

    // Get errors and warnings from journal
    let journal_output = Command::new("journalctl")
        .args([
            "--priority=warning",
            &format!("--since={} min ago", minutes),
            "--no-pager",
            "-o",
            "short",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Count by service/unit
    let mut service_counts: HashMap<String, u32> = HashMap::new();
    let mut total_count: u32 = 0;

    for line in journal_output.lines().take(200) {
        // Limit to 200 lines
        total_count += 1;
        // Extract service name from log line
        // Format: "Dec 03 12:34:56 hostname service[pid]: message"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let service = parts[4]
                .split('[')
                .next()
                .unwrap_or("unknown")
                .trim_end_matches(':');
            *service_counts.entry(service.to_string()).or_insert(0) += 1;
        }
    }

    // Sort by count descending and take top 10
    let mut sorted: Vec<_> = service_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    let top_services: Vec<serde_json::Value> = sorted
        .iter()
        .take(10)
        .map(|(svc, count)| json!({ "service": svc, "count": count }))
        .collect();

    let human_summary = if total_count == 0 {
        format!("No errors/warnings in last {} minutes", minutes)
    } else {
        format!(
            "{} warnings/errors in last {} min from {} services",
            total_count,
            minutes,
            service_counts.len()
        )
    };

    ToolResult {
        tool_name: "recent_errors_summary".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "minutes": minutes,
            "total_count": total_count,
            "service_count": service_counts.len(),
            "top_services": top_services,
            "sample_lines": journal_output.lines().take(10).collect::<Vec<_>>(),
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

// =============================================================================
// v0.0.47: File Evidence Tools for Mutation Support
// =============================================================================

fn execute_file_stat(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let path_str = params.get("path").and_then(|v| v.as_str()).unwrap_or("");

    if path_str.is_empty() {
        return ToolResult {
            tool_name: "file_stat".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({ "error": "Missing 'path' parameter" }),
            human_summary: "Error: missing path parameter".to_string(),
            success: false,
            error: Some("Missing 'path' parameter".to_string()),
            timestamp,
        };
    }

    let path = std::path::Path::new(path_str);
    let exists = path.exists();

    if !exists {
        return ToolResult {
            tool_name: "file_stat".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "path": path_str,
                "exists": false,
            }),
            human_summary: format!("File does not exist: {}", path_str),
            success: true,
            error: None,
            timestamp,
        };
    }

    match std::fs::metadata(path) {
        Ok(meta) => {
            use std::os::unix::fs::MetadataExt;
            let uid = meta.uid();
            let gid = meta.gid();
            let mode = meta.mode();
            let size = meta.len();
            let mtime = meta.mtime();
            let is_file = meta.is_file();
            let is_dir = meta.is_dir();

            let human_summary = format!(
                "{}: {}bytes, uid={}, gid={}, mode={:o}",
                path_str,
                size,
                uid,
                gid,
                mode & 0o7777
            );

            ToolResult {
                tool_name: "file_stat".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "path": path_str,
                    "exists": true,
                    "uid": uid,
                    "gid": gid,
                    "mode": format!("{:o}", mode & 0o7777),
                    "mode_raw": mode,
                    "size": size,
                    "mtime": mtime,
                    "is_file": is_file,
                    "is_dir": is_dir,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "file_stat".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "path": path_str,
                "exists": true,
                "error": e.to_string(),
            }),
            human_summary: format!("Cannot stat {}: {}", path_str, e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_file_preview(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    use crate::redaction::redact_transcript;

    let path_str = params.get("path").and_then(|v| v.as_str()).unwrap_or("");

    let max_bytes = params
        .get("max_bytes")
        .and_then(|v| v.as_u64())
        .unwrap_or(2048) as usize;

    if path_str.is_empty() {
        return ToolResult {
            tool_name: "file_preview".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({ "error": "Missing 'path' parameter" }),
            human_summary: "Error: missing path parameter".to_string(),
            success: false,
            error: Some("Missing 'path' parameter".to_string()),
            timestamp,
        };
    }

    let path = std::path::Path::new(path_str);

    if !path.exists() {
        return ToolResult {
            tool_name: "file_preview".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "path": path_str,
                "exists": false,
            }),
            human_summary: format!("File does not exist: {}", path_str),
            success: true,
            error: None,
            timestamp,
        };
    }

    match std::fs::read(path) {
        Ok(bytes) => {
            let truncated = bytes.len() > max_bytes;
            let preview_bytes = if truncated {
                &bytes[..max_bytes]
            } else {
                &bytes
            };

            // Convert to string, handling non-UTF8
            let content = String::from_utf8_lossy(preview_bytes);

            // Apply secrets redaction
            let redacted = redact_transcript(&content);

            // Get last N lines for diff context
            let lines: Vec<&str> = redacted.lines().collect();
            let line_count = lines.len();
            let last_20_lines: Vec<&str> = lines.iter().rev().take(20).rev().copied().collect();

            let human_summary = format!(
                "{}: {} bytes, {} lines{}",
                path_str,
                bytes.len(),
                line_count,
                if truncated { " (truncated)" } else { "" }
            );

            ToolResult {
                tool_name: "file_preview".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "path": path_str,
                    "exists": true,
                    "total_bytes": bytes.len(),
                    "preview_bytes": preview_bytes.len(),
                    "truncated": truncated,
                    "line_count": line_count,
                    "last_20_lines": last_20_lines,
                    "content_redacted": redacted,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "file_preview".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "path": path_str,
                "error": e.to_string(),
            }),
            human_summary: format!("Cannot read {}: {}", path_str, e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_file_hash(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    let path_str = params.get("path").and_then(|v| v.as_str()).unwrap_or("");

    if path_str.is_empty() {
        return ToolResult {
            tool_name: "file_hash".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({ "error": "Missing 'path' parameter" }),
            human_summary: "Error: missing path parameter".to_string(),
            success: false,
            error: Some("Missing 'path' parameter".to_string()),
            timestamp,
        };
    }

    let path = std::path::Path::new(path_str);

    if !path.exists() {
        return ToolResult {
            tool_name: "file_hash".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "path": path_str,
                "exists": false,
            }),
            human_summary: format!("File does not exist: {}", path_str),
            success: true,
            error: None,
            timestamp,
        };
    }

    match std::fs::read(path) {
        Ok(bytes) => {
            // Use a simple hash for now (same as RollbackManager)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            bytes.hash(&mut hasher);
            let hash = format!("{:016x}", hasher.finish());

            let human_summary = format!("{}: hash={}", path_str, &hash[..12]);

            ToolResult {
                tool_name: "file_hash".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "path": path_str,
                    "exists": true,
                    "hash": hash,
                    "size": bytes.len(),
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "file_hash".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({
                "path": path_str,
                "error": e.to_string(),
            }),
            human_summary: format!("Cannot hash {}: {}", path_str, e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_path_policy_check(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    use crate::mutation_tools::check_path_policy;

    let path_str = params.get("path").and_then(|v| v.as_str()).unwrap_or("");

    if path_str.is_empty() {
        return ToolResult {
            tool_name: "path_policy_check".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({ "error": "Missing 'path' parameter" }),
            human_summary: "Error: missing path parameter".to_string(),
            success: false,
            error: Some("Missing 'path' parameter".to_string()),
            timestamp,
        };
    }

    let path = std::path::Path::new(path_str);
    let policy_result = check_path_policy(path);

    // Also check dev sandbox allowlist (v0.0.47)
    let cwd = std::env::current_dir().ok();
    let in_cwd = cwd.as_ref().map(|c| path.starts_with(c)).unwrap_or(false);
    let in_tmp = path.starts_with("/tmp");
    let in_sandbox = in_cwd || in_tmp;

    let human_summary = if policy_result.allowed {
        format!(
            "{}: ALLOWED [{}] - {}",
            path_str, policy_result.evidence_id, policy_result.reason
        )
    } else {
        format!(
            "{}: BLOCKED [{}] - {}",
            path_str, policy_result.evidence_id, policy_result.reason
        )
    };

    ToolResult {
        tool_name: "path_policy_check".to_string(),
        evidence_id: evidence_id.to_string(),
        data: json!({
            "path": path_str,
            "allowed": policy_result.allowed,
            "in_sandbox": in_sandbox,
            "in_cwd": in_cwd,
            "in_tmp": in_tmp,
            "reason": policy_result.reason,
            "policy_evidence_id": policy_result.evidence_id,
            "policy_rule": policy_result.policy_rule,
        }),
        human_summary,
        success: true,
        error: None,
        timestamp,
    }
}

// =============================================================================
// v0.0.48: Knowledge Search Tools
// =============================================================================

fn execute_learned_recipe_search(
    params: &HashMap<String, serde_json::Value>,
    evidence_id: &str,
    timestamp: u64,
) -> ToolResult {
    use crate::learning::LearningManager;

    let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

    if query.is_empty() {
        return ToolResult {
            tool_name: "learned_recipe_search".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({ "error": "Missing 'query' parameter" }),
            human_summary: "Error: missing query parameter".to_string(),
            success: false,
            error: Some("Missing 'query' parameter".to_string()),
            timestamp,
        };
    }

    match LearningManager::search_all(query, limit) {
        Ok(hits) => {
            let results: Vec<serde_json::Value> = hits
                .iter()
                .map(|h| {
                    json!({
                        "evidence_id": h.evidence_id,
                        "recipe_id": h.recipe.recipe_id,
                        "title": h.recipe.title,
                        "pack_id": h.pack_id,
                        "pack_name": h.pack_name,
                        "triggers": h.recipe.triggers,
                        "targets": h.recipe.targets,
                        "score": h.score,
                        "wins": h.recipe.wins,
                        "summary": h.summary(),
                        "match_reason": h.match_reason(),
                    })
                })
                .collect();

            let human_summary = if hits.is_empty() {
                format!("No matching recipes found for: {}", query)
            } else {
                let top_titles: Vec<&str> = hits
                    .iter()
                    .take(3)
                    .map(|h| h.recipe.title.as_str())
                    .collect();
                format!("Found {} recipes: {}", hits.len(), top_titles.join(", "))
            };

            ToolResult {
                tool_name: "learned_recipe_search".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "query": query,
                    "result_count": hits.len(),
                    "results": results,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "learned_recipe_search".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({ "error": e.to_string() }),
            human_summary: format!("Search failed: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

fn execute_learning_stats(evidence_id: &str, timestamp: u64) -> ToolResult {
    use crate::learning::LearningManager;

    match LearningManager::get_stats() {
        Ok(stats) => {
            let human_summary = format!(
                "Level {} {} | XP: {}/{} | {} recipes in {} packs",
                stats.xp_summary.level,
                stats.xp_summary.title,
                stats.xp_summary.current_xp,
                stats.xp_summary.next_level_xp,
                stats.recipe_count,
                stats.pack_count
            );

            ToolResult {
                tool_name: "learning_stats".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "level": stats.xp_summary.level,
                    "title": stats.xp_summary.title,
                    "current_xp": stats.xp_summary.current_xp,
                    "next_level_xp": stats.xp_summary.next_level_xp,
                    "successful_answers": stats.xp_summary.successful_answers,
                    "recipes_created": stats.xp_summary.recipes_created,
                    "recipes_improved": stats.xp_summary.recipes_improved,
                    "pack_count": stats.pack_count,
                    "recipe_count": stats.recipe_count,
                    "max_packs": stats.max_packs,
                    "max_recipes": stats.max_recipes,
                }),
                human_summary,
                success: true,
                error: None,
                timestamp,
            }
        }
        Err(e) => ToolResult {
            tool_name: "learning_stats".to_string(),
            evidence_id: evidence_id.to_string(),
            data: json!({ "error": e.to_string() }),
            human_summary: format!("Stats failed: {}", e),
            success: false,
            error: Some(e.to_string()),
            timestamp,
        },
    }
}

// ============================================================================
// v0.0.58: Proactive Alerts Tool Executors
// ============================================================================

fn execute_proactive_alerts_summary(evidence_id: &str, _timestamp: u64) -> ToolResult {
    probe_alerts_summary(evidence_id)
}

fn execute_disk_pressure_summary(evidence_id: &str, _timestamp: u64) -> ToolResult {
    probe_disk_pressure_summary(evidence_id)
}

fn execute_failed_units_summary(evidence_id: &str, _timestamp: u64) -> ToolResult {
    probe_failed_units_summary(evidence_id)
}

fn execute_thermal_status_summary(evidence_id: &str, _timestamp: u64) -> ToolResult {
    probe_thermal_summary(evidence_id)
}

fn execute_journal_error_burst_summary(evidence_id: &str, _timestamp: u64) -> ToolResult {
    probe_journal_error_burst_summary(evidence_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_unknown_tool() {
        let catalog = ToolCatalog::new();
        let request = ToolRequest {
            tool_name: "nonexistent_tool".to_string(),
            parameters: HashMap::new(),
        };
        let result = execute_tool(&request, &catalog, "E1");
        assert!(!result.success);
        assert!(result.human_summary.contains("Unknown tool"));
    }

    #[test]
    fn test_execute_tool_plan() {
        let catalog = ToolCatalog::new();
        let mut collector = EvidenceCollector::new();

        let requests = vec![
            ToolRequest {
                tool_name: "status_snapshot".to_string(),
                parameters: HashMap::new(),
            },
            ToolRequest {
                tool_name: "hw_snapshot_summary".to_string(),
                parameters: HashMap::new(),
            },
        ];

        let results = execute_tool_plan(&requests, &catalog, &mut collector);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].evidence_id, "E1");
        assert_eq!(results[1].evidence_id, "E2");
        assert_eq!(collector.all().len(), 2);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90061), "1d 1h");
    }
}
