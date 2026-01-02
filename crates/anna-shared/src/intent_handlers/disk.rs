//! Disk-related intent handlers.

use crate::strict_contract::{EvidenceItem, StrictSpecialistResponse, StrictStatus};
use serde_json::json;
use std::collections::HashMap;

use super::types::HandlerResult;

/// Handle check_disk_usage intent
pub fn handle_check_disk_usage(ticket_id: &str, probes: &HashMap<String, String>) -> HandlerResult {
    let disk_usage = match probes.get("disk_usage") {
        Some(d) if !d.trim().is_empty() => d,
        _ => {
            return HandlerResult::MissingProbe {
                probe_name: "disk_usage".to_string(),
                reason: "Need 'df -h' output to check disk usage".to_string(),
            }
        }
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
                    root_usage = Some((
                        device.to_string(),
                        percent,
                        used.to_string(),
                        size.to_string(),
                    ));
                }
                // Track critical filesystems (>90%)
                if percent >= 90 {
                    critical_filesystems.push(format!("{} at {}%", mount, percent));
                }
            }
        }
    }

    let (summary, status) = match (&root_usage, critical_filesystems.len()) {
        (Some((device, percent, used, size)), _) if *percent >= 95 => (
            format!(
                "[CRITICAL] Root filesystem {} is at {}% ({} used of {})",
                device, percent, used, size
            ),
            StrictStatus::Ok,
        ),
        (Some((device, percent, used, size)), _) if *percent >= 90 => (
            format!(
                "[WARNING] Root filesystem {} is at {}% ({} used of {})",
                device, percent, used, size
            ),
            StrictStatus::Ok,
        ),
        (Some((device, percent, used, size)), _) => (
            format!(
                "Root filesystem {} is at {}% ({} used of {})",
                device, percent, used, size
            ),
            StrictStatus::Ok,
        ),
        (None, _) => (
            "Could not determine root filesystem usage from df output".to_string(),
            StrictStatus::Partial,
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

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
