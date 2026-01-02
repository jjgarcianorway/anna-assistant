//! System information intent handlers.

use crate::strict_contract::StrictSpecialistResponse;
use serde_json::json;
use std::collections::HashMap;

use super::helpers::extract_boot_total;
use super::types::HandlerResult;

/// Handle check_boot_time intent
pub fn handle_check_boot_time(ticket_id: &str, probes: &HashMap<String, String>) -> HandlerResult {
    let boot_time = match probes.get("boot_time") {
        Some(b) if !b.trim().is_empty() => b,
        _ => {
            return HandlerResult::MissingProbe {
                probe_name: "boot_time".to_string(),
                reason: "Need 'systemd-analyze' output".to_string(),
            }
        }
    };

    // Parse systemd-analyze output
    // Format: "Startup finished in Xs (firmware) + Ys (loader) + Zs (kernel) + Ws (userspace) = Ts"
    let total_match = extract_boot_total(boot_time);

    match total_match {
        Some(total_secs) => {
            let summary = format!("Boot time: {:.1}s total", total_secs);

            HandlerResult::Success(
                StrictSpecialistResponse::ok(ticket_id, "check_boot_time", &summary, 0.95)
                    .with_evidence(
                        "boot_time",
                        &format!("systemd-analyze shows {:.1}s", total_secs),
                    )
                    .with_metrics(json!({ "boot_time_seconds": total_secs })),
            )
        }
        None => {
            // Try to extract just the total from the line
            let summary = format!(
                "Boot analysis: {}",
                boot_time.lines().next().unwrap_or("unknown")
            );
            HandlerResult::Success(
                StrictSpecialistResponse::ok(ticket_id, "check_boot_time", &summary, 0.8)
                    .with_evidence("boot_time", "Raw systemd-analyze output"),
            )
        }
    }
}

/// Handle check_uptime intent
pub fn handle_check_uptime(ticket_id: &str, probes: &HashMap<String, String>) -> HandlerResult {
    let uptime = match probes.get("uptime") {
        Some(u) if !u.trim().is_empty() => u.trim(),
        _ => {
            return HandlerResult::MissingProbe {
                probe_name: "uptime".to_string(),
                reason: "Need 'uptime' command output".to_string(),
            }
        }
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
            .with_evidence("uptime", "uptime command output"),
    )
}
