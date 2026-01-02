//! Fallback generator for creating answers from evidence (v0.0.433).

use super::extractors::{
    extract_boot_time, extract_disk_info, extract_failed_services, extract_memory_info,
};
use super::types::{truncate, FallbackAnswer, ProbeEvidence};
use crate::robustness::contract::{EvidenceRef, SpecialistResult, TicketOutcome};
use std::collections::HashMap;

/// Pattern for generating fallback from probe data.
pub(crate) struct FallbackPattern {
    pub(crate) name: String,
    pub(crate) extractor: Box<dyn Fn(&str) -> Option<String> + Send + Sync>,
    pub(crate) template: String,
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

#[cfg(test)]
mod tests {
    use super::*;

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
