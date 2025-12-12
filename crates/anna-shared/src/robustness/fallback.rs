//! Evidence fallback handling (v0.0.433).
//!
//! When LLM fails but probes succeeded, provide minimal fallback answers.

use super::contract::{EvidenceRef, ProposedStep, SpecialistResult, TicketMetrics, TicketOutcome};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Evidence collected from a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    /// Probe name.
    pub name: String,
    /// Raw output.
    pub raw_output: String,
    /// Parsed values (if applicable).
    pub parsed: HashMap<String, String>,
    /// Whether the probe succeeded.
    pub success: bool,
    /// Duration in ms.
    pub duration_ms: u64,
}

impl ProbeEvidence {
    /// Create new evidence.
    pub fn new(name: &str, raw_output: &str, success: bool) -> Self {
        Self {
            name: name.to_string(),
            raw_output: raw_output.to_string(),
            parsed: HashMap::new(),
            success,
            duration_ms: 0,
        }
    }

    /// Add a parsed value.
    pub fn with_parsed(mut self, key: &str, value: &str) -> Self {
        self.parsed.insert(key.to_string(), value.to_string());
        self
    }

    /// Set duration.
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Fallback answer generated from evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackAnswer {
    /// Summary based on evidence.
    pub summary: String,
    /// Raw data snippet.
    pub raw_data: String,
    /// Confidence (lower than LLM answer).
    pub confidence: f32,
    /// Evidence sources used.
    pub sources: Vec<String>,
    /// What failed (LLM stage).
    pub failure_reason: String,
}

/// Fallback generator for different probe types.
pub struct FallbackGenerator {
    /// Known probe patterns.
    patterns: HashMap<String, FallbackPattern>,
}

impl FallbackGenerator {
    /// Create a new generator with default patterns.
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Memory probe
        patterns.insert(
            "proc_meminfo".to_string(),
            FallbackPattern {
                name: "memory".to_string(),
                extractor: Box::new(extract_memory_info),
                template: "Memory usage from /proc/meminfo: {summary}".to_string(),
            },
        );
        patterns.insert(
            "memory".to_string(),
            FallbackPattern {
                name: "memory".to_string(),
                extractor: Box::new(extract_memory_info),
                template: "Memory usage: {summary}".to_string(),
            },
        );

        // Disk probe
        patterns.insert(
            "disk_usage".to_string(),
            FallbackPattern {
                name: "disk".to_string(),
                extractor: Box::new(extract_disk_info),
                template: "Disk usage: {summary}".to_string(),
            },
        );

        // Boot time probe
        patterns.insert(
            "systemd_boot_time".to_string(),
            FallbackPattern {
                name: "boot".to_string(),
                extractor: Box::new(extract_boot_time),
                template: "Boot time analysis: {summary}".to_string(),
            },
        );

        // Failed services probe
        patterns.insert(
            "systemd_failed".to_string(),
            FallbackPattern {
                name: "services".to_string(),
                extractor: Box::new(extract_failed_services),
                template: "Failed services check: {summary}".to_string(),
            },
        );

        Self { patterns }
    }

    /// Generate fallback from collected evidence.
    pub fn generate(
        &self,
        evidence: &[ProbeEvidence],
        failure_reason: &str,
    ) -> Option<FallbackAnswer> {
        if evidence.is_empty() {
            return None;
        }

        // Find successful probes
        let successful: Vec<_> = evidence.iter().filter(|e| e.success).collect();
        if successful.is_empty() {
            return None;
        }

        // Try to find a matching pattern
        for probe in &successful {
            if let Some(pattern) = self.patterns.get(&probe.name) {
                if let Some(summary) = (pattern.extractor)(&probe.raw_output) {
                    let answer = pattern.template.replace("{summary}", &summary);
                    return Some(FallbackAnswer {
                        summary: answer,
                        raw_data: truncate(&probe.raw_output, 500),
                        confidence: 0.6, // Lower than LLM
                        sources: vec![probe.name.clone()],
                        failure_reason: failure_reason.to_string(),
                    });
                }
            }
        }

        // Generic fallback - just show raw data
        let first = &successful[0];
        Some(FallbackAnswer {
            summary: format!(
                "I collected data from {} but could not process it through my specialist.",
                first.name
            ),
            raw_data: truncate(&first.raw_output, 500),
            confidence: 0.4,
            sources: successful.iter().map(|e| e.name.clone()).collect(),
            failure_reason: failure_reason.to_string(),
        })
    }

    /// Convert fallback to SpecialistResult.
    pub fn to_result(&self, fallback: &FallbackAnswer) -> SpecialistResult {
        let mut result = SpecialistResult::partial(
            &fallback.summary,
            &format!("LLM processing failed: {}", fallback.failure_reason),
        );

        result.confidence = fallback.confidence;
        result.error_info = Some(format!(
            "Fallback answer from probes. LLM failure: {}",
            fallback.failure_reason
        ));

        for source in &fallback.sources {
            result.evidence_refs.push(EvidenceRef::new(source, "raw"));
        }

        result
    }

