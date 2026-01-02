//! Request metrics type definitions (v0.0.444).
//!
//! Core data structures for tracking per-request metrics.

use super::canonical_outcome::CanonicalOutcome;
use serde::{Deserialize, Serialize};

/// Metrics for a single request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Unique request ID.
    pub request_id: String,

    /// Original user query.
    pub query: String,

    /// Routed topic/domain.
    pub routed_topic: String,

    /// Intent classification.
    pub intent: Option<String>,

    // === Probe Metrics ===
    /// Probe IDs that were required for this query.
    pub probes_required: Vec<String>,

    /// Probe IDs that were actually run.
    pub probes_run: Vec<String>,

    /// Count of probes that returned successfully.
    pub probes_ok_count: u32,

    /// Total probes that were run.
    pub probes_run_count: u32,

    // === Evidence Metrics ===
    /// Total claims in the answer.
    pub claim_count: u32,

    /// Claims that have evidence backing.
    pub claims_with_evidence_count: u32,

    /// Evidence IDs used.
    pub evidence_ids: Vec<String>,

    // === Validation ===
    /// Did the validator pass?
    pub validator_pass: bool,

    /// Validation error message if failed.
    pub validation_error: Option<String>,

    // === Outcome ===
    /// Final canonical outcome.
    pub outcome: CanonicalOutcome,

    /// Error message if outcome is a failure.
    pub error_message: Option<String>,

    // === Timing (milliseconds) ===
    /// Total request duration.
    pub total_ms: u64,

    /// Time spent on probes.
    pub probe_ms: u64,

    /// Time spent on LLM calls.
    pub llm_ms: u64,

    /// Time spent on knowledge lookup.
    pub knowledge_ms: u64,

    // === Models Used ===
    /// Models used in this request.
    pub models_used: ModelsUsed,

    /// Timestamp when request started (Unix ms).
    pub started_at: u64,

    /// Timestamp when request completed (Unix ms).
    pub completed_at: u64,
}

/// Models used during request processing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelsUsed {
    /// Translator model (if any).
    pub translator: Option<String>,
    /// Specialist model (if any).
    pub specialist: Option<String>,
    /// Verifier model (if any).
    pub verifier: Option<String>,
}

impl RequestMetrics {
    /// Create a new request metrics instance.
    pub fn new(request_id: impl Into<String>, query: impl Into<String>) -> Self {
        let now = current_millis();
        Self {
            request_id: request_id.into(),
            query: query.into(),
            routed_topic: String::new(),
            intent: None,
            probes_required: Vec::new(),
            probes_run: Vec::new(),
            probes_ok_count: 0,
            probes_run_count: 0,
            claim_count: 0,
            claims_with_evidence_count: 0,
            evidence_ids: Vec::new(),
            validator_pass: false,
            validation_error: None,
            outcome: CanonicalOutcome::ErrorInternal,
            error_message: None,
            total_ms: 0,
            probe_ms: 0,
            llm_ms: 0,
            knowledge_ms: 0,
            models_used: ModelsUsed::default(),
            started_at: now,
            completed_at: 0,
        }
    }

    /// Set routed topic.
    pub fn set_topic(&mut self, topic: impl Into<String>) {
        self.routed_topic = topic.into();
    }

    /// Set intent.
    pub fn set_intent(&mut self, intent: impl Into<String>) {
        self.intent = Some(intent.into());
    }

    /// Record probe execution.
    pub fn record_probes(&mut self, required: Vec<String>, run: Vec<String>, ok_count: u32) {
        self.probes_required = required;
        self.probes_run_count = run.len() as u32;
        self.probes_run = run;
        self.probes_ok_count = ok_count;
    }

    /// Record claim/evidence counts.
    pub fn record_claims(&mut self, total: u32, with_evidence: u32, evidence_ids: Vec<String>) {
        self.claim_count = total;
        self.claims_with_evidence_count = with_evidence;
        self.evidence_ids = evidence_ids;
    }

    /// Record validation result.
    pub fn record_validation(&mut self, passed: bool, error: Option<String>) {
        self.validator_pass = passed;
        self.validation_error = error;
    }

    /// Record timing.
    pub fn record_timing(&mut self, probe_ms: u64, llm_ms: u64, knowledge_ms: u64) {
        self.probe_ms = probe_ms;
        self.llm_ms = llm_ms;
        self.knowledge_ms = knowledge_ms;
    }

    /// Record models used.
    pub fn record_models(
        &mut self,
        translator: Option<String>,
        specialist: Option<String>,
        verifier: Option<String>,
    ) {
        self.models_used = ModelsUsed {
            translator,
            specialist,
            verifier,
        };
    }

    /// Complete the request with an outcome.
    pub fn complete(&mut self, outcome: CanonicalOutcome, error: Option<String>) {
        self.completed_at = current_millis();
        self.total_ms = self.completed_at.saturating_sub(self.started_at);
        self.outcome = outcome;
        self.error_message = error;
    }

    /// Get evidence coverage (0.0-1.0).
    pub fn evidence_coverage(&self) -> f32 {
        if self.claim_count == 0 {
            1.0 // No claims = 100% coverage (vacuously true)
        } else {
            self.claims_with_evidence_count as f32 / self.claim_count as f32
        }
    }

    /// Get probe coverage (0.0-1.0).
    pub fn probe_coverage(&self) -> f32 {
        if self.probes_required.is_empty() {
            1.0 // No required probes = 100% coverage
        } else {
            self.probes_ok_count as f32 / self.probes_required.len() as f32
        }
    }

    /// Get summary string for debug output.
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} → {} | evidence: {:.0}% | probes: {}/{} | {}ms",
            self.request_id,
            truncate(&self.query, 30),
            self.outcome.label(),
            self.evidence_coverage() * 100.0,
            self.probes_ok_count,
            self.probes_run_count,
            self.total_ms,
        )
    }
}

/// Helper to get current time in milliseconds.
pub(super) fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Truncate string with ellipsis.
pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
