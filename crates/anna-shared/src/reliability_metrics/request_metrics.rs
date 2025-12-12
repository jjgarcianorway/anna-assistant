//! Per-request metrics tracking (v0.0.444).
//!
//! Tracks detailed metrics for each request:
//! - Query details and routing
//! - Probe execution and coverage
//! - Claim/evidence tracking
//! - Timing breakdown
//! - Model usage

use super::canonical_outcome::CanonicalOutcome;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Builder for constructing RequestMetrics.
pub struct RequestMetricsBuilder {
    metrics: RequestMetrics,
}

impl RequestMetricsBuilder {
    /// Start building metrics for a request.
    pub fn new(request_id: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            metrics: RequestMetrics::new(request_id, query),
        }
    }

    /// Set topic.
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.metrics.set_topic(topic);
        self
    }

    /// Set intent.
    pub fn intent(mut self, intent: impl Into<String>) -> Self {
        self.metrics.set_intent(intent);
        self
    }

    /// Record probes.
    pub fn probes(mut self, required: Vec<String>, run: Vec<String>, ok: u32) -> Self {
        self.metrics.record_probes(required, run, ok);
        self
    }

    /// Record claims.
    pub fn claims(mut self, total: u32, with_evidence: u32, evidence_ids: Vec<String>) -> Self {
        self.metrics.record_claims(total, with_evidence, evidence_ids);
        self
    }

    /// Record timing.
    pub fn timing(mut self, probe_ms: u64, llm_ms: u64, knowledge_ms: u64) -> Self {
        self.metrics.record_timing(probe_ms, llm_ms, knowledge_ms);
        self
    }

    /// Set models.
    pub fn models(
        mut self,
        translator: Option<String>,
        specialist: Option<String>,
        verifier: Option<String>,
    ) -> Self {
        self.metrics.record_models(translator, specialist, verifier);
        self
    }

    /// Set validation.
    pub fn validation(mut self, passed: bool, error: Option<String>) -> Self {
        self.metrics.record_validation(passed, error);
        self
    }

    /// Complete with outcome.
    pub fn finish(mut self, outcome: CanonicalOutcome, error: Option<String>) -> RequestMetrics {
        self.metrics.complete(outcome, error);
        self.metrics
    }
}

/// Storage for request metrics history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestMetricsStore {
    /// Recent metrics (rolling window).
    pub recent: Vec<RequestMetrics>,

    /// Maximum entries to keep.
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,

    /// Metrics indexed by request_id for quick lookup.
    #[serde(skip)]
    pub by_id: HashMap<String, usize>,
}

fn default_max_entries() -> usize {
    1000
}

impl RequestMetricsStore {
    /// Create a new store.
    pub fn new() -> Self {
        Self {
            recent: Vec::new(),
            max_entries: 1000,
            by_id: HashMap::new(),
        }
    }

    /// Add a request to the store.
    pub fn add(&mut self, metrics: RequestMetrics) {
        let id = metrics.request_id.clone();
        self.recent.push(metrics);
        let idx = self.recent.len() - 1;
        self.by_id.insert(id, idx);

        // Trim if needed
        if self.recent.len() > self.max_entries {
            let removed = self.recent.remove(0);
            self.by_id.remove(&removed.request_id);
            // Rebuild index (indices shifted)
            self.rebuild_index();
        }
    }

    /// Get metrics by request ID.
    pub fn get(&self, request_id: &str) -> Option<&RequestMetrics> {
        self.by_id.get(request_id).and_then(|&i| self.recent.get(i))
    }

    /// Rebuild the by_id index.
    fn rebuild_index(&mut self) {
        self.by_id.clear();
        for (i, m) in self.recent.iter().enumerate() {
            self.by_id.insert(m.request_id.clone(), i);
        }
    }

    /// Get recent N requests.
    pub fn recent(&self, n: usize) -> &[RequestMetrics] {
        let start = self.recent.len().saturating_sub(n);
        &self.recent[start..]
    }

    /// Get requests with a specific outcome.
    pub fn with_outcome(&self, outcome: CanonicalOutcome) -> Vec<&RequestMetrics> {
        self.recent.iter().filter(|m| m.outcome == outcome).collect()
    }

    /// Get requests for a topic.
    pub fn for_topic(&self, topic: &str) -> Vec<&RequestMetrics> {
        self.recent
            .iter()
            .filter(|m| m.routed_topic == topic)
            .collect()
    }
}

/// Helper to get current time in milliseconds.
fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Truncate string with ellipsis.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metrics() {
        let mut m = RequestMetrics::new("REQ-001", "What is my disk usage?");
        m.set_topic("storage");
        m.record_probes(
            vec!["df".into(), "du".into()],
            vec!["df".into(), "du".into()],
            2,
        );
        m.record_claims(3, 3, vec!["ev1".into(), "ev2".into()]);
        m.complete(CanonicalOutcome::AnsweredVerified, None);

        assert_eq!(m.evidence_coverage(), 1.0);
        assert_eq!(m.probe_coverage(), 1.0);
        assert!(m.outcome.is_resolved());
    }

    #[test]
    fn test_metrics_builder() {
        let m = RequestMetricsBuilder::new("REQ-002", "Test query")
            .topic("network")
            .intent("diagnose")
            .probes(vec!["ping".into()], vec!["ping".into()], 1)
            .claims(2, 1, vec!["ev1".into()])
            .timing(50, 500, 100)
            .finish(CanonicalOutcome::AnsweredPartial, None);

        assert_eq!(m.routed_topic, "network");
        assert_eq!(m.evidence_coverage(), 0.5);
        assert!(m.outcome.is_partial());
    }

    #[test]
    fn test_metrics_store() {
        let mut store = RequestMetricsStore::new();
        store.max_entries = 3;

        for i in 1..=5 {
            let m = RequestMetrics::new(format!("REQ-{:03}", i), format!("Query {}", i));
            store.add(m);
        }

        // Should have trimmed to 3 entries
        assert_eq!(store.recent.len(), 3);
        assert!(store.get("REQ-001").is_none());
        assert!(store.get("REQ-005").is_some());
    }
}
