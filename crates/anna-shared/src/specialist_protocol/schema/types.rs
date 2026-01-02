//! Core types for strict specialist response schema (v0.0.428).
//!
//! All specialists must respond with this exact JSON structure.
//! No extra fields, no freeform text outside designated areas.

use serde::{Deserialize, Serialize};

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

/// Actions that can be taken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseActions {
    /// Actions that Anna COULD run if user confirms
    #[serde(default)]
    pub proposed: Vec<ProposedAction>,

    /// Actions already applied (rare, low-risk only)
    #[serde(default)]
    pub auto_applied: Vec<AppliedAction>,
}

/// An action that can be proposed to the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Unique action ID
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// Risk level
    pub risk: RiskLevel,

    /// Whether sudo is required
    #[serde(default)]
    pub requires_sudo: bool,

    /// Commands to execute
    #[serde(default)]
    pub commands: Vec<String>,
}

/// An action that was auto-applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedAction {
    /// Action ID
    pub id: String,

    /// What was done
    pub description: String,

    /// Result
    pub result: String,
}

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

impl std::cmp::PartialOrd for RiskLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for RiskLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_rank = match self {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
        };
        let other_rank = match other {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
        };
        self_rank.cmp(&other_rank)
    }
}

/// Evidence backing up claims
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseEvidence {
    /// Probes used and their results
    #[serde(default)]
    pub probes_used: Vec<ProbeEvidence>,

    /// Arch Wiki pages referenced
    #[serde(default)]
    pub arch_wiki_pages: Vec<String>,

    /// Man pages referenced
    #[serde(default)]
    pub man_pages: Vec<String>,

    /// Help commands referenced
    #[serde(default)]
    pub help_commands: Vec<String>,
}

/// Evidence from a probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    /// Probe ID
    pub id: String,

    /// Short summary of what was found
    pub summary: String,

    /// Reference to raw probe output (hash or ID)
    #[serde(default)]
    pub raw_reference: Option<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseMetrics {
    /// Total latency in milliseconds
    #[serde(default)]
    pub latency_ms: u64,

    /// Input tokens (if LLM used)
    #[serde(default)]
    pub tokens_in: u32,

    /// Output tokens (if LLM used)
    #[serde(default)]
    pub tokens_out: u32,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// Who handled this (e.g., "Sofia (Desktop Administrator)")
    pub handled_by: String,

    /// Ticket ID
    pub ticket_id: String,

    /// Schema version
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self {
            handled_by: "System".to_string(),
            ticket_id: String::new(),
            version: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
    }
}
