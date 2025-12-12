//! Fallback Summarizer (Part D) - v0.0.440.
//!
//! If specialist fails after retries:
//! - Translator model produces a minimal answer from evidence only
//! - No speculation
//! - Prevents garbage answers (e.g., "CPU model" for "top CPU service")

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fallback response when specialist fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackResponse {
    /// Case ID.
    pub case_id: String,
    /// Short factual answer OR "insufficient evidence".
    pub answer: String,
    /// Confidence (typically lower than specialist).
    pub confidence: f64,
    /// Missing evidence that would have helped.
    #[serde(default)]
    pub missing_evidence: Vec<String>,
    /// Next probes to run if more evidence needed.
    #[serde(default)]
    pub next_probe: Vec<String>,
}

impl FallbackResponse {
    /// Create a new fallback response.
    pub fn new(case_id: &str, answer: &str, confidence: f64) -> Self {
        Self {
            case_id: case_id.to_string(),
            answer: answer.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            missing_evidence: Vec::new(),
            next_probe: Vec::new(),
        }
    }

    /// Create an "insufficient evidence" response.
    pub fn insufficient_evidence(case_id: &str, missing: Vec<&str>) -> Self {
        Self {
            case_id: case_id.to_string(),
            answer: "Insufficient evidence to answer.".to_string(),
            confidence: 0.0,
            missing_evidence: missing.into_iter().map(String::from).collect(),
            next_probe: Vec::new(),
        }
    }

    /// Add missing evidence.
    pub fn with_missing(mut self, probe_id: &str) -> Self {
        self.missing_evidence.push(probe_id.to_string());
        self
    }

    /// Add next probe.
    pub fn with_next_probe(mut self, probe_id: &str) -> Self {
        self.next_probe.push(probe_id.to_string());
        self
    }

    /// Check if this is an insufficient evidence response.
    pub fn is_insufficient(&self) -> bool {
        self.answer.contains("Insufficient evidence") || self.confidence == 0.0
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

/// Probe evidence for fallback summarizer.
#[derive(Debug, Clone)]
pub struct ProbeEvidence {
    /// Probe ID.
    pub probe_id: String,
    /// Probe output.
    pub output: String,
    /// Whether probe succeeded.
    pub success: bool,
}

impl ProbeEvidence {
    /// Create successful evidence.
    pub fn success(probe_id: &str, output: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            output: output.to_string(),
            success: true,
        }
    }

    /// Create failed evidence.
    pub fn failed(probe_id: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            output: String::new(),
            success: false,
        }
    }
}

/// Fallback summarizer that produces minimal answers from evidence.
pub struct FallbackSummarizer {
    /// Known probe-to-answer templates.
    templates: HashMap<String, AnswerTemplate>,
}

/// Template for generating answers from probe output.
#[derive(Debug, Clone)]
struct AnswerTemplate {
    /// Probe ID this template handles.
    probe_id: String,
    /// Function to extract answer from probe output.
    extractor: fn(&str) -> Option<String>,
    /// Default confidence for this template.
    default_confidence: f64,
}

impl FallbackSummarizer {
    /// Create a new summarizer with default templates.
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Memory templates
        templates.insert(
            "free_h".to_string(),
            AnswerTemplate {
                probe_id: "free_h".to_string(),
                extractor: extract_memory_from_free,
                default_confidence: 0.85,
            },
        );

        // Boot templates
        templates.insert(
            "systemd_analyze".to_string(),
            AnswerTemplate {
                probe_id: "systemd_analyze".to_string(),
                extractor: extract_boot_time,
                default_confidence: 0.9,
            },
        );

        // Disk templates
        templates.insert(
            "df_h".to_string(),
            AnswerTemplate {
                probe_id: "df_h".to_string(),
                extractor: extract_disk_usage,
                default_confidence: 0.85,
            },
        );

