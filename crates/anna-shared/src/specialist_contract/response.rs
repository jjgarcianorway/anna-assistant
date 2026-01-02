//! SpecialistResponse and builder methods.

use serde::{Deserialize, Serialize};

use super::citation::KnowledgeCitation;
use super::discovery::Discovery;
use super::types::{Answer, Evidence, Mood, NextSteps, ResponseStatus, Severity, StaffView, UserAction};

/// The complete response from a specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResponse {
    pub ticket_id: String,
    pub status: ResponseStatus,
    pub answer: Answer,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub staff_view: Option<StaffView>,
    #[serde(default)]
    pub next_steps: Option<NextSteps>,
    #[serde(default)]
    pub discovery: Option<Discovery>,
    /// If status is needs_more_data, which probes are missing
    #[serde(default)]
    pub missing_probes: Vec<String>,
    /// v0.0.408: Explicit "I cannot answer this" flag
    #[serde(default = "default_can_answer")]
    pub can_answer: bool,
    /// v0.0.408: IDs of knowledge items referenced by the answer
    #[serde(default)]
    pub evidence_references: Vec<String>,
    /// v0.0.408: Short titles of knowledge items used (for display)
    #[serde(default)]
    pub knowledge_used: Vec<String>,
    /// v0.0.419: Citations with full provenance from knowledge sources
    #[serde(default)]
    pub citations: Vec<KnowledgeCitation>,
}

fn default_can_answer() -> bool {
    true
}

impl SpecialistResponse {
    /// v0.0.409: Validate response for forbidden patterns and invalid data
    pub fn validate(&self) -> Vec<String> {
        let mut errors = vec![];

        // Check for forbidden patterns in answer
        let forbidden = [
            "unknown is installed",
            "unknown is not installed",
            "**unknown**",
            "2 is installed", // Common parse bug
            "1 is installed",
        ];

        let answer_lower = self.answer.short.to_lowercase();
        let detail_lower = self
            .answer
            .detail
            .as_ref()
            .map(|d| d.to_lowercase())
            .unwrap_or_default();

        for f in forbidden {
            if answer_lower.contains(f) || detail_lower.contains(f) {
                errors.push(format!("Answer contains forbidden pattern: '{}'", f));
            }
        }

        // Check confidence vs evidence consistency
        if self.status == ResponseStatus::Ok {
            if self.confidence > 0.8 && self.evidence.is_empty() {
                errors.push("High confidence (>0.8) but no evidence provided".to_string());
            }
            if self.can_answer && self.answer.short.is_empty() {
                errors.push("can_answer=true but answer.short is empty".to_string());
            }
        }

        // Check for can_answer consistency
        if !self.can_answer && self.status == ResponseStatus::Ok && self.confidence > 0.7 {
            errors.push("can_answer=false but status=ok with high confidence".to_string());
        }

        // Check confidence range
        if self.confidence < 0.0 || self.confidence > 1.0 {
            errors.push(format!(
                "Confidence {} out of range [0.0, 1.0]",
                self.confidence
            ));
        }

        errors
    }

    /// v0.0.409: Check if this is a valid, meaningful response
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Create a fallback response when JSON parsing fails
    pub fn parse_error(ticket_id: &str, error: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Error,
            answer: Answer {
                short: "Failed to parse specialist response.".to_string(),
                detail: Some(format!("Parse error: {}", error)),
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.0,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Unknown,
                mood: Mood::Blocked,
                short_note: Some("JSON parse failed".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: false,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        }
    }

    /// Create a response from deterministic/direct answer
    pub fn from_direct_answer(
        ticket_id: &str,
        answer: &str,
        evidence_probe: &str,
        evidence_snippet: &str,
    ) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: answer.to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: evidence_probe.to_string(),
                snippet: evidence_snippet.to_string(),
                interpretation: "Directly extracted from probe output.".to_string(),
            }],
            confidence: 0.95,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Info,
                mood: Mood::Confident,
                short_note: Some("Direct answer from probe data.".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec![evidence_probe.to_string()],
            knowledge_used: vec![evidence_probe.to_string()],
            citations: vec![],
        }
    }

    /// v0.0.408: Create a "cannot answer" response with suggestions
    pub fn no_evidence(ticket_id: &str, reason: &str, suggestions: Vec<String>) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::NoEvidence,
            answer: Answer {
                short: "I cannot safely answer this from local data.".to_string(),
                detail: Some(reason.to_string()),
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.0,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Info,
                mood: Mood::Uncertain,
                short_note: Some("No evidence found".to_string()),
                complexity: 1,
            }),
            next_steps: Some(NextSteps {
                user_actions: suggestions
                    .into_iter()
                    .enumerate()
                    .map(|(i, s)| UserAction {
                        id: format!("suggest_{}", i),
                        description: s,
                        recipe_id: None,
                    })
                    .collect(),
                internal_actions: vec![],
            }),
            discovery: None,
            missing_probes: vec![],
            can_answer: false,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        }
    }

    /// v0.0.410: Create response from instant answer (knowledge index hit)
    pub fn instant_answer(ticket_id: &str, answer: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: answer.to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: "knowledge_index".to_string(),
                snippet: "Learned from previous successful answer".to_string(),
                interpretation: "This pattern was previously verified and stored.".to_string(),
            }],
            confidence: 0.85,
            staff_view: Some(StaffView {
                assignee_role: "Knowledge Index".to_string(),
                severity: Severity::Info,
                mood: Mood::Confident,
                short_note: Some("Instant answer from learned pattern".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec!["knowledge_index".to_string()],
            knowledge_used: vec!["learned_pattern".to_string()],
            citations: vec![],
        }
    }

    /// v0.0.419: Create response with citations
    pub fn with_citations(mut self, citations: Vec<KnowledgeCitation>) -> Self {
        self.citations = citations;
        self
    }

    /// v0.0.419: Add a single citation
    pub fn add_citation(&mut self, citation: KnowledgeCitation) {
        self.citations.push(citation);
    }
}
