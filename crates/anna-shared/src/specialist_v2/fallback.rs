//! Fallback engine for specialist failures (v0.0.421).
//!
//! Provides deterministic fallback answers when:
//! - JSON parsing fails
//! - LLM response is invalid
//! - Timeout occurs
//!
//! Covers common question types:
//! - Memory usage
//! - Failed services
//! - Disk usage
//! - Network interfaces
//! - Swap status

use std::collections::HashMap;

use super::answer::{AnswerType, DirectAnswer, FindingSeverity, KeyFinding};
use super::schema::{SpecialistResponseV2, SpecialistStatus};
use super::FALLBACK_CONFIDENCE;

/// Result from fallback engine
#[derive(Debug, Clone)]
pub struct FallbackResult {
    /// The generated response
    pub response: SpecialistResponseV2,
    /// Whether fallback was successful
    pub success: bool,
    /// Reason for fallback
    pub reason: String,
}

/// Fallback engine for generating deterministic answers from probe data
pub struct FallbackEngine {
    /// Probe results available
    probes: HashMap<String, String>,
    /// Intent string
    intent: String,
    /// Original question (kept for future use in more complex fallbacks)
    #[allow(dead_code)]
    question: String,
}

impl FallbackEngine {
    /// Create a new fallback engine
    pub fn new(intent: &str, question: &str, probes: HashMap<String, String>) -> Self {
        Self {
            probes,
            intent: intent.to_string(),
            question: question.to_string(),
        }
    }

    /// Try to generate a fallback response
    pub fn try_fallback(&self, reason: &str) -> FallbackResult {
        let answer_type = AnswerType::from_intent(&self.intent);

        // Try specific fallbacks based on intent
        let response = self
            .try_memory_fallback()
            .or_else(|| self.try_services_fallback())
            .or_else(|| self.try_disk_fallback())
            .or_else(|| self.try_network_fallback())
            .or_else(|| self.try_swap_fallback())
            .or_else(|| self.try_uptime_fallback())
            .or_else(|| self.try_boot_fallback())
            .unwrap_or_else(|| self.generic_fallback(reason));

        FallbackResult {
            success: response.status == SpecialistStatus::Ok,
            response,
            reason: reason.to_string(),
        }
    }

    /// Fallback for memory questions
    fn try_memory_fallback(&self) -> Option<SpecialistResponseV2> {
        if !self.intent.contains("memory") && !self.intent.contains("ram") {
            return None;
        }

        // Try to find memory probe data
        let free_output = self.find_probe(&["free", "free_h", "meminfo", "proc_meminfo"])?;

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
    fn try_services_fallback(&self) -> Option<SpecialistResponseV2> {
        if !self.intent.contains("service") && !self.intent.contains("failed") {
            return None;
        }

        let output = self.find_probe(&[
            "systemctl_failed",
            "systemd_failed",
            "failed_services",
            "systemctl",
        ])?;

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
    fn try_disk_fallback(&self) -> Option<SpecialistResponseV2> {
        if !self.intent.contains("disk")
            && !self.intent.contains("storage")
            && !self.intent.contains("space")
        {
            return None;
        }

        let output = self.find_probe(&["df", "df_h", "disk_usage", "lsblk"])?;
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
                KeyFinding::new(&mount, &format!("{}% of {}", percent, size))
                    .with_severity(severity),
            );
        }

        Some(response)
    }

