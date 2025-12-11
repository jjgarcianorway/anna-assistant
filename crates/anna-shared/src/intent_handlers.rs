//! Intent Handlers (v0.0.417).
//!
//! Explicit, deterministic rules for each intent.
//! NO generic tutorials. NO hallucination. DIRECT answers only.
//!
//! Each handler:
//! - Defines required probes
//! - Defines exact transformation from probe data to answer
//! - Returns structured response or explicit failure

use crate::strict_contract::{StrictSpecialistResponse, StrictStatus, EvidenceItem};
use std::collections::HashMap;
use serde_json::json;

/// Intent handler result
pub enum HandlerResult {
    /// Successfully handled - return this response
    Success(StrictSpecialistResponse),
    /// Missing required probe - specify which one
    MissingProbe { probe_name: String, reason: String },
    /// Cannot handle deterministically - fall back to LLM
    NeedsSpecialist { reason: String },
}

/// Handle check_free_ram intent
pub fn handle_check_free_ram(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    // Required: memory_info probe
    let mem_info = match probes.get("memory_info") {
        Some(m) if !m.trim().is_empty() => m,
        _ => return HandlerResult::MissingProbe {
            probe_name: "memory_info".to_string(),
            reason: "Need /proc/meminfo or 'free -h' output to check RAM".to_string(),
        },
    };

    // Parse MemTotal and MemAvailable
    let mem_total_kb = extract_meminfo_value(mem_info, "MemTotal");
    let mem_available_kb = extract_meminfo_value(mem_info, "MemAvailable");

    match (mem_total_kb, mem_available_kb) {
        (Some(total), Some(available)) => {
            let total_gb = total as f64 / 1_048_576.0;
            let available_gb = available as f64 / 1_048_576.0;
            let used_percent = ((total - available) as f64 / total as f64 * 100.0) as u32;

            let summary = format!(
                "Available memory: {:.1} GiB out of {:.1} GiB ({}% used)",
                available_gb, total_gb, used_percent
            );

            HandlerResult::Success(
                StrictSpecialistResponse::ok(ticket_id, "check_free_ram", &summary, 0.95)
                    .with_evidence("memory_info", &format!("MemTotal: {} kB, MemAvailable: {} kB", total, available))
                    .with_metrics(json!({
                        "mem_total_gb": format!("{:.1}", total_gb),
                        "mem_available_gb": format!("{:.1}", available_gb),
                        "mem_used_percent": used_percent
                    }))
            )
        }
        _ => HandlerResult::MissingProbe {
            probe_name: "memory_info".to_string(),
            reason: "Could not parse MemTotal/MemAvailable from memory_info probe".to_string(),
        },
    }
}

/// Handle check_swap_presence intent
pub fn handle_check_swap_presence(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    // Check swap_files probe first
    let swap_files = probes.get("swap_files").map(|s| s.as_str()).unwrap_or("");

    // Also check memory_info for SwapTotal
    let mem_info = probes.get("memory_info").map(|s| s.as_str()).unwrap_or("");
    let swap_total_kb = extract_meminfo_value(mem_info, "SwapTotal");

    // Determine swap presence
    let has_swap = !swap_files.trim().is_empty()
        || swap_total_kb.map(|s| s > 0).unwrap_or(false);

    let (summary, evidence_summary) = if has_swap {
        let swap_size = swap_total_kb.map(|kb| format!("{:.1} GiB", kb as f64 / 1_048_576.0));
        let summary = match swap_size {
            Some(size) => format!("Yes, swap is configured ({} total)", size),
            None => "Yes, swap is configured on this system".to_string(),
        };
        (summary, "Swap partition/file found")
    } else {
        ("No swap is configured on this system".to_string(), "No swap entries found")
    };

    HandlerResult::Success(
        StrictSpecialistResponse::ok(ticket_id, "check_swap_presence", &summary, 0.95)
            .with_evidence("swap_files", evidence_summary)
            .with_metrics(json!({ "swap_present": has_swap }))
    )
}

