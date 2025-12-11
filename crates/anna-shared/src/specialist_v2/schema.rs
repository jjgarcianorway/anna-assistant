//! Specialist V2 response schema (v0.0.421).
//!
//! This is the SINGLE canonical schema that ALL specialists must return.
//! No freeform text blobs - everything maps to structured fields.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::answer::{DirectAnswer, KeyFinding, RecommendedAction};

/// The complete response from a specialist - SINGLE canonical schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResponseV2 {
    /// Response status: ok, insufficient_evidence, error
    pub status: SpecialistStatus,

    /// Confidence score (0.0 - 1.0), clamped on parse
    #[serde(default)]
    pub confidence: f32,

    /// Direct answer for factual questions
    #[serde(default)]
    pub direct_answer: Option<DirectAnswer>,

    /// Key findings from evidence
    #[serde(default)]
    pub key_findings: Vec<KeyFinding>,

    /// Recommended actions for the user
    #[serde(default)]
    pub recommended_actions: Vec<RecommendedAction>,

    /// Citations: probe IDs, man pages, wiki references
    #[serde(default)]
    pub citations: Vec<String>,

    /// Short human-readable extra info (NOT an essay)
    #[serde(default)]
    pub notes: Option<String>,
}

/// Specialist response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistStatus {
    /// Everything worked, answer is complete
    #[default]
    Ok,
    /// Not enough data to answer confidently
    InsufficientEvidence,
    /// Something went wrong
    Error,
}

impl SpecialistResponseV2 {
    /// Create a new response with ok status
    pub fn ok() -> Self {
        Self {
            status: SpecialistStatus::Ok,
            confidence: 0.8,
            direct_answer: None,
            key_findings: vec![],
            recommended_actions: vec![],
            citations: vec![],
            notes: None,
        }
    }

    /// Create an insufficient evidence response
    pub fn insufficient_evidence(notes: &str) -> Self {
        Self {
            status: SpecialistStatus::InsufficientEvidence,
            confidence: 0.2,
            direct_answer: None,
            key_findings: vec![],
            recommended_actions: vec![],
            citations: vec![],
            notes: Some(notes.to_string()),
        }
    }

    /// Create an error response
    pub fn error(notes: &str) -> Self {
        Self {
            status: SpecialistStatus::Error,
            confidence: 0.0,
            direct_answer: None,
            key_findings: vec![],
            recommended_actions: vec![],
            citations: vec![],
            notes: Some(notes.to_string()),
        }
    }

    /// Builder: set direct answer
    pub fn with_direct_answer(mut self, answer: DirectAnswer) -> Self {
        self.direct_answer = Some(answer);
        self
    }

    /// Builder: add a key finding
    pub fn with_finding(mut self, finding: KeyFinding) -> Self {
        self.key_findings.push(finding);
        self
    }

    /// Builder: add findings
    pub fn with_findings(mut self, findings: Vec<KeyFinding>) -> Self {
        self.key_findings.extend(findings);
        self
    }

    /// Builder: add a recommended action
    pub fn with_action(mut self, action: RecommendedAction) -> Self {
        self.recommended_actions.push(action);
        self
    }

    /// Builder: set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Builder: add a citation
    pub fn with_citation(mut self, citation: &str) -> Self {
        self.citations.push(citation.to_string());
        self
    }

    /// Builder: set notes
    pub fn with_notes(mut self, notes: &str) -> Self {
        self.notes = Some(notes.to_string());
        self
    }

    /// Clamp confidence to valid range
    pub fn clamp_confidence(&mut self) {
        self.confidence = self.confidence.clamp(0.0, 1.0);
    }

    /// Check if this response has a usable direct answer
    pub fn has_direct_answer(&self) -> bool {
        self.direct_answer
            .as_ref()
            .map(|a| !a.short_text.is_empty())
            .unwrap_or(false)
    }

    /// Get the main answer text (direct_answer.short_text or notes)
    pub fn main_text(&self) -> &str {
        self.direct_answer
            .as_ref()
            .map(|a| a.short_text.as_str())
            .unwrap_or_else(|| self.notes.as_deref().unwrap_or("No answer available."))
    }

    /// Check if this is a successful response with content
    pub fn is_successful(&self) -> bool {
        self.status == SpecialistStatus::Ok && self.has_direct_answer()
    }
}

impl Default for SpecialistResponseV2 {
    fn default() -> Self {
        Self::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_v2::answer::DirectAnswer;

    #[test]
    fn test_response_builder() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple("17.0 GiB available"))
            .with_confidence(0.95)
            .with_citation("probe:free")
            .with_citation("man:free(1)");

        assert_eq!(response.status, SpecialistStatus::Ok);
        assert!(response.has_direct_answer());
        assert_eq!(response.citations.len(), 2);
    }

    #[test]
    fn test_confidence_clamping() {
        let mut response = SpecialistResponseV2::ok().with_confidence(1.5);
        response.clamp_confidence();
        assert_eq!(response.confidence, 1.0);

        let mut response2 = SpecialistResponseV2::ok().with_confidence(-0.5);
        response2.clamp_confidence();
        assert_eq!(response2.confidence, 0.0);
    }

    #[test]
    fn test_insufficient_evidence() {
        let response = SpecialistResponseV2::insufficient_evidence("Missing probe data");
        assert_eq!(response.status, SpecialistStatus::InsufficientEvidence);
        assert!(!response.is_successful());
    }
}
