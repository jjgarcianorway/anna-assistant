//! Fact extraction from probe outputs (v0.0.428).

use super::fallback_types::ExtractedFact;
use std::collections::HashMap;

/// Extract facts from raw probe outputs
pub(super) fn extract_facts_from_probes(
    probes: &HashMap<String, String>,
    intent: &str,
) -> Vec<ExtractedFact> {
    let mut facts = vec![];

    for (probe_id, output) in probes {
        if output.trim().is_empty() {
            continue;
        }

        // Try to extract meaningful data based on probe type
        if let Some(fact) = extract_fact_from_probe(probe_id, output, intent) {
            facts.push(fact);
        }
    }

    facts
}

/// Extract a fact from a specific probe output
fn extract_fact_from_probe(probe_id: &str, output: &str, _intent: &str) -> Option<ExtractedFact> {
    let probe_lower = probe_id.to_lowercase();
    let output_trimmed = output.trim();

    // Memory probes (free, /proc/meminfo)
    if probe_lower.contains("free") || probe_lower.contains("meminfo") {
        return extract_memory_fact(probe_id, output_trimmed);
    }

    // Disk probes (df, lsblk)
    if probe_lower.contains("df") || probe_lower.contains("disk") {
        return extract_disk_fact(probe_id, output_trimmed);
    }

    // Systemd probes (systemctl)
    if probe_lower.contains("systemctl") || probe_lower.contains("systemd") {
        return extract_systemd_fact(probe_id, output_trimmed);
    }

    // Failed services
    if probe_lower.contains("failed") {
        return extract_failed_services_fact(probe_id, output_trimmed);
    }

    // Generic: if output is short enough, use as-is
    if output_trimmed.len() < 200 && !output_trimmed.is_empty() {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: format!("{} output available", probe_id),
            raw_snippet: truncate(output_trimmed, 150),
        });
    }

    None
}

/// Extract memory fact from free/meminfo output
fn extract_memory_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    // Look for "Mem:" line in free output
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let total = parts.get(1).unwrap_or(&"?");
                let available = parts.get(6).unwrap_or(&"?");
                return Some(ExtractedFact {
                    probe_id: probe_id.to_string(),
                    summary: format!("{} available out of {} total RAM", available, total),
                    raw_snippet: line.to_string(),
                });
            }
        }
    }

    // Look for MemAvailable in /proc/meminfo
    for line in output.lines() {
        if line.starts_with("MemAvailable:") {
            let value = line.split(':').nth(1).map(|s| s.trim()).unwrap_or("?");
            return Some(ExtractedFact {
                probe_id: probe_id.to_string(),
                summary: format!("{} memory available", value),
                raw_snippet: line.to_string(),
            });
        }
    }

    None
}

/// Extract disk fact from df output
fn extract_disk_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    // Look for root filesystem line
    for line in output.lines() {
        if line.contains(" /") && !line.contains(" /boot") && !line.starts_with("Filesystem") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let used_pct = parts.get(4).unwrap_or(&"?");
                let available = parts.get(3).unwrap_or(&"?");
                return Some(ExtractedFact {
                    probe_id: probe_id.to_string(),
                    summary: format!(
                        "Root filesystem at {} used ({} available)",
                        used_pct, available
                    ),
                    raw_snippet: line.to_string(),
                });
            }
        }
    }

    None
}

/// Extract systemd fact from systemctl output
fn extract_systemd_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    let output_lower = output.to_lowercase();

    // Check for running status
    if output_lower.contains("active (running)") {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: "Service is active and running".to_string(),
            raw_snippet: truncate(output, 100),
        });
    }

    // Check for failed status
    if output_lower.contains("failed") || output_lower.contains("inactive (dead)") {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: "Service is not running or has failed".to_string(),
            raw_snippet: truncate(output, 100),
        });
    }

    None
}

/// Extract failed services fact
fn extract_failed_services_fact(probe_id: &str, output: &str) -> Option<ExtractedFact> {
    let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();

    // Count failed units (excluding header)
    let failed_count = lines
        .iter()
        .filter(|l| l.contains(".service") || l.contains(".socket") || l.contains(".timer"))
        .count();

    if failed_count == 0 {
        return Some(ExtractedFact {
            probe_id: probe_id.to_string(),
            summary: "No failed systemd units".to_string(),
            raw_snippet: output.lines().next().unwrap_or("").to_string(),
        });
    }

    // List up to 3 failed services
    let failed_names: Vec<&str> = lines
        .iter()
        .filter(|l| l.contains(".service"))
        .take(3)
        .map(|l| l.split_whitespace().next().unwrap_or(""))
        .collect();

    let summary = if failed_count <= 3 {
        format!(
            "{} failed service(s): {}",
            failed_count,
            failed_names.join(", ")
        )
    } else {
        format!(
            "{} failed service(s), including: {}",
            failed_count,
            failed_names.join(", ")
        )
    };

    Some(ExtractedFact {
        probe_id: probe_id.to_string(),
        summary,
        raw_snippet: truncate(output, 150),
    })
}

/// Truncate string to max length
pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