/// Handle check_disk_usage intent
pub fn handle_check_disk_usage(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let disk_usage = match probes.get("disk_usage") {
        Some(d) if !d.trim().is_empty() => d,
        _ => return HandlerResult::MissingProbe {
            probe_name: "disk_usage".to_string(),
            reason: "Need 'df -h' output to check disk usage".to_string(),
        },
    };

    // Parse df -h output, look for root filesystem
    let mut root_usage: Option<(String, u32, String, String)> = None; // (device, percent, used, size)
    let mut critical_filesystems: Vec<String> = vec![];

    for line in disk_usage.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let device = parts[0];
            let size = parts[1];
            let used = parts[2];
            let mount = parts[5];

            // Parse percentage (remove %)
            let percent_str = parts[4].trim_end_matches('%');
            if let Ok(percent) = percent_str.parse::<u32>() {
                // Check for root filesystem
                if mount == "/" {
                    root_usage = Some((device.to_string(), percent, used.to_string(), size.to_string()));
                }
                // Track critical filesystems (>90%)
                if percent >= 90 {
                    critical_filesystems.push(format!("{} at {}%", mount, percent));
                }
            }
        }
    }

    let (summary, status) = match (&root_usage, critical_filesystems.len()) {
        (Some((device, percent, used, size)), _) if *percent >= 95 => {
            (format!("[CRITICAL] Root filesystem {} is at {}% ({} used of {})", device, percent, used, size), StrictStatus::Ok)
        }
        (Some((device, percent, used, size)), _) if *percent >= 90 => {
            (format!("[WARNING] Root filesystem {} is at {}% ({} used of {})", device, percent, used, size), StrictStatus::Ok)
        }
        (Some((device, percent, used, size)), _) => {
            (format!("Root filesystem {} is at {}% ({} used of {})", device, percent, used, size), StrictStatus::Ok)
        }
        (None, _) => {
            ("Could not determine root filesystem usage from df output".to_string(), StrictStatus::Partial)
        }
    };

    let mut response = if status == StrictStatus::Ok {
        StrictSpecialistResponse::ok(ticket_id, "check_disk_usage", &summary, 0.95)
    } else {
        StrictSpecialistResponse::partial(ticket_id, "check_disk_usage", &summary)
    };

    response.evidence.push(EvidenceItem {
        probe: "disk_usage".to_string(),
        summary: "df -h output parsed".to_string(),
    });

    if let Some((_, percent, _, _)) = root_usage {
        response.metrics = Some(json!({ "root_usage_percent": percent }));
    }

    if !critical_filesystems.is_empty() {
        response.details = critical_filesystems;
    }

    HandlerResult::Success(response)
}

/// Handle check_failed_services intent
pub fn handle_check_failed_services(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let failed_services = match probes.get("failed_services") {
        Some(f) => f,
        None => return HandlerResult::MissingProbe {
            probe_name: "failed_services".to_string(),
            reason: "Need 'systemctl --failed' output".to_string(),
        },
    };

    // Parse failed services - count lines that look like unit entries
    let failed_units: Vec<&str> = failed_services
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Look for unit names (ends with .service, .socket, etc.)
            trimmed.contains(".service") || trimmed.contains(".socket") ||
            trimmed.contains(".timer") || trimmed.contains(".mount")
        })
        .filter(|line| {
            // Skip header/footer lines
            !line.contains("UNIT") && !line.contains("LOAD") && !line.contains("loaded units")
        })
        .collect();

    let count = failed_units.len();
    let summary = if count == 0 {
        "No failed systemd services".to_string()
    } else {
        let names: Vec<String> = failed_units.iter()
            .map(|line| {
                line.split_whitespace().next().unwrap_or("unknown").to_string()
            })
            .take(3)
            .collect();
        if count <= 3 {
            format!("{} failed service(s): {}", count, names.join(", "))
        } else {
            format!("{} failed services: {} and {} more", count, names.join(", "), count - 3)
        }
    };

    let mut response = StrictSpecialistResponse::ok(ticket_id, "check_failed_services", &summary, 0.95)
        .with_evidence("failed_services", &format!("{} failed units detected", count))
        .with_metrics(json!({ "failed_count": count }));

    // Add action if there are failures
    if count > 0 {
        response.actions.push(crate::strict_contract::SuggestedAction {
            kind: crate::strict_contract::ActionKind::Investigate,
            description: "Check service status for details".to_string(),
            command: Some("systemctl status <service-name>".to_string()),
            risk: crate::strict_contract::RiskLevel::Low,
        });
    }

    HandlerResult::Success(response)
}