        // Service templates
        templates.insert(
            "systemctl_failed".to_string(),
            AnswerTemplate {
                probe_id: "systemctl_failed".to_string(),
                extractor: extract_failed_services,
                default_confidence: 0.9,
            },
        );

        // Load templates
        templates.insert(
            "uptime".to_string(),
            AnswerTemplate {
                probe_id: "uptime".to_string(),
                extractor: extract_load_average,
                default_confidence: 0.85,
            },
        );

        // GPU templates
        templates.insert(
            "lspci_gpu".to_string(),
            AnswerTemplate {
                probe_id: "lspci_gpu".to_string(),
                extractor: extract_gpu_info,
                default_confidence: 0.9,
            },
        );

        Self { templates }
    }

    /// Generate a fallback response from evidence.
    pub fn summarize(
        &self,
        case_id: &str,
        evidence: &[ProbeEvidence],
        required_probes: &[&str],
    ) -> FallbackResponse {
        // Check for missing required probes
        let mut missing = Vec::new();
        for probe_id in required_probes {
            if !evidence
                .iter()
                .any(|e| e.probe_id == *probe_id && e.success)
            {
                missing.push(*probe_id);
            }
        }

        if !missing.is_empty() {
            return FallbackResponse::insufficient_evidence(case_id, missing);
        }

        // Try to generate answer from available evidence
        for probe in evidence.iter().filter(|e| e.success) {
            if let Some(template) = self.templates.get(&probe.probe_id) {
                if let Some(answer) = (template.extractor)(&probe.output) {
                    return FallbackResponse::new(case_id, &answer, template.default_confidence);
                }
            }
        }

        // If no template matched, build a generic response
        self.build_generic_response(case_id, evidence)
    }

    /// Build a generic response listing available evidence.
    fn build_generic_response(
        &self,
        case_id: &str,
        evidence: &[ProbeEvidence],
    ) -> FallbackResponse {
        let successful: Vec<_> = evidence.iter().filter(|e| e.success).collect();

        if successful.is_empty() {
            return FallbackResponse::insufficient_evidence(case_id, vec![]);
        }

        // List what we have
        let probes: Vec<_> = successful.iter().map(|e| e.probe_id.as_str()).collect();
        let answer = format!(
            "Data collected from {} probe(s): {}. Unable to synthesize a specific answer.",
            successful.len(),
            probes.join(", ")
        );

        FallbackResponse::new(case_id, &answer, 0.3)
    }

    /// Check if evidence is sufficient for a direct answer.
    pub fn can_answer_directly(&self, evidence: &[ProbeEvidence]) -> bool {
        evidence
            .iter()
            .any(|e| e.success && self.templates.contains_key(&e.probe_id))
    }
}

impl Default for FallbackSummarizer {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Extraction functions ==========

fn extract_memory_from_free(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts.get(1)?;
                let available = parts.get(6).or(parts.get(3))?;
                return Some(format!("Memory: {} total, {} available.", total, available));
            }
        }
    }
    None
}

fn extract_boot_time(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Startup finished") {
            return Some(line.trim().to_string());
        }
    }
    output
        .lines()
        .next()
        .map(|l| format!("Boot time: {}", l.trim()))
}

fn extract_disk_usage(output: &str) -> Option<String> {
    let mut results = Vec::new();
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let mount = parts.get(5)?;
            let use_pct = parts.get(4)?;
            let avail = parts.get(3)?;
            if *mount == "/" || mount.starts_with("/home") {
                results.push(format!("{}: {} used, {} available", mount, use_pct, avail));
            }
        }
    }
    if results.is_empty() {
        None
    } else {
        Some(results.join(". "))
    }
}

fn extract_failed_services(output: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() || output.contains("0 loaded units") {
        return Some("No failed services.".to_string());
    }

    let count = lines.len().saturating_sub(1);
    if count == 0 {
        Some("No failed services.".to_string())
    } else {
        Some(format!("{} failed service(s).", count))
    }
}

