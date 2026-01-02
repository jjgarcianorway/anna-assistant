//! Specific fallback handlers for different question types.

use std::collections::HashMap;

use crate::specialist_v2::answer::{DirectAnswer, FindingSeverity, KeyFinding};
use crate::specialist_v2::schema::SpecialistResponseV2;
use crate::specialist_v2::FALLBACK_CONFIDENCE;

use super::parsers::{
    extract_boot_time, extract_swap_size, extract_uptime, format_bytes, parse_disk_usage,
    parse_failed_services, parse_memory_from_free, parse_network_interfaces,
};

/// Fallback for memory questions
pub(super) fn try_memory_fallback(
    probes: &HashMap<String, String>,
) -> Option<SpecialistResponseV2> {
    // Try to find memory probe data
    let free_output = find_probe(probes, &["free", "free_h", "meminfo", "proc_meminfo"])?;

    // Parse memory from free output
    if let Some((available, total, used)) = parse_memory_from_free(&free_output) {
        let percent = (available as f64 / total as f64 * 100.0) as u32;

        let mut metrics = HashMap::new();
        metrics.insert(
            "mem_available_bytes".to_string(),
            serde_json::json!(available),
        );
        metrics.insert("mem_total_bytes".to_string(), serde_json::json!(total));
        metrics.insert("mem_used_bytes".to_string(), serde_json::json!(used));

        let answer = DirectAnswer::with_metrics(
            &format!(
                "Available memory: {} ({}% of {} total)",
                format_bytes(available),
                percent,
                format_bytes(total)
            ),
            metrics,
        );

        return Some(
            SpecialistResponseV2::ok()
                .with_direct_answer(answer)
                .with_finding(KeyFinding::info("available", &format_bytes(available)))
                .with_finding(KeyFinding::info("total", &format_bytes(total)))
                .with_finding(KeyFinding::info("used", &format_bytes(used)))
                .with_confidence(FALLBACK_CONFIDENCE)
                .with_citation("probe:free")
                .with_notes("Answer from probe data (specialist unavailable)"),
        );
    }

    None
}

/// Fallback for service questions
pub(super) fn try_services_fallback(
    probes: &HashMap<String, String>,
) -> Option<SpecialistResponseV2> {
    let output = find_probe(
        probes,
        &[
            "systemctl_failed",
            "systemd_failed",
            "failed_services",
            "systemctl",
        ],
    )?;

    let failed = parse_failed_services(&output);

    if failed.is_empty() {
        return Some(
            SpecialistResponseV2::ok()
                .with_direct_answer(DirectAnswer::no("there are no failed systemd services."))
                .with_confidence(FALLBACK_CONFIDENCE)
                .with_citation("probe:systemctl_failed"),
        );
    }

    let count = failed.len();
    let answer = if count == 1 {
        DirectAnswer::yes(&format!("1 failed service: {}", failed[0]))
    } else {
        DirectAnswer::yes(&format!("{} failed services: {}", count, failed.join(", ")))
    };

    let mut response = SpecialistResponseV2::ok()
        .with_direct_answer(answer)
        .with_confidence(FALLBACK_CONFIDENCE)
        .with_citation("probe:systemctl_failed");

    for service in failed {
        response = response.with_finding(KeyFinding::warning("failed_service", &service));
    }

    Some(response)
}

/// Fallback for disk questions
pub(super) fn try_disk_fallback(
    probes: &HashMap<String, String>,
) -> Option<SpecialistResponseV2> {
    let output = find_probe(probes, &["df", "df_h", "disk_usage", "lsblk"])?;
    let partitions = parse_disk_usage(&output);

    if partitions.is_empty() {
        return None;
    }

    // Find most critical partition
    let critical: Vec<_> = partitions.iter().filter(|p| p.2 >= 90).collect();
    let warning: Vec<_> = partitions
        .iter()
        .filter(|p| p.2 >= 80 && p.2 < 90)
        .collect();

    let answer_text = if !critical.is_empty() {
        format!("Critical: {} at {}% capacity", critical[0].0, critical[0].2)
    } else if !warning.is_empty() {
        format!("Warning: {} at {}% capacity", warning[0].0, warning[0].2)
    } else {
        let root = partitions
            .iter()
            .find(|p| p.0 == "/")
            .unwrap_or(&partitions[0]);
        format!("Root filesystem: {}% used", root.2)
    };

    let mut response = SpecialistResponseV2::ok()
        .with_direct_answer(DirectAnswer::simple(&answer_text))
        .with_confidence(FALLBACK_CONFIDENCE)
        .with_citation("probe:df");

    for (mount, size, percent) in partitions {
        let severity = if percent >= 90 {
            FindingSeverity::Critical
        } else if percent >= 80 {
            FindingSeverity::Warning
        } else {
            FindingSeverity::Info
        };
        response = response.with_finding(
            KeyFinding::new(&mount, &format!("{}% of {}", percent, size)).with_severity(severity),
        );
    }

    Some(response)
}