/// Handle check_boot_time intent
pub fn handle_check_boot_time(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let boot_time = match probes.get("boot_time") {
        Some(b) if !b.trim().is_empty() => b,
        _ => return HandlerResult::MissingProbe {
            probe_name: "boot_time".to_string(),
            reason: "Need 'systemd-analyze' output".to_string(),
        },
    };

    // Parse systemd-analyze output
    // Format: "Startup finished in Xs (firmware) + Ys (loader) + Zs (kernel) + Ws (userspace) = Ts"
    let total_match = extract_boot_total(boot_time);

    match total_match {
        Some(total_secs) => {
            let summary = format!("Boot time: {:.1}s total", total_secs);

            HandlerResult::Success(
                StrictSpecialistResponse::ok(ticket_id, "check_boot_time", &summary, 0.95)
                    .with_evidence("boot_time", &format!("systemd-analyze shows {:.1}s", total_secs))
                    .with_metrics(json!({ "boot_time_seconds": total_secs }))
            )
        }
        None => {
            // Try to extract just the total from the line
            let summary = format!("Boot analysis: {}", boot_time.lines().next().unwrap_or("unknown"));
            HandlerResult::Success(
                StrictSpecialistResponse::ok(ticket_id, "check_boot_time", &summary, 0.8)
                    .with_evidence("boot_time", "Raw systemd-analyze output")
            )
        }
    }
}

/// Handle check_package_count intent
pub fn handle_check_package_count(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let package_count = match probes.get("package_count") {
        Some(p) if !p.trim().is_empty() => p.trim(),
        _ => return HandlerResult::MissingProbe {
            probe_name: "package_count".to_string(),
            reason: "Need 'pacman -Qq | wc -l' output".to_string(),
        },
    };

    // Parse the count
    match package_count.parse::<u32>() {
        Ok(count) => {
            let summary = format!("You have {} packages installed", count);
            HandlerResult::Success(
                StrictSpecialistResponse::ok(ticket_id, "check_package_count", &summary, 0.95)
                    .with_evidence("package_count", &format!("{} packages", count))
                    .with_metrics(json!({ "package_count": count }))
            )
        }
        Err(_) => {
            // Might be multi-line, try first line
            if let Ok(count) = package_count.lines().next().unwrap_or("0").trim().parse::<u32>() {
                let summary = format!("You have {} packages installed", count);
                HandlerResult::Success(
                    StrictSpecialistResponse::ok(ticket_id, "check_package_count", &summary, 0.95)
                        .with_evidence("package_count", &format!("{} packages", count))
                        .with_metrics(json!({ "package_count": count }))
                )
            } else {
                HandlerResult::NeedsSpecialist {
                    reason: format!("Could not parse package count: {}", package_count),
                }
            }
        }
    }
}

/// Handle check_uptime intent
pub fn handle_check_uptime(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let uptime = match probes.get("uptime") {
        Some(u) if !u.trim().is_empty() => u.trim(),
        _ => return HandlerResult::MissingProbe {
            probe_name: "uptime".to_string(),
            reason: "Need 'uptime' command output".to_string(),
        },
    };

    // Parse uptime output - look for "up X days, Y:Z" or "up X:Y"
    let summary = if let Some(up_part) = uptime.split("up").nth(1) {
        let up_str = up_part.split(',').next().unwrap_or(up_part).trim();
        format!("System uptime: {}", up_str)
    } else {
        format!("Uptime: {}", uptime.lines().next().unwrap_or(uptime))
    };

    HandlerResult::Success(
        StrictSpecialistResponse::ok(ticket_id, "check_uptime", &summary, 0.95)
            .with_evidence("uptime", "uptime command output")
    )
}

