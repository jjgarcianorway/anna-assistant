//! Evidence-Only Fallback Types (Part E) - v0.0.436.
//!
//! Core types for fallback mode when model calls fail.

use super::super::decoder::DecodeError;
use super::super::envelope::{Action, EvidenceKind, EvidenceRef, ModelRole};
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
