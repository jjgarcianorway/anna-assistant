//! Strict specialist response schema (v0.0.428).
//!
//! All specialists must respond with this exact JSON structure.
//! No extra fields, no freeform text outside designated areas.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl StrictResponse {
    /// Create a successful response
    pub fn success(
        domain: &str,
        intent: &str,
        summary: &str,
        key_facts: Vec<String>,
        probes: Vec<ProbeEvidence>,
        meta: ResponseMeta,
    ) -> Self {
        Self {
            status: ResponseStatus::Success,
            confidence: 0.9,
            domain: domain.to_string(),
            intent: intent.to_string(),
            summary: summary.to_string(),
            details: ResponseDetails {
                key_facts,
                diagnosis: None,
                recommendations: vec![],
            },
            actions: ResponseActions::default(),
            evidence: ResponseEvidence {
                probes_used: probes,
                arch_wiki_pages: vec![],
                man_pages: vec![],
                help_commands: vec![],
            },
            metrics: ResponseMetrics::default(),
            meta,
        }
    }

    /// Create a partial response
    pub fn partial(
        domain: &str,
        intent: &str,
        summary: &str,
        known_facts: Vec<String>,
        unknown_reason: &str,
        probes: Vec<ProbeEvidence>,
        meta: ResponseMeta,
    ) -> Self {
        Self {
            status: ResponseStatus::Partial,
            confidence: 0.5,
            domain: domain.to_string(),
            intent: intent.to_string(),
            summary: summary.to_string(),
            details: ResponseDetails {
                key_facts: known_facts,
                diagnosis: Some(unknown_reason.to_string()),
                recommendations: vec![],
            },
            actions: ResponseActions::default(),
            evidence: ResponseEvidence {
                probes_used: probes,
                arch_wiki_pages: vec![],
                man_pages: vec![],
                help_commands: vec![],
            },
            metrics: ResponseMetrics::default(),
            meta,
        }
    }

    /// Create a failure response
    pub fn failure(domain: &str, intent: &str, reason: &str, meta: ResponseMeta) -> Self {
        Self {
            status: ResponseStatus::Failure,
            confidence: 0.0,
            domain: domain.to_string(),
            intent: intent.to_string(),
            summary: reason.to_string(),
            details: ResponseDetails::default(),
            actions: ResponseActions::default(),
            evidence: ResponseEvidence::default(),
            metrics: ResponseMetrics::default(),
            meta,
        }
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.metrics.latency_ms = latency_ms;
        self
    }

    /// Add recommendations
    pub fn with_recommendations(mut self, recs: Vec<String>) -> Self {
        self.details.recommendations = recs;
        self
    }

    /// Add proposed action
    pub fn with_action(mut self, action: ProposedAction) -> Self {
        self.actions.proposed.push(action);
        self
    }

    /// Check if response is learnable (high confidence success)
    pub fn is_learnable(&self) -> bool {
        self.status == ResponseStatus::Success
            && self.confidence >= super::MIN_LEARN_CONFIDENCE
            && !self.evidence.probes_used.is_empty()
    }

    /// Check if actions can be suggested
    pub fn can_suggest_actions(&self) -> bool {
        self.confidence >= super::MIN_ACTION_CONFIDENCE
            && matches!(self.status, ResponseStatus::Success | ResponseStatus::Partial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta(ticket_id: &str) -> ResponseMeta {
        ResponseMeta {
            handled_by: "Test".to_string(),
            ticket_id: ticket_id.to_string(),
            version: 1,
        }
    }

    #[test]
    fn test_success_response() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed systemd services.",
            vec!["0 failed units".to_string()],
            vec![ProbeEvidence {
                id: "systemctl_failed".to_string(),
                summary: "No failed units found".to_string(),
                raw_reference: None,
            }],
            make_meta("DSK-001"),
        );

        assert_eq!(response.status, ResponseStatus::Success);
        assert!(response.is_learnable());
    }

    #[test]
    fn test_partial_response() {
        let response = StrictResponse::partial(
            "storage.disk",
            "check_disk_usage",
            "Root filesystem is at 97% used.",
            vec!["30 GiB free out of 803 GiB".to_string()],
            "Could not identify which directories are using space (analysis timed out).",
            vec![],
            make_meta("DSK-002"),
        );

        assert_eq!(response.status, ResponseStatus::Partial);
        assert!(!response.is_learnable());
    }

    #[test]
    fn test_failure_response() {
        let response = StrictResponse::failure(
            "system",
            "unknown",
            "Could not determine the answer.",
            make_meta("DSK-003"),
        );

        assert_eq!(response.status, ResponseStatus::Failure);
        assert_eq!(response.confidence, 0.0);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
    }

    #[test]
    fn test_serialize_response() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed services.",
            vec!["All services healthy".to_string()],
            vec![],
            make_meta("DSK-004"),
        );

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"domain\":\"services.systemd\""));
    }
}