/// Handle check_package_installed intent
pub fn handle_check_package_installed(
    ticket_id: &str,
    package_name: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    // Look for package_check_<name> probe
    let probe_key = format!("package_check_{}", package_name.to_lowercase());

    let probe_output = match probes.get(&probe_key) {
        Some(p) => p.trim(),
        None => {
            // Also try installed_packages probe
            if let Some(installed) = probes.get("installed_packages") {
                if installed.lines().any(|l| l.trim().starts_with(package_name)) {
                    return HandlerResult::Success(
                        StrictSpecialistResponse::ok(
                            ticket_id,
                            "check_package_installed",
                            &format!("Yes, {} is installed", package_name),
                            0.9
                        ).with_evidence("installed_packages", "Found in package list")
                    );
                }
            }
            return HandlerResult::MissingProbe {
                probe_name: probe_key,
                reason: format!("Need 'pacman -Q {}' output", package_name),
            };
        }
    };

    if probe_output.is_empty() {
        // Empty output means not installed
        let summary = format!("No, {} is not installed", package_name);
        HandlerResult::Success(
            StrictSpecialistResponse::ok(ticket_id, "check_package_installed", &summary, 0.95)
                .with_evidence(&probe_key, "Package not found")
                .with_metrics(json!({ "installed": false }))
        )
    } else {
        // Has output - parse version
        let version = probe_output.split_whitespace().nth(1).unwrap_or("installed");
        let summary = format!("Yes, {} {} is installed", package_name, version);
        HandlerResult::Success(
            StrictSpecialistResponse::ok(ticket_id, "check_package_installed", &summary, 0.95)
                .with_evidence(&probe_key, probe_output)
                .with_metrics(json!({ "installed": true, "version": version }))
        )
    }
}

/// Handle list_top_memory_processes intent
pub fn handle_list_top_memory_processes(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let top_mem = match probes.get("top_memory") {
        Some(t) if !t.trim().is_empty() => t,
        _ => return HandlerResult::MissingProbe {
            probe_name: "top_memory".to_string(),
            reason: "Need 'ps aux --sort=-%mem' output".to_string(),
        },
    };

    // Parse ps output - skip header, get top 5
    let processes: Vec<String> = top_mem
        .lines()
        .skip(1) // Skip header
        .take(5)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let mem_percent = parts[3];
                let command = parts[10..].join(" ");
                Some(format!("{}% - {}", mem_percent, command))
            } else {
                None
            }
        })
        .collect();

    if processes.is_empty() {
        return HandlerResult::NeedsSpecialist {
            reason: "Could not parse process list".to_string(),
        };
    }

    let summary = format!("Top {} memory-using processes:", processes.len());
    let mut response = StrictSpecialistResponse::ok(ticket_id, "list_top_memory_processes", &summary, 0.95)
        .with_evidence("top_memory", &format!("{} processes parsed", processes.len()));
    response.details = processes;

    HandlerResult::Success(response)
}

// Helper functions

fn extract_meminfo_value(meminfo: &str, key: &str) -> Option<u64> {
    for line in meminfo.lines() {
        if line.starts_with(key) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse().ok();
            }
        }
    }
    None
}

fn extract_boot_total(boot_output: &str) -> Option<f64> {
    // Look for "= Xs" or "= X.Ys" pattern
    for line in boot_output.lines() {
        if line.contains("=") {
            // Find the total time after =
            if let Some(after_eq) = line.split('=').last() {
                let trimmed = after_eq.trim();
                // Extract number before 's'
                let num_str = trimmed.trim_end_matches('s').trim();
                if let Ok(secs) = num_str.parse::<f64>() {
                    return Some(secs);
                }
            }
        }
    }
    None
}