    /// Fallback for network questions
    fn try_network_fallback(&self) -> Option<SpecialistResponseV2> {
        if !self.intent.contains("network")
            && !self.intent.contains("interface")
            && !self.intent.contains("ip")
        {
            return None;
        }

        let output = self.find_probe(&["ip_addr", "ip_brief", "ifconfig", "interfaces"])?;
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
    fn try_swap_fallback(&self) -> Option<SpecialistResponseV2> {
        if !self.intent.contains("swap") {
            return None;
        }

        let output = self.find_probe(&["swapon", "swap", "free", "proc_swaps"])?;

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
    fn try_uptime_fallback(&self) -> Option<SpecialistResponseV2> {
        if !self.intent.contains("uptime") && !self.intent.contains("running") {
            return None;
        }

        let output = self.find_probe(&["uptime", "uptime_p", "proc_uptime"])?;

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
    fn try_boot_fallback(&self) -> Option<SpecialistResponseV2> {
        if !self.intent.contains("boot") {
            return None;
        }

        let output = self.find_probe(&["systemd_analyze", "boot_time", "systemd_blame"])?;

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

    /// Generic fallback when no specific handler matches
    fn generic_fallback(&self, reason: &str) -> SpecialistResponseV2 {
        // If we have probe data, build a generic summary
        if !self.probes.is_empty() {
            let probe_names: Vec<_> = self.probes.keys().collect();
            return SpecialistResponseV2::insufficient_evidence(&format!(
                "I had trouble processing the response. Available probe data: {}",
                probe_names
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .with_notes(reason);
        }

        SpecialistResponseV2::insufficient_evidence(
            "I couldn't collect enough data to answer this. Try again or run: anna status",
        )
        .with_notes(reason)
    }

    /// Find probe output by trying multiple possible names
    fn find_probe(&self, names: &[&str]) -> Option<String> {
        for name in names {
            if let Some(output) = self.probes.get(*name) {
                if !output.trim().is_empty() {
                    return Some(output.clone());
                }
            }
        }
        None
    }
}

// =============================================================================
// Parsing helpers
// =============================================================================

fn parse_memory_from_free(output: &str) -> Option<(u64, u64, u64)> {
    for line in output.lines() {
        if line.to_lowercase().starts_with("mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parse_size(parts[1])?;
                let used = parse_size(parts[2])?;
                let available = if parts.len() >= 7 {
                    parse_size(parts[6])? // free -h format
                } else {
                    parse_size(parts[3])? // older format
                };
                return Some((available, total, used));
            }
        }
    }
    None
}

fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Handle suffixes: K, M, G, T, Ki, Mi, Gi, Ti
    let (num_str, multiplier): (&str, u64) = if let Some(n) = s.strip_suffix("Gi") {
        (n, 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("Mi") {
        (n, 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("Ki") {
        (n, 1024)
    } else if let Some(n) = s.strip_suffix('G') {
        (n, 1_000_000_000)
    } else if let Some(n) = s.strip_suffix('M') {
        (n, 1_000_000)
    } else if let Some(n) = s.strip_suffix('K') {
        (n, 1000)
    } else if let Some(n) = s.strip_suffix('T') {
        (n, 1_000_000_000_000_u64)
    } else {
        (s, 1)
    };

    num_str
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64) as u64)
}

fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn parse_failed_services(output: &str) -> Vec<String> {
    let mut failed = vec![];
    for line in output.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("failed") && line.contains(".service") {
            // Extract service name
            for word in line.split_whitespace() {
                if word.ends_with(".service") {
                    failed.push(word.to_string());
                    break;
                }
            }
        }
    }
    failed
}

fn parse_disk_usage(output: &str) -> Vec<(String, String, u32)> {
    let mut partitions = vec![];
    for line in output.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            if let Some(percent_str) = parts.iter().find(|s| s.ends_with('%')) {
                if let Ok(percent) = percent_str.trim_end_matches('%').parse::<u32>() {
                    let mount = parts.last().unwrap_or(&"/").to_string();
                    let size = parts.get(1).unwrap_or(&"?").to_string();
                    if !mount.starts_with("/dev") && !mount.starts_with("tmpfs") {
                        partitions.push((mount, size, percent));
                    }
                }
            }
        }
    }
    partitions
}

fn parse_network_interfaces(output: &str) -> Vec<(String, String, String)> {
    let mut interfaces = vec![];
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].trim_end_matches(':').to_string();
            if name == "lo" {
                continue;
            }
            let state = if parts.len() >= 2 {
                parts[1].to_string()
            } else {
                "UNKNOWN".to_string()
            };
            let ip = parts.get(2).unwrap_or(&"").to_string();
            interfaces.push((name, state, ip));
        }
    }
    interfaces
}

fn extract_swap_size(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.to_lowercase().contains("swap") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if part.to_lowercase().contains("swap") && i + 1 < parts.len() {
                    let size = parts[i + 1];
                    if size
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        return Some(size.to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_uptime(output: &str) -> Option<String> {
    // "up X days, Y hours, Z minutes" format
    if let Some(start) = output.find("up ") {
        let rest = &output[start + 3..];
        if let Some(end) = rest.find(',') {
            return Some(rest[..end + 20.min(rest.len() - end)].trim().to_string());
        }
        return Some(
            rest.split_whitespace()
                .take(4)
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    None
}

fn extract_boot_time(output: &str) -> Option<String> {
    // Look for "Xmin Ys" or "Xs" patterns
    for line in output.lines() {
        if line.contains("startup finished") || line.contains("=") {
            for word in line.split_whitespace() {
                if word.ends_with('s') || word.ends_with("min") {
                    return Some(word.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_fallback() {
        let mut probes = HashMap::new();
        probes.insert(
            "free".to_string(),
            "Mem:           31Gi       14Gi       17Gi".to_string(),
        );

        let engine = FallbackEngine::new("show_memory", "How much free RAM?", probes);
        let result = engine.try_fallback("test");

        assert!(result.success);
        assert!(result.response.has_direct_answer());
    }

    #[test]
    fn test_services_fallback_none() {
        let mut probes = HashMap::new();
        probes.insert(
            "systemctl_failed".to_string(),
            "0 loaded units listed.".to_string(),
        );

        let engine = FallbackEngine::new("check_failed_services", "Any failed services?", probes);
        let result = engine.try_fallback("test");

        assert!(result.success);
        assert!(result.response.main_text().contains("No,"));
    }

    #[test]
    fn test_services_fallback_failed() {
        let mut probes = HashMap::new();
        probes.insert(
            "systemctl_failed".to_string(),
            "foo.service failed\nbar.service failed".to_string(),
        );

        let engine = FallbackEngine::new("check_failed_services", "Any failed services?", probes);
        let result = engine.try_fallback("test");

        assert!(result.success);
        assert!(result.response.main_text().contains("Yes,"));
        assert_eq!(result.response.key_findings.len(), 2);
    }

    #[test]
    fn test_disk_fallback() {
        let mut probes = HashMap::new();
        probes.insert(
            "df".to_string(),
            "Filesystem Size Used Avail Use% Mounted on\n/dev/sda1 100G 85G 15G 85% /".to_string(),
        );

        let engine = FallbackEngine::new("show_disk_usage", "How much disk space?", probes);
        let result = engine.try_fallback("test");

        assert!(result.success);
    }

    #[test]
    fn test_generic_fallback() {
        let probes = HashMap::new();
        let engine = FallbackEngine::new("unknown_intent", "Unknown question", probes);
        let result = engine.try_fallback("test");

        assert!(!result.success);
        assert_eq!(
            result.response.status,
            SpecialistStatus::InsufficientEvidence
        );
    }
}
