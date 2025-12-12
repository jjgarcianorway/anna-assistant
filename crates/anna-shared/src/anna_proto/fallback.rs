//! Evidence-Only Fallback Mode (Part E) - v0.0.436.
//!
//! When a model call fails (timeout, parse failure, crash):
//! - Render gathered evidence in compact form
//! - State what couldn't be concluded without synthesis
//! - Propose next 1-2 probes deterministically
//! - Never claim confidence > 0.5 without synthesis

use super::decoder::DecodeError;
use super::envelope::{Action, EvidenceKind, EvidenceRef, ModelRole};
use serde::{Deserialize, Serialize};

/// Maximum confidence for fallback responses.
pub const MAX_FALLBACK_CONFIDENCE: f64 = 0.5;

/// Evidence gathered before model failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatheredEvidence {
    /// Evidence ID.
    pub id: String,
    /// Kind of evidence.
    pub kind: EvidenceKind,
    /// Human-readable title.
    pub title: String,
    /// Brief summary of content.
    pub summary: String,
    /// Raw content (truncated).
    pub content_preview: String,
}

impl GatheredEvidence {
    /// Create new gathered evidence.
    pub fn new(id: &str, kind: EvidenceKind, title: &str, summary: &str) -> Self {
        Self {
            id: id.to_string(),
            kind,
            title: title.to_string(),
            summary: summary.to_string(),
            content_preview: String::new(),
        }
    }

    /// Add content preview.
    pub fn with_preview(mut self, content: &str, max_len: usize) -> Self {
        self.content_preview = if content.len() > max_len {
            format!("{}...", &content[..max_len])
        } else {
            content.to_string()
        };
        self
    }

    /// Convert to evidence reference.
    pub fn to_ref(&self) -> EvidenceRef {
        EvidenceRef::new(&self.id, self.kind, &self.title)
    }
}

/// Fallback response when model fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackResponse {
    /// Ticket ID.
    pub ticket_id: String,
    /// Why fallback was triggered.
    pub failure_reason: String,
    /// Which model failed.
    pub failed_role: ModelRole,
    /// Evidence that was gathered.
    pub evidence: Vec<GatheredEvidence>,
    /// What couldn't be determined.
    pub limitations: Vec<String>,
    /// Suggested next probes.
    pub suggested_probes: Vec<String>,
    /// Rendered message for user.
    pub message: String,
    /// Confidence (capped at 0.5).
    pub confidence: f64,
}

impl FallbackResponse {
    /// Create a fallback response.
    pub fn new(ticket_id: &str, failed_role: ModelRole, error: &DecodeError) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            failure_reason: error.message(),
            failed_role,
            evidence: Vec::new(),
            limitations: Vec::new(),
            suggested_probes: Vec::new(),
            message: String::new(),
            confidence: 0.0,
        }
    }

    /// Add gathered evidence.
    pub fn with_evidence(mut self, evidence: Vec<GatheredEvidence>) -> Self {
        self.evidence = evidence;
        self
    }

    /// Add limitations.
    pub fn with_limitations(mut self, limitations: Vec<String>) -> Self {
        self.limitations = limitations;
        self
    }

    /// Add suggested probes.
    pub fn with_suggested_probes(mut self, probes: Vec<String>) -> Self {
        self.suggested_probes = probes;
        self
    }

    /// Set confidence (capped at MAX_FALLBACK_CONFIDENCE).
    pub fn with_confidence(mut self, conf: f64) -> Self {
        self.confidence = conf.min(MAX_FALLBACK_CONFIDENCE);
        self
    }

    /// Build the user-facing message.
    pub fn build_message(&mut self) {
        let mut parts = Vec::new();

        // Evidence summary
        if !self.evidence.is_empty() {
            parts.push("**Evidence collected:**".to_string());
            for ev in &self.evidence {
                parts.push(format!(
                    "- {} ({}): {}",
                    ev.title,
                    ev.kind_label(),
                    ev.summary
                ));
            }
            parts.push(String::new());
        }

        // Limitations
        if !self.limitations.is_empty() {
            parts.push("**Could not determine:**".to_string());
            for lim in &self.limitations {
                parts.push(format!("- {}", lim));
            }
            parts.push(String::new());
        }

        // Failure reason
        parts.push(format!(
            "*Note: The {} model {} Unable to synthesize full analysis.*",
            self.failed_role.label(),
            if self.failure_reason.contains("timed out") {
                "timed out."
            } else {
                "encountered an error."
            }
        ));

        // Suggested actions
        if !self.suggested_probes.is_empty() {
            parts.push(String::new());
            parts.push("**Suggested next steps:**".to_string());
            for probe in &self.suggested_probes {
                parts.push(format!("- Run probe: `{}`", probe));
            }
        }

        self.message = parts.join("\n");
    }

    /// Get suggested probe actions.
    pub fn probe_actions(&self) -> Vec<Action> {
        self.suggested_probes
            .iter()
            .map(|p| Action::probe(p))
            .collect()
    }
}

impl GatheredEvidence {
    /// Kind label for display.
    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            EvidenceKind::Probe => "probe",
            EvidenceKind::Man => "man page",
            EvidenceKind::Help => "help",
            EvidenceKind::Wiki => "wiki",
        }
    }
}

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
    fn test_gathered_evidence() {
        let ev = GatheredEvidence::new(
            "ev_boot_1",
            EvidenceKind::Probe,
            "Boot Analysis",
            "Boot took 15 seconds",
        )
        .with_preview("Full boot output here...", 20);

        assert_eq!(ev.kind_label(), "probe");
        assert!(ev.content_preview.len() <= 23); // 20 + "..."
    }

    #[test]
    fn test_fallback_response_confidence_cap() {
        let error = DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        };

        let response =
            FallbackResponse::new("DSK-001", ModelRole::Junior, &error).with_confidence(0.9); // Try to set high confidence

        assert!(response.confidence <= MAX_FALLBACK_CONFIDENCE);
    }

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
        assert!(response.confidence <= MAX_FALLBACK_CONFIDENCE);
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

    #[test]
    fn test_fallback_message_build() {
        let error = DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        };

        let mut response = FallbackResponse::new("DSK-001", ModelRole::Junior, &error);
        response = response
            .with_evidence(vec![GatheredEvidence::new(
                "ev_1",
                EvidenceKind::Probe,
                "Test Probe",
                "Test summary",
            )])
            .with_limitations(vec!["Root cause".to_string()])
            .with_suggested_probes(vec!["sys.boot.analyze".to_string()]);

        response.build_message();

        assert!(response.message.contains("Evidence collected"));
        assert!(response.message.contains("Test Probe"));
        assert!(response.message.contains("timed out"));
        assert!(response.message.contains("sys.boot.analyze"));
    }

    #[test]
    fn test_probe_actions() {
        let error = DecodeError::EmptyOutput;
        let response = FallbackResponse::new("DSK-001", ModelRole::Junior, &error)
            .with_suggested_probes(vec!["sys.boot.analyze".to_string()]);

        let actions = response.probe_actions();
        assert_eq!(actions.len(), 1);
    }
}
