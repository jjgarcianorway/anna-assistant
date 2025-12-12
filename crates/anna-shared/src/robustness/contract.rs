//! Strict specialist contract (v0.0.433).
//!
//! All specialist responses must follow this contract.
//! No freeform text allowed - only structured results.

use serde::{Deserialize, Serialize};

/// Outcome of a ticket processing attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TicketOutcome {
    /// Ticket fully resolved with a good answer.
    Success,
    /// Ticket partially resolved - some info missing or uncertain.
    Partial,
    /// Need more information from the user before proceeding.
    ClarificationRequired,
    /// This type of request is not supported.
    Unsupported,
    /// Internal error during processing.
    InternalError,
    /// Processing exceeded time budget.
    Timeout,
    /// Failed to parse LLM response.
    ParseError,
}

impl TicketOutcome {
    /// Whether this outcome counts as a success for stats.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Whether this outcome counts as resolved (success or partial).
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Success | Self::Partial)
    }

    /// Whether this outcome is a failure for stats.
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::Unsupported | Self::InternalError | Self::Timeout | Self::ParseError
        )
    }

    /// Whether this outcome needs follow-up.
    pub fn needs_followup(&self) -> bool {
        matches!(self, Self::ClarificationRequired)
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Partial => "Partial",
            Self::ClarificationRequired => "Clarification Required",
            Self::Unsupported => "Unsupported",
            Self::InternalError => "Internal Error",
            Self::Timeout => "Timeout",
            Self::ParseError => "Parse Error",
        }
    }
}

impl Default for TicketOutcome {
    fn default() -> Self {
        Self::InternalError
    }
}

/// Category of proposed step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepCategory {
    /// Diagnostic command to gather more info.
    Diagnostic,
    /// Command that fixes the issue.
    Fix,
    /// Cleanup or maintenance command.
    Cleanup,
    /// Informational - no command needed.
    Info,
}

impl Default for StepCategory {
    fn default() -> Self {
        Self::Info
    }
}

/// A proposed step the user or Anna may take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedStep {
    /// Human-readable description.
    pub description: String,
    /// Command to execute (if any).
    pub command: String,
    /// Whether the command requires sudo.
    pub needs_sudo: bool,
    /// Category of this step.
    pub category: StepCategory,
}

impl ProposedStep {
    /// Create a diagnostic step.
    pub fn diagnostic(description: &str, command: &str) -> Self {
        Self {
            description: description.to_string(),
            command: command.to_string(),
            needs_sudo: false,
            category: StepCategory::Diagnostic,
        }
    }

    /// Create a fix step.
    pub fn fix(description: &str, command: &str, needs_sudo: bool) -> Self {
        Self {
            description: description.to_string(),
            command: command.to_string(),
            needs_sudo,
            category: StepCategory::Fix,
        }
    }

    /// Create an info step (no command).
    pub fn info(description: &str) -> Self {
        Self {
            description: description.to_string(),
            command: String::new(),
            category: StepCategory::Info,
            needs_sudo: false,
        }
    }
}

/// Reference to evidence from a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// Name of the probe (e.g., "disk_usage", "systemd_boot_time").
    pub probe_name: String,
    /// Key into transcript evidence store.
    pub snippet_id: String,
    /// Optional excerpt from the evidence.
    pub excerpt: Option<String>,
}

impl EvidenceRef {
    /// Create a new evidence reference.
    pub fn new(probe_name: &str, snippet_id: &str) -> Self {
        Self {
            probe_name: probe_name.to_string(),
            snippet_id: snippet_id.to_string(),
            excerpt: None,
        }
    }

    /// Create with excerpt.
    pub fn with_excerpt(probe_name: &str, snippet_id: &str, excerpt: &str) -> Self {
        Self {
            probe_name: probe_name.to_string(),
            snippet_id: snippet_id.to_string(),
            excerpt: Some(excerpt.to_string()),
        }
    }
}

/// Metrics collected during ticket processing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketMetrics {
    /// Total LLM tokens used.
    pub llm_token_count: u64,
    /// LLM latency in milliseconds.
    pub llm_latency_ms: u64,
    /// Number of probes that ran.
    pub probes_ran: usize,
    /// Total probe latency in milliseconds.
    pub probes_latency_ms: u64,
    /// Number of knowledge lookups.
    pub knowledge_lookups: usize,
    /// Whether a retry was attempted.
    pub retry_attempted: bool,
}

impl TicketMetrics {
    /// Total processing time.
    pub fn total_ms(&self) -> u64 {
        self.llm_latency_ms + self.probes_latency_ms
    }
}

