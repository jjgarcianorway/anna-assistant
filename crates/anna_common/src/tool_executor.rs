//! Tool Executor v0.0.22
//!
//! Executes read-only tools from the catalog with structured outputs
//! and human-readable summaries.

use crate::daemon_state::StatusSnapshot;
use crate::snapshots::{SwSnapshot, HwSnapshot};
use crate::tools::{ToolCatalog, ToolResult, ToolRequest, EvidenceCollector, unknown_tool_result, unavailable_result};
use crate::anomaly_engine::{AlertQueue, what_changed, analyze_slowness};
use crate::knowledge_packs::{KnowledgeIndex, DEFAULT_TOP_K};
use crate::source_labels::{AnswerContext, SourcePlan, QaStats, classify_question_type};
use crate::reliability::{DiagnosticsReport, MetricsStore, ErrorBudgets, calculate_budget_status, check_budget_alerts};
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

// v0.0.19: Knowledge pack tool implementations

fn execute_knowledge_search(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let query = params.get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("");

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

    let top_k = params.get("top_k")
        .and_then(|v| v.as_i64())
        .unwrap_or(DEFAULT_TOP_K as i64) as usize;

    // Open the knowledge index
    match KnowledgeIndex::open() {
        Ok(index) => {
            match index.search(query, top_k) {
                Ok(results) => {
                    let result_count = results.len();

                    // Format results for JSON output
                    let results_data: Vec<serde_json::Value> = results.iter().map(|r| {
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
                    }).collect();

                    let human_summary = if result_count == 0 {
                        format!("No results found for query: '{}'", query)
                    } else {
                        let top_titles: Vec<&str> = results.iter()
                            .take(3)
                            .map(|r| r.title.as_str())
                            .collect();
                        format!(
                            "Found {} results for '{}'. Top matches: {}",
                            result_count, query, top_titles.join(", ")
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
                        format!("{:.1} MB", stats.total_size_bytes as f64 / (1024.0 * 1024.0))
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
            success: true,  // Not a failure, just empty
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
        context.distro,
        context.knowledge_packs_available,
        context.knowledge_docs_count
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

fn execute_source_plan(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let request = params.get("request")
        .and_then(|v| v.as_str())
        .unwrap_or("");

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

    let top_sources: Vec<String> = stats.top_source_types(3)
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
            if top_sources.is_empty() { "none".to_string() } else { top_sources.join(", ") }
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

fn execute_metrics_summary(params: &HashMap<String, serde_json::Value>, evidence_id: &str, timestamp: u64) -> ToolResult {
    let days = params.get("days")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

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
    let (p50_e2e, p95_e2e) = metrics.for_date(&today)
        .map(|m| (
            m.percentile_latency("e2e", 50.0).unwrap_or(0),
            m.percentile_latency("e2e", 95.0).unwrap_or(0)
        ))
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
    let today_metrics = metrics.for_date(&today)
        .cloned()
        .unwrap_or_else(|| crate::reliability::DailyMetrics::default());

    let statuses = calculate_budget_status(&today_metrics, &budgets);
    let alerts = check_budget_alerts(&statuses);

    let has_issues = statuses.iter().any(|s| {
        s.status == crate::reliability::BudgetState::Warning ||
        s.status == crate::reliability::BudgetState::Critical ||
        s.status == crate::reliability::BudgetState::Exhausted
    });

    let human_summary = if statuses.is_empty() {
        "No error budget data yet today".to_string()
    } else if has_issues {
        let issues: Vec<String> = statuses.iter()
            .filter(|s| s.status != crate::reliability::BudgetState::Ok)
            .map(|s| format!("{}: {:.1}%/{:.1}%", s.category, s.current_percent, s.budget_percent))
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
