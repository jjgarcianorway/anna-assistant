//! Core response types for specialist protocol.

use serde::{Deserialize, Serialize};

use super::schema_actions::ResponseActions;
use super::schema_evidence::{ResponseEvidence, ResponseMetrics, ResponseMeta};

/// The complete specialist response (strict schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictResponse {
    /// Response status: success, partial, or failure
    pub status: ResponseStatus,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,

    /// Domain classification (e.g., "services.systemd")
    pub domain: String,

    /// Intent classification (e.g., "check_failed_services")
    pub intent: String,

    /// Short, 1-3 sentence human-readable answer
    pub summary: String,

    /// Detailed breakdown
    pub details: ResponseDetails,

    /// Proposed and auto-applied actions
    #[serde(default)]
    pub actions: ResponseActions,

    /// Evidence backing all claims
    pub evidence: ResponseEvidence,

    /// Performance metrics
    #[serde(default)]
    pub metrics: ResponseMetrics,

    /// Metadata
    pub meta: ResponseMeta,
}

/// Response status with strict semantics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Clear, evidence-backed answer; question fully addressed
    Success,
    /// Partial answer; some aspects unknown (must state what is known/unknown)
    Partial,
    /// Could not provide useful answer (not an error to hide)
    Failure,
}

impl Default for ResponseStatus {
    fn default() -> Self {
        Self::Failure
    }
}

/// Detailed response breakdown
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseDetails {
    /// Facts grounded in probe output (required for success)
    #[serde(default)]
    pub key_facts: Vec<String>,

    /// Brief explanation of what is happening (grounded in evidence)
    #[serde(default)]
    pub diagnosis: Option<String>,

    /// Actionable suggestions (optional)
    #[serde(default)]
    pub recommendations: Vec<String>,
}