/// The strict result contract for all specialists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResult {
    /// The outcome of processing.
    pub outcome: TicketOutcome,
    /// Plain language summary (2-4 lines max).
    pub human_summary: String,
    /// More detailed explanation (for REPL display).
    pub diagnosis: Option<String>,
    /// Proposed steps/commands.
    pub steps: Vec<ProposedStep>,
    /// References to evidence used.
    pub evidence_refs: Vec<EvidenceRef>,
    /// Error or timeout details (for failures).
    pub error_info: Option<String>,
    /// Processing metrics.
    pub metrics: TicketMetrics,
    /// Which specialist handled this.
    pub handler: Option<String>,
    /// Which department.
    pub department: Option<String>,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f32,
}

impl SpecialistResult {
    /// Create a successful result.
    pub fn success(summary: &str) -> Self {
        Self {
            outcome: TicketOutcome::Success,
            human_summary: summary.to_string(),
            diagnosis: None,
            steps: Vec::new(),
            evidence_refs: Vec::new(),
            error_info: None,
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence: 1.0,
        }
    }

    /// Create a partial result.
    pub fn partial(summary: &str, missing: &str) -> Self {
        Self {
            outcome: TicketOutcome::Partial,
            human_summary: summary.to_string(),
            diagnosis: Some(format!("Missing: {}", missing)),
            steps: Vec::new(),
            evidence_refs: Vec::new(),
            error_info: None,
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence: 0.7,
        }
    }

    /// Create a timeout result.
    pub fn timeout(stage: &str) -> Self {
        Self {
            outcome: TicketOutcome::Timeout,
            human_summary: "Processing exceeded time budget.".to_string(),
            diagnosis: None,
            steps: Vec::new(),
            evidence_refs: Vec::new(),
            error_info: Some(format!("Timeout at stage: {}", stage)),
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence: 0.0,
        }
    }

    /// Create a parse error result.
    pub fn parse_error(details: &str) -> Self {
        Self {
            outcome: TicketOutcome::ParseError,
            human_summary: "Could not process specialist response.".to_string(),
            diagnosis: None,
            steps: Vec::new(),
            evidence_refs: Vec::new(),
            error_info: Some(details.to_string()),
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence: 0.0,
        }
    }

    /// Create an internal error result.
    pub fn internal_error(details: &str) -> Self {
        Self {
            outcome: TicketOutcome::InternalError,
            human_summary: "An internal error occurred.".to_string(),
            diagnosis: None,
            steps: Vec::new(),
            evidence_refs: Vec::new(),
            error_info: Some(details.to_string()),
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence: 0.0,
        }
    }

    /// Create an unsupported request result.
    pub fn unsupported(reason: &str) -> Self {
        Self {
            outcome: TicketOutcome::Unsupported,
            human_summary: reason.to_string(),
            diagnosis: None,
            steps: Vec::new(),
            evidence_refs: Vec::new(),
            error_info: None,
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence: 0.0,
        }
    }

    /// Create a clarification required result.
    pub fn needs_clarification(question: &str) -> Self {
        Self {
            outcome: TicketOutcome::ClarificationRequired,
            human_summary: question.to_string(),
            diagnosis: None,
            steps: Vec::new(),
            evidence_refs: Vec::new(),
            error_info: None,
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence: 0.5,
        }
    }

    /// Set handler info.
    pub fn with_handler(mut self, handler: &str, department: &str) -> Self {
        self.handler = Some(handler.to_string());
        self.department = Some(department.to_string());
        self
    }

    /// Add a step.
    pub fn with_step(mut self, step: ProposedStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add evidence reference.
    pub fn with_evidence(mut self, evidence: EvidenceRef) -> Self {
        self.evidence_refs.push(evidence);
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set metrics.
    pub fn with_metrics(mut self, metrics: TicketMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set diagnosis.
    pub fn with_diagnosis(mut self, diagnosis: &str) -> Self {
        self.diagnosis = Some(diagnosis.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_classification() {
        assert!(TicketOutcome::Success.is_success());
        assert!(TicketOutcome::Success.is_resolved());
        assert!(!TicketOutcome::Success.is_failure());

        assert!(!TicketOutcome::Partial.is_success());
        assert!(TicketOutcome::Partial.is_resolved());

        assert!(TicketOutcome::Timeout.is_failure());
        assert!(!TicketOutcome::Timeout.is_resolved());

        assert!(TicketOutcome::ClarificationRequired.needs_followup());
    }

    #[test]
    fn test_result_builders() {
        let success = SpecialistResult::success("All good");
        assert_eq!(success.outcome, TicketOutcome::Success);
        assert_eq!(success.confidence, 1.0);

        let timeout = SpecialistResult::timeout("senior_llm");
        assert_eq!(timeout.outcome, TicketOutcome::Timeout);
        assert!(timeout.error_info.is_some());
    }
}
