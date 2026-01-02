//! Core types for strict specialist contract
//!
//! This module defines all the data structures used in the specialist response schema.

use serde::{Deserialize, Serialize};

/// The STRICT response schema for all specialists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictSpecialistResponse {
    /// Ticket ID (echoed back)
    pub ticket_id: String,

    /// Intent classification
    pub intent: String,

    /// Response status
    pub status: StrictStatus,

    /// Confidence (0.0-1.0)
    pub confidence: f32,

    /// Short one-line answer (MAX 100 chars)
    pub summary: String,

    /// Optional bullet details (0-5 items, each MAX 200 chars)
    #[serde(default)]
    pub details: Vec<String>,

    /// Domain-specific structured metrics
    #[serde(default)]
    pub metrics: Option<serde_json::Value>,

    /// Suggested actions (0-3 items)
    #[serde(default)]
    pub actions: Vec<SuggestedAction>,

    /// Evidence from probes (REQUIRED if status=ok)
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,

    /// Citations from docs (if docs were consulted)
    #[serde(default)]
    pub citations: Vec<Citation>,
}

/// Response status - STRICT enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrictStatus {
    /// Answer is complete and grounded
    Ok,
    /// Answer is partial - some data missing
    Partial,
    /// Cannot answer - insufficient data or error
    Failed,
}

/// A suggested action for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action kind
    pub kind: ActionKind,
    /// One-sentence description
    pub description: String,
    /// Optional shell command (safe, generic)
    #[serde(default)]
    pub command: Option<String>,
    /// Risk level
    #[serde(default)]
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    #[default]
    Suggestion,
    Fix,
    Investigate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

/// Evidence from a probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Probe ID
    pub probe: String,
    /// Short summary of what this probe shows
    pub summary: String,
}

/// Citation from documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Document ID (e.g., "man:systemctl")
    pub doc_id: String,
    /// Source kind
    pub kind: CitationKind,
    /// Display string for user
    pub display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationKind {
    ManPage,
    ArchWiki,
    HelpOutput,
    BuiltIn,
    LearnedRecipe,
}

impl StrictSpecialistResponse {
    /// Create a successful response
    pub fn ok(ticket_id: &str, intent: &str, summary: &str, confidence: f32) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            intent: intent.to_string(),
            status: StrictStatus::Ok,
            confidence: confidence.clamp(0.0, 1.0),
            summary: summary.to_string(),
            details: vec![],
            metrics: None,
            actions: vec![],
            evidence: vec![],
            citations: vec![],
        }
    }

    /// Create a partial response
    pub fn partial(ticket_id: &str, intent: &str, summary: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            intent: intent.to_string(),
            status: StrictStatus::Partial,
            confidence: 0.0,
            summary: summary.to_string(),
            details: vec![],
            metrics: None,
            actions: vec![],
            evidence: vec![],
            citations: vec![],
        }
    }

    /// Create a failed response
    pub fn failed(ticket_id: &str, intent: &str, reason: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            intent: intent.to_string(),
            status: StrictStatus::Failed,
            confidence: 0.0,
            summary: reason.to_string(),
            details: vec![],
            metrics: None,
            actions: vec![],
            evidence: vec![],
            citations: vec![],
        }
    }

    /// Create from timeout
    pub fn timeout(ticket_id: &str, intent: &str, elapsed_secs: u64) -> Self {
        Self::failed(
            ticket_id,
            intent,
            &format!("Specialist timed out after {}s.", elapsed_secs),
        )
    }

    /// Create from parse error
    pub fn parse_error(ticket_id: &str, intent: &str, error: &str) -> Self {
        Self::failed(
            ticket_id,
            intent,
            &format!("Invalid specialist response: {}", truncate(error, 100)),
        )
    }

    /// Builder: add evidence
    pub fn with_evidence(mut self, probe: &str, summary: &str) -> Self {
        self.evidence.push(EvidenceItem {
            probe: probe.to_string(),
            summary: summary.to_string(),
        });
        self
    }

    /// Builder: add metrics
    pub fn with_metrics(mut self, metrics: serde_json::Value) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Builder: add action
    pub fn with_action(
        mut self,
        kind: ActionKind,
        desc: &str,
        cmd: Option<&str>,
        risk: RiskLevel,
    ) -> Self {
        self.actions.push(SuggestedAction {
            kind,
            description: desc.to_string(),
            command: cmd.map(|s| s.to_string()),
            risk,
        });
        self
    }
}

/// Parse result with error classification
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Successfully parsed and validated
    Success(StrictSpecialistResponse),
    /// No JSON found in output
    NoJson { raw: String },
    /// JSON found but invalid structure
    InvalidJson { raw: String, error: String },
    /// Schema validation failed
    ValidationFailed {
        response: StrictSpecialistResponse,
        issues: Vec<String>,
    },
    /// LLM timed out
    Timeout { elapsed_secs: u64 },
}

impl ParseResult {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Convert to StrictSpecialistResponse (creates error response if not success)
    pub fn to_response(self, ticket_id: &str, intent: &str) -> StrictSpecialistResponse {
        match self {
            Self::Success(r) => r,
            Self::NoJson { raw } => StrictSpecialistResponse::parse_error(
                ticket_id,
                intent,
                &format!("No JSON found: {}", truncate(&raw, 100)),
            ),
            Self::InvalidJson { error, .. } => {
                StrictSpecialistResponse::parse_error(ticket_id, intent, &error)
            }
            Self::ValidationFailed {
                mut response,
                issues,
            } => {
                // Downgrade to failed with issues noted
                response.status = StrictStatus::Failed;
                response.summary = format!("Invalid response: {}", issues.join(", "));
                response.confidence = 0.0;
                response
            }
            Self::Timeout { elapsed_secs } => {
                StrictSpecialistResponse::timeout(ticket_id, intent, elapsed_secs)
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
