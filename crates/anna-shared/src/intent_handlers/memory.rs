//! Memory-related intent handlers.

use crate::strict_contract::StrictSpecialistResponse;
use serde_json::json;
use std::collections::HashMap;

use super::helpers::extract_meminfo_value;
use super::types::HandlerResult;

/// Handle check_free_ram intent
pub fn handle_check_free_ram(ticket_id: &str, probes: &HashMap<String, String>) -> HandlerResult {
    // Required: memory_info probe
    let mem_info = match probes.get("memory_info") {
        Some(m) if !m.trim().is_empty() => m,
        _ => {
            return HandlerResult::MissingProbe {
                probe_name: "memory_info".to_string(),
                reason: "Need /proc/meminfo or 'free -h' output to check RAM".to_string(),
            }
        }
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
                    .with_evidence(
                        "memory_info",
                        &format!("MemTotal: {} kB, MemAvailable: {} kB", total, available),
                    )
                    .with_metrics(json!({
                        "mem_total_gb": format!("{:.1}", total_gb),
                        "mem_available_gb": format!("{:.1}", available_gb),
                        "mem_used_percent": used_percent
                    })),
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
    let has_swap = !swap_files.trim().is_empty() || swap_total_kb.map(|s| s > 0).unwrap_or(false);

    let (summary, evidence_summary) = if has_swap {
        let swap_size = swap_total_kb.map(|kb| format!("{:.1} GiB", kb as f64 / 1_048_576.0));
        let summary = match swap_size {
            Some(size) => format!("Yes, swap is configured ({} total)", size),
            None => "Yes, swap is configured on this system".to_string(),
        };
        (summary, "Swap partition/file found")
    } else {
        (
            "No swap is configured on this system".to_string(),
            "No swap entries found",
        )
    };

    HandlerResult::Success(
        StrictSpecialistResponse::ok(ticket_id, "check_swap_presence", &summary, 0.95)
            .with_evidence("swap_files", evidence_summary)
            .with_metrics(json!({ "swap_present": has_swap })),
    )
}

/// Handle list_top_memory_processes intent
pub fn handle_list_top_memory_processes(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let top_mem = match probes.get("top_memory") {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            return HandlerResult::MissingProbe {
                probe_name: "top_memory".to_string(),
                reason: "Need 'ps aux --sort=-%mem' output".to_string(),
            }
        }
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
    let mut response =
        StrictSpecialistResponse::ok(ticket_id, "list_top_memory_processes", &summary, 0.95)
            .with_evidence(
                "top_memory",
                &format!("{} processes parsed", processes.len()),
            );
    response.details = processes;

    HandlerResult::Success(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_free_ram() {
        let mut probes = HashMap::new();
        probes.insert(
            "memory_info".to_string(),
            "MemTotal:       32768000 kB\nMemAvailable:   17892232 kB".to_string(),
        );

        match handle_check_free_ram("TEST-001", &probes) {
            HandlerResult::Success(r) => {
                assert_eq!(r.status, crate::strict_contract::StrictStatus::Ok);
                assert!(r.summary.contains("17."));
                assert!(r.summary.contains("GiB"));
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
}