    /// Generate full SpecialistResult from evidence.
    pub fn generate_result(
        &self,
        evidence: &[ProbeEvidence],
        failure_reason: &str,
    ) -> SpecialistResult {
        if let Some(fallback) = self.generate(evidence, failure_reason) {
            self.to_result(&fallback)
        } else {
            SpecialistResult::internal_error(&format!(
                "No usable evidence for fallback: {}",
                failure_reason
            ))
        }
    }
}

impl Default for FallbackGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern for generating fallback from probe data.
struct FallbackPattern {
    name: String,
    extractor: Box<dyn Fn(&str) -> Option<String> + Send + Sync>,
    template: String,
}

/// Extract memory info from /proc/meminfo.
fn extract_memory_info(raw: &str) -> Option<String> {
    let mut mem_total: Option<u64> = None;
    let mut mem_available: Option<u64> = None;
    let mut mem_free: Option<u64> = None;

    for line in raw.lines() {
        if line.starts_with("MemTotal:") {
            mem_total = extract_kb_value(line);
        } else if line.starts_with("MemAvailable:") {
            mem_available = extract_kb_value(line);
        } else if line.starts_with("MemFree:") {
            mem_free = extract_kb_value(line);
        }
    }

    let total = mem_total?;
    let available = mem_available.or(mem_free)?;

    let total_gib = total as f64 / 1024.0 / 1024.0;
    let available_gib = available as f64 / 1024.0 / 1024.0;
    let percent = (available as f64 / total as f64) * 100.0;

    Some(format!(
        "{:.1} GiB available out of {:.1} GiB total ({:.0}% free)",
        available_gib, total_gib, percent
    ))
}

/// Extract kB value from /proc/meminfo line.
fn extract_kb_value(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse().ok()
    } else {
        None
    }
}

/// Extract disk info from df output.
fn extract_disk_info(raw: &str) -> Option<String> {
    // Look for root filesystem
    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 && parts[5] == "/" {
            let used = parts[2];
            let avail = parts[3];
            let percent = parts[4];
            return Some(format!(
                "Root filesystem: {} used, {} available ({})",
                used, avail, percent
            ));
        }
    }

    // Fallback: just report first line
    let first_data = raw.lines().nth(1)?;
    let parts: Vec<&str> = first_data.split_whitespace().collect();
    if parts.len() >= 5 {
        Some(format!(
            "Disk: {} used, {} available ({})",
            parts.get(2).unwrap_or(&"?"),
            parts.get(3).unwrap_or(&"?"),
            parts.get(4).unwrap_or(&"?")
        ))
    } else {
        None
    }
}

/// Extract boot time from systemd-analyze.
fn extract_boot_time(raw: &str) -> Option<String> {
    // Look for the summary line
    for line in raw.lines() {
        if line.contains("reached after") || line.contains("Startup finished") {
            return Some(line.trim().to_string());
        }
    }

    // Try to extract time values
    if raw.contains("firmware") || raw.contains("kernel") {
        Some(raw.lines().next()?.trim().to_string())
    } else {
        None
    }
}

/// Extract failed services from systemctl --failed.
fn extract_failed_services(raw: &str) -> Option<String> {
    let mut failed_units = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        // Look for .service entries marked as failed
        if line.contains(".service") && line.contains("failed") {
            if let Some(unit) = line.split_whitespace().next() {
                failed_units.push(unit.to_string());
            }
        }
    }

    if failed_units.is_empty() {
        if raw.contains("0 loaded units listed") {
            Some("No failed units found.".to_string())
        } else {
            None
        }
    } else {
        Some(format!(
            "{} failed unit(s): {}",
            failed_units.len(),
            failed_units.join(", ")
        ))
    }
}

/// Truncate string to max length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_extraction() {
        let raw = "MemTotal:       32896136 kB\n\
                   MemFree:         8234567 kB\n\
                   MemAvailable:   17825792 kB\n";

        let summary = extract_memory_info(raw).unwrap();
        assert!(summary.contains("GiB"));
        assert!(summary.contains("available"));
    }

    #[test]
    fn test_fallback_generator() {
        let generator = FallbackGenerator::new();

        let evidence = vec![ProbeEvidence::new(
            "proc_meminfo",
            "MemTotal: 32000000 kB\nMemAvailable: 16000000 kB\n",
            true,
        )];

        let fallback = generator.generate(&evidence, "LLM parse failed");
        assert!(fallback.is_some());

        let f = fallback.unwrap();
        assert!(f.summary.contains("GiB"));
        assert_eq!(f.sources, vec!["proc_meminfo"]);
    }

    #[test]
    fn test_generic_fallback() {
        let generator = FallbackGenerator::new();

        let evidence = vec![ProbeEvidence::new(
            "unknown_probe",
            "some raw data here",
            true,
        )];

        let fallback = generator.generate(&evidence, "test failure");
        assert!(fallback.is_some());
        assert!(fallback.unwrap().summary.contains("collected data"));
    }

    #[test]
    fn test_to_result() {
        let generator = FallbackGenerator::new();

        let evidence = vec![ProbeEvidence::new(
            "memory",
            "MemTotal: 16000000 kB\nMemAvailable: 8000000 kB\n",
            true,
        )];

        let result = generator.generate_result(&evidence, "parse failed");
        assert_eq!(result.outcome, TicketOutcome::Partial);
        assert!(!result.evidence_refs.is_empty());
    }
}
