//! Fallback summarizer - v0.0.440.
//!
//! Produces minimal answers from evidence only when specialist fails.

use super::extractors::*;
use super::types::{FallbackResponse, ProbeEvidence};
use std::collections::HashMap;

/// Template for generating answers from probe output.
#[derive(Debug, Clone)]
pub struct AnswerTemplate {
    /// Probe ID this template handles.
    pub probe_id: String,
    /// Function to extract answer from probe output.
    pub extractor: fn(&str) -> Option<String>,
    /// Default confidence for this template.
    pub default_confidence: f64,
}

/// Fallback summarizer that produces minimal answers from evidence.
pub struct FallbackSummarizer {
    /// Known probe-to-answer templates.
    templates: HashMap<String, AnswerTemplate>,
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
