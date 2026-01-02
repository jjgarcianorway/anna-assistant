//! Fallback builder for evidence-only responses.

use super::decoder::DecodeError;
use super::envelope::{EvidenceKind, ModelRole};
use super::fallback_types::{FallbackResponse, GatheredEvidence};

/// Builder for evidence-only fallback.
pub struct EvidenceFallback {
    ticket_id: String,
    failed_role: ModelRole,
    error: DecodeError,
    evidence: Vec<GatheredEvidence>,
}

impl EvidenceFallback {
    /// Create a new fallback builder.
    pub fn new(ticket_id: &str, failed_role: ModelRole, error: DecodeError) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            failed_role,
            error,
            evidence: Vec::new(),
        }
    }

    /// Add evidence.
    pub fn add_evidence(&mut self, evidence: GatheredEvidence) {
        self.evidence.push(evidence);
    }

    /// Add multiple evidence items.
    pub fn add_evidence_batch(&mut self, evidence: Vec<GatheredEvidence>) {
        self.evidence.extend(evidence);
    }

    /// Build the fallback response with deterministic probe suggestions.
    pub fn build(self) -> FallbackResponse {
        let mut response = FallbackResponse::new(&self.ticket_id, self.failed_role, &self.error);

        // Set confidence based on evidence amount
        let confidence = if self.evidence.is_empty() {
            0.1
        } else if self.evidence.len() < 3 {
            0.3
        } else {
            0.5
        };

        // Determine limitations based on what's missing
        let limitations = self.determine_limitations();

        // Suggest next probes deterministically
        let suggested_probes = self.suggest_next_probes();

        response = response
            .with_evidence(self.evidence)
            .with_limitations(limitations)
            .with_suggested_probes(suggested_probes)
            .with_confidence(confidence);

        response.build_message();
        response
    }

    /// Determine what couldn't be concluded.
    fn determine_limitations(&self) -> Vec<String> {
        let mut limitations = Vec::new();

        // Check what evidence types we're missing
        let has_probe = self.evidence.iter().any(|e| e.kind == EvidenceKind::Probe);
        let has_docs = self.evidence.iter().any(|e| {
            matches!(
                e.kind,
                EvidenceKind::Man | EvidenceKind::Help | EvidenceKind::Wiki
            )
        });

        if !has_probe {
            limitations.push("System state (no probe data collected)".to_string());
        }

        if !has_docs {
            limitations.push("Documentation context (no docs retrieved)".to_string());
        }

        // Generic limitation
        limitations.push("Root cause analysis (requires model synthesis)".to_string());

        limitations
    }

    /// Suggest next probes based on what we have.
    fn suggest_next_probes(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check what probes we already have
        let probe_ids: Vec<&str> = self
            .evidence
            .iter()
            .filter(|e| e.kind == EvidenceKind::Probe)
            .map(|e| e.id.as_str())
            .collect();

        // Deterministic suggestions based on common workflows
        if !probe_ids.iter().any(|p| p.contains("boot")) {
            // No boot probes - suggest boot analysis
            suggestions.push("sys.boot.analyze".to_string());
        }

        if !probe_ids
            .iter()
            .any(|p| p.contains("services") || p.contains("failed"))
        {
            // No service probes - suggest failed services check
            suggestions.push("sys.services.failed".to_string());
        }

        if !probe_ids
            .iter()
            .any(|p| p.contains("mem") || p.contains("memory"))
        {
            // No memory probes
            suggestions.push("sys.mem.free".to_string());
        }

        if !probe_ids
            .iter()
            .any(|p| p.contains("logs") || p.contains("errors"))
        {
            // No log probes
            suggestions.push("sys.logs.errors".to_string());
        }

        // Limit to 2 suggestions
        suggestions.truncate(2);
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_builder() {
        let error = DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        };

        let mut builder = EvidenceFallback::new("DSK-001", ModelRole::Junior, error);

        builder.add_evidence(GatheredEvidence::new(
            "ev_boot_1",
            EvidenceKind::Probe,
            "Boot Analysis",
            "Boot took 15 seconds",
        ));

        let response = builder.build();

        assert_eq!(response.ticket_id, "DSK-001");
        assert!(!response.evidence.is_empty());
        assert!(!response.message.is_empty());
        assert!(response.confidence <= super::super::fallback_types::MAX_FALLBACK_CONFIDENCE);
    }

    #[test]
    fn test_fallback_limitations() {
        let error = DecodeError::NoFrame {
            raw_output: "test".to_string(),
        };

        let builder = EvidenceFallback::new("DSK-001", ModelRole::Senior, error);
        let limitations = builder.determine_limitations();

        assert!(!limitations.is_empty());
        assert!(limitations.iter().any(|l| l.contains("probe")));
    }

    #[test]
    fn test_fallback_probe_suggestions() {
        let error = DecodeError::EmptyOutput;
        let builder = EvidenceFallback::new("DSK-001", ModelRole::Junior, error);
        let suggestions = builder.suggest_next_probes();

        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 2);
    }
}
