//! Tool Executor v0.0.12
//!
//! Executes read-only tools from the catalog with structured outputs
//! and human-readable summaries.

use crate::daemon_state::StatusSnapshot;
use crate::snapshots::{SwSnapshot, HwSnapshot};
use crate::tools::{ToolCatalog, ToolResult, ToolRequest, EvidenceCollector, unknown_tool_result, unavailable_result};
use crate::anomaly_engine::{AlertQueue, what_changed, analyze_slowness};
use serde_json::json;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Execute a tool and return the result with evidence ID
pub fn execute_tool(
    request: &ToolRequest,
    catalog: &ToolCatalog,
    evidence_id: &str,
) -> ToolResult {
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
        "top_resource_processes" => execute_top_resource_processes(&request.parameters, evidence_id, timestamp),
        "package_info" => execute_package_info(&request.parameters, evidence_id, timestamp),
        "service_status" => execute_service_status(&request.parameters, evidence_id, timestamp),
        "disk_usage" => execute_disk_usage(evidence_id, timestamp),
        // v0.0.12: Anomaly detection tools
        "active_alerts" => execute_active_alerts(evidence_id, timestamp),
        "what_changed" => execute_what_changed(&request.parameters, evidence_id, timestamp),
        "slowness_hypotheses" => execute_slowness_hypotheses(&request.parameters, evidence_id, timestamp),
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

            let llm_status = snapshot.llm_bootstrap_phase
                .as_deref()
                .unwrap_or("unknown");

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
            human_summary: "Status snapshot is not available (daemon may not be running).".to_string(),
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
            let categories: HashMap<String, usize> = snapshot.categories.iter()
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
            let cpu_info = format!("{} ({} cores, {} threads)",
                snapshot.cpu.model, snapshot.cpu.cores, snapshot.cpu.threads);

            let memory_gb = snapshot.memory.total_bytes / (1024 * 1024 * 1024);

            let gpu_info = snapshot.gpu.as_ref()
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

fn execute_recent_installs(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let days = params.get("days")
        .and_then(|v| v.as_i64())
        .unwrap_or(14) as i64;

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

fn execute_journal_warnings(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let service = params.get("service")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let minutes = params.get("minutes")
        .and_then(|v| v.as_i64())
        .unwrap_or(60) as u64;

    // Run journalctl with priority filter
    let mut cmd = Command::new("journalctl");
    cmd.arg("--no-pager")
        .arg("-p").arg("warning")
        .arg("--since").arg(format!("{} minutes ago", minutes))
        .arg("-n").arg("50"); // Limit entries

    if let Some(svc) = &service {
        cmd.arg("-u").arg(svc);
    }

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().take(30).collect();
            let total_lines = stdout.lines().count();

            let human_summary = if let Some(svc) = &service {
                format!("Found {} warnings/errors for {} in the last {} minutes", total_lines, svc, minutes)
            } else {
                format!("Found {} system warnings/errors in the last {} minutes", total_lines, minutes)
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

fn execute_boot_time_trend(_params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
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

fn execute_top_resource_processes(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    // Use ps to get top CPU/memory consumers
    match Command::new("ps")
        .args(["aux", "--sort=-%cpu"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().take(11).collect(); // Header + top 10

            let process_count = lines.len().saturating_sub(1);
            let human_summary = format!("Retrieved top {} resource-consuming processes", process_count);

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

fn execute_package_info(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let name = params.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

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

            let version = info.get("version").cloned().unwrap_or_else(|| "unknown".to_string());
            let installed_size = info.get("installed_size").cloned().unwrap_or_else(|| "unknown".to_string());

            let human_summary = format!("Package {}: version {}, size {}", name, version, installed_size);

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

fn execute_service_status(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let name = params.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

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

    match Command::new("systemctl").args(["status", &unit_name]).output() {
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
    match Command::new("df").args(["-h", "--output=source,size,used,avail,pcent,target"]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            // Find root filesystem usage
            let root_usage = lines.iter()
                .find(|l| l.ends_with(" /"))
                .map(|l| l.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let human_summary = format!("Disk usage: {}", root_usage.split_whitespace().nth(4).unwrap_or("unknown"));

            ToolResult {
                tool_name: "disk_usage".to_string(),
                evidence_id: evidence_id.to_string(),
                data: json!({
                    "filesystems": lines,
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
            active.len(), critical, warning, info
        )
    };

    let alerts_data: Vec<serde_json::Value> = active.iter().map(|a| {
        json!({
            "evidence_id": a.evidence_id,
            "severity": a.severity.as_str(),
            "title": a.title,
            "description": a.description,
            "confidence": a.confidence,
            "occurrence_count": a.occurrence_count,
            "hints": a.hints,
        })
    }).collect();

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

fn execute_what_changed(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let days = params.get("days")
        .and_then(|v| v.as_i64())
        .unwrap_or(7) as u32;

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

fn execute_slowness_hypotheses(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let days = params.get("days")
        .and_then(|v| v.as_i64())
        .unwrap_or(7) as u32;

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

    let hypotheses_data: Vec<serde_json::Value> = result.hypotheses.iter().map(|h| {
        json!({
            "evidence_id": h.evidence_id,
            "title": h.title,
            "explanation": h.explanation,
            "confidence": h.confidence,
            "supporting_evidence": h.supporting_evidence,
            "suggested_diagnostics": h.suggested_diagnostics,
        })
    }).collect();

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