fn extract_load_average(output: &str) -> Option<String> {
    if let Some(idx) = output.find("load average:") {
        return Some(output[idx..].trim().to_string());
    }
    None
}

fn extract_gpu_info(output: &str) -> Option<String> {
    let gpu_lines: Vec<&str> = output
        .lines()
        .filter(|l| l.contains("VGA") || l.contains("3D") || l.contains("Display"))
        .collect();

    if gpu_lines.is_empty() {
        Some("No GPU detected.".to_string())
    } else {
        // Extract just the device name
        Some(gpu_lines.join("; "))
    }
}

/// Context for fallback decision.
#[derive(Debug, Clone)]
pub struct FallbackContext {
    /// Case ID.
    pub case_id: String,
    /// Why fallback was triggered.
    pub reason: FallbackReason,
    /// Number of specialist attempts.
    pub attempts: usize,
    /// Total time spent on specialist calls.
    pub total_time_ms: u64,
}

/// Why fallback was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackReason {
    /// Specialist timed out.
    Timeout,
    /// Specialist response invalid.
    InvalidResponse,
    /// Max retries exhausted.
    RetriesExhausted,
    /// Specialist not available.
    Unavailable,
}

impl FallbackReason {
    /// Get label for logging.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::InvalidResponse => "invalid_response",
            Self::RetriesExhausted => "retries_exhausted",
            Self::Unavailable => "unavailable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_response_new() {
        let response = FallbackResponse::new("DSK-0101", "Boot time is 7.5 seconds.", 0.85);
        assert_eq!(response.case_id, "DSK-0101");
        assert!(!response.is_insufficient());
    }

    #[test]
    fn test_fallback_response_insufficient() {
        let response = FallbackResponse::insufficient_evidence("DSK-0101", vec!["systemd_analyze"]);
        assert!(response.is_insufficient());
        assert_eq!(response.confidence, 0.0);
    }

    #[test]
    fn test_summarizer_memory() {
        let summarizer = FallbackSummarizer::new();
        let evidence = vec![ProbeEvidence::success(
            "free_h",
            "              total        used        free      shared  buff/cache   available\nMem:           31Gi       8.2Gi        15Gi       1.2Gi       7.8Gi        21Gi",
        )];

        let response = summarizer.summarize("DSK-0101", &evidence, &["free_h"]);
        assert!(!response.is_insufficient());
        assert!(response.answer.contains("31Gi"));
    }

    #[test]
    fn test_summarizer_boot() {
        let summarizer = FallbackSummarizer::new();
        let evidence = vec![ProbeEvidence::success(
            "systemd_analyze",
            "Startup finished in 2.5s (kernel) + 5.2s (userspace) = 7.7s",
        )];

        let response = summarizer.summarize("DSK-0101", &evidence, &["systemd_analyze"]);
        assert!(!response.is_insufficient());
        assert!(response.answer.contains("7.7s"));
    }

    #[test]
    fn test_summarizer_missing_evidence() {
        let summarizer = FallbackSummarizer::new();
        let evidence = vec![ProbeEvidence::failed("systemd_analyze")];

        let response = summarizer.summarize("DSK-0101", &evidence, &["systemd_analyze"]);
        assert!(response.is_insufficient());
        assert!(response
            .missing_evidence
            .contains(&"systemd_analyze".to_string()));
    }

    #[test]
    fn test_extract_failed_services() {
        assert_eq!(
            extract_failed_services(""),
            Some("No failed services.".to_string())
        );

        assert_eq!(
            extract_failed_services("0 loaded units"),
            Some("No failed services.".to_string())
        );
    }

    #[test]
    fn test_fallback_reason() {
        assert_eq!(FallbackReason::Timeout.label(), "timeout");
        assert_eq!(
            FallbackReason::RetriesExhausted.label(),
            "retries_exhausted"
        );
    }
}