/// Fallback for network questions
pub(super) fn try_network_fallback(
    probes: &HashMap<String, String>,
) -> Option<SpecialistResponseV2> {
    let output = find_probe(probes, &["ip_addr", "ip_brief", "ifconfig", "interfaces"])?;
    let interfaces = parse_network_interfaces(&output);

    if interfaces.is_empty() {
        return None;
    }

    let active: Vec<_> = interfaces
        .iter()
        .filter(|(_, state, _)| state == "UP")
        .collect();

    let answer_text = if active.is_empty() {
        "No active network interfaces found.".to_string()
    } else {
        format!(
            "{} active interface(s): {}",
            active.len(),
            active
                .iter()
                .map(|(n, _, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let mut response = SpecialistResponseV2::ok()
        .with_direct_answer(DirectAnswer::simple(&answer_text))
        .with_confidence(FALLBACK_CONFIDENCE)
        .with_citation("probe:ip_addr");

    for (name, state, ip) in interfaces {
        let severity = if state == "UP" {
            FindingSeverity::Info
        } else {
            FindingSeverity::Warning
        };
        let value = if ip.is_empty() {
            state.clone()
        } else {
            format!("{} ({})", state, ip)
        };
        response =
            response.with_finding(KeyFinding::new(&name, &value).with_severity(severity));
    }

    Some(response)
}

/// Fallback for swap questions
pub(super) fn try_swap_fallback(
    probes: &HashMap<String, String>,
) -> Option<SpecialistResponseV2> {
    let output = find_probe(probes, &["swapon", "swap", "free", "proc_swaps"])?;

    // Check for "no swap" indicators
    let output_lower = output.to_lowercase();
    if output_lower.contains("no swap")
        || output_lower.contains("swap: 0")
        || (output_lower.contains("swap") && output_lower.contains("0b"))
    {
        return Some(
            SpecialistResponseV2::ok()
                .with_direct_answer(DirectAnswer::no("swap is not configured."))
                .with_confidence(FALLBACK_CONFIDENCE)
                .with_citation("probe:swapon"),
        );
    }

    // Try to find swap size
    if let Some(size) = extract_swap_size(&output) {
        return Some(
            SpecialistResponseV2::ok()
                .with_direct_answer(DirectAnswer::yes(&format!("swap is enabled ({}).", size)))
                .with_finding(KeyFinding::info("swap_size", &size))
                .with_confidence(FALLBACK_CONFIDENCE)
                .with_citation("probe:swapon"),
        );
    }

    None
}

/// Fallback for uptime questions
pub(super) fn try_uptime_fallback(
    probes: &HashMap<String, String>,
) -> Option<SpecialistResponseV2> {
    let output = find_probe(probes, &["uptime", "uptime_p", "proc_uptime"])?;

    // Extract uptime info
    if let Some(uptime_str) = extract_uptime(&output) {
        return Some(
            SpecialistResponseV2::ok()
                .with_direct_answer(DirectAnswer::simple(&format!(
                    "System uptime: {}",
                    uptime_str
                )))
                .with_finding(KeyFinding::info("uptime", &uptime_str))
                .with_confidence(FALLBACK_CONFIDENCE)
                .with_citation("probe:uptime"),
        );
    }

    None
}

/// Fallback for boot time questions
pub(super) fn try_boot_fallback(
    probes: &HashMap<String, String>,
) -> Option<SpecialistResponseV2> {
    let output = find_probe(probes, &["systemd_analyze", "boot_time", "systemd_blame"])?;

    if let Some(boot_time) = extract_boot_time(&output) {
        return Some(
            SpecialistResponseV2::ok()
                .with_direct_answer(DirectAnswer::simple(&format!("Boot time: {}", boot_time)))
                .with_finding(KeyFinding::info("boot_time", &boot_time))
                .with_confidence(FALLBACK_CONFIDENCE)
                .with_citation("probe:systemd_analyze"),
        );
    }

    None
}

/// Find probe output by trying multiple possible names
fn find_probe(probes: &HashMap<String, String>, names: &[&str]) -> Option<String> {
    for name in names {
        if let Some(output) = probes.get(*name) {
            if !output.trim().is_empty() {
                return Some(output.clone());
            }
        }
    }
    None
}
