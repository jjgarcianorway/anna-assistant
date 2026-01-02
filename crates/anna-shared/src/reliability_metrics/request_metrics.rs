//! Per-request metrics tracking (v0.0.444).
//!
//! Tracks detailed metrics for each request:
//! - Query details and routing
//! - Probe execution and coverage
//! - Claim/evidence tracking
//! - Timing breakdown
//! - Model usage
//!
//! This module provides re-exports of the main types and functionality.

// Re-export everything for convenience from sibling modules
pub use super::request_metrics_types::{ModelsUsed, RequestMetrics};
pub use super::request_metrics_builder::RequestMetricsBuilder;
pub use super::request_metrics_store::RequestMetricsStore;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reliability_metrics::canonical_outcome::CanonicalOutcome;

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
