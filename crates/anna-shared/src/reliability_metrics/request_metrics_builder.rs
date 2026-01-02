//! Request metrics builder pattern (v0.0.444).
//!
//! Provides a fluent builder API for constructing RequestMetrics.

use super::canonical_outcome::CanonicalOutcome;
use super::request_metrics_types::RequestMetrics;

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
        self.metrics
            .record_claims(total, with_evidence, evidence_ids);
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