/// Dispatch to appropriate handler based on intent
pub fn dispatch_handler(
    ticket_id: &str,
    intent: &str,
    probes: &HashMap<String, String>,
    question: &str,
) -> HandlerResult {
    match intent {
        "check_free_ram" | "query_metric" if question.to_lowercase().contains("ram") || question.to_lowercase().contains("memory") => {
            handle_check_free_ram(ticket_id, probes)
        }
        "check_swap_presence" | "check_swap" => {
            handle_check_swap_presence(ticket_id, probes)
        }
        "check_disk_usage" | "query_metric" if question.to_lowercase().contains("disk") || question.to_lowercase().contains("space") => {
            handle_check_disk_usage(ticket_id, probes)
        }
        "check_failed_services" | "check_status" if question.to_lowercase().contains("failed") && question.to_lowercase().contains("service") => {
            handle_check_failed_services(ticket_id, probes)
        }
        "check_boot_time" | "query_metric" if question.to_lowercase().contains("boot") => {
            handle_check_boot_time(ticket_id, probes)
        }
        "check_package_count" | "query_metric" if question.to_lowercase().contains("package") && question.to_lowercase().contains("count") => {
            handle_check_package_count(ticket_id, probes)
        }
        "check_uptime" | "query_metric" if question.to_lowercase().contains("uptime") => {
            handle_check_uptime(ticket_id, probes)
        }
        "list_top_memory_processes" | "list" if question.to_lowercase().contains("memory") && question.to_lowercase().contains("process") => {
            handle_list_top_memory_processes(ticket_id, probes)
        }
        _ => {
            // Check if we can infer intent from probes available
            if probes.contains_key("memory_info") && (question.contains("ram") || question.contains("memory")) {
                return handle_check_free_ram(ticket_id, probes);
            }
            if probes.contains_key("disk_usage") && (question.contains("disk") || question.contains("space")) {
                return handle_check_disk_usage(ticket_id, probes);
            }
            if probes.contains_key("failed_services") && question.contains("failed") {
                return handle_check_failed_services(ticket_id, probes);
            }
            if probes.contains_key("boot_time") && question.contains("boot") {
                return handle_check_boot_time(ticket_id, probes);
            }
            if probes.contains_key("uptime") && question.contains("uptime") {
                return handle_check_uptime(ticket_id, probes);
            }

            HandlerResult::NeedsSpecialist {
                reason: format!("No deterministic handler for intent '{}' with question '{}'", intent, question),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_free_ram() {
        let mut probes = HashMap::new();
        probes.insert("memory_info".to_string(),
            "MemTotal:       32768000 kB\nMemAvailable:   17892232 kB".to_string());

        match handle_check_free_ram("TEST-001", &probes) {
            HandlerResult::Success(r) => {
                assert_eq!(r.status, StrictStatus::Ok);
                assert!(r.summary.contains("17."));
                assert!(r.summary.contains("GiB"));
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_check_failed_services_none() {
        let mut probes = HashMap::new();
        probes.insert("failed_services".to_string(), "".to_string());

        match handle_check_failed_services("TEST-001", &probes) {
            HandlerResult::Success(r) => {
                assert!(r.summary.contains("No failed"));
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_check_swap_presence() {
        let mut probes = HashMap::new();
        probes.insert("swap_files".to_string(), "".to_string());
        probes.insert("memory_info".to_string(), "SwapTotal: 0 kB".to_string());

        match handle_check_swap_presence("TEST-001", &probes) {
            HandlerResult::Success(r) => {
                assert!(r.summary.to_lowercase().contains("no swap"));
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_check_disk_usage() {
        let mut probes = HashMap::new();
        probes.insert("disk_usage".to_string(),
            "Filesystem      Size  Used Avail Use% Mounted on\n/dev/nvme0n1p1  100G   50G   50G  50% /".to_string());

        match handle_check_disk_usage("TEST-001", &probes) {
            HandlerResult::Success(r) => {
                assert!(r.summary.contains("50%"));
            }
            _ => panic!("Expected success"),
        }
    }
}
