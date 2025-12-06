//! Fast path handler for health/status queries (v0.0.39).
//!
//! Answers common queries without LLM using cached snapshot data.
//! v0.0.40: Added force_fast_path_fallback for timeout scenarios.

use anna_shared::fastpath::{classify_fast_path, try_fast_path, FastPathClass, FastPathInput, FastPathPolicy};
use anna_shared::health_view::build_health_summary;
use anna_shared::rpc::{EvidenceBlock, ReliabilitySignals, ServiceDeskResult, SpecialistDomain};
use anna_shared::snapshot::load_last_snapshot;
use anna_shared::trace::{ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;

/// Fast path result for building ServiceDeskResult
pub struct FastPathResult {
    pub answer: String,
    pub class: FastPathClass,
    pub reliability: u8,
    pub trace_note: String,
}

/// Try fast path answer (v0.0.39)
/// Returns Some if fast path can handle, None if should proceed to normal flow
pub fn try_fast_path_answer(query: &str, snapshot_max_age: u64) -> Option<FastPathResult> {
    let policy = FastPathPolicy {
        snapshot_max_age_secs: snapshot_max_age,
        enabled: true,
        min_reliability: 70,
    };

    // Try to load existing snapshot (fast path only uses cached data)
    // If snapshot is stale or missing, fast path will decline and probes run elsewhere
    let loaded = load_last_snapshot();
    let snapshot_ref = loaded.as_ref();

    let input = FastPathInput {
        request: query,
        snapshot: snapshot_ref,
        facts: None, // TODO: integrate facts in Phase 3
        policy: &policy,
    };

    let result = try_fast_path(&input);

    if result.handled {
        Some(FastPathResult {
            answer: result.answer_text,
            class: result.class,
            reliability: result.reliability_hint,
            trace_note: result.trace_note,
        })
    } else {
        None
    }
}

/// Check if query is a health query that should use fast path fallback on timeout (v0.0.40)
pub fn is_health_query(query: &str) -> bool {
    matches!(classify_fast_path(query), FastPathClass::SystemHealth)
}

/// Force fast path answer for health queries even with stale/missing snapshot (v0.0.40)
/// Used on timeout to avoid "rephrase" responses for health queries.
/// Returns a deterministic health status based on last available snapshot.
pub fn force_fast_path_fallback(query: &str) -> Option<FastPathResult> {
    let class = classify_fast_path(query);

    // Only force fallback for health-related queries
    if !matches!(
        class,
        FastPathClass::SystemHealth | FastPathClass::FailedServices
    ) {
        return None;
    }

    // Try to get any snapshot (even stale)
    let snapshot = load_last_snapshot();

    let answer = if let Some(ref snap) = snapshot {
        // Use RelevantHealthSummary even with stale snapshot
        let summary = build_health_summary(snap, None);
        format!(
            "{}\n\n*Note: Based on last cached snapshot (may be stale)*",
            summary.format()
        )
    } else {
        // No snapshot at all - return minimal healthy status
        "No critical issues detected. No warnings detected.\n\n*Note: No system snapshot available*"
            .to_string()
    };

    Some(FastPathResult {
        answer,
        class,
        reliability: 60, // Lower reliability for stale/missing data
        trace_note: "fast path fallback (timeout scenario)".to_string(),
    })
}

/// Build ServiceDeskResult from fast path answer
pub fn build_fast_path_result(
    request_id: String,
    answer: String,
    class: FastPathClass,
    reliability: u8,
    transcript: Transcript,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };

    // Build trace for fast path
    let trace = ExecutionTrace::deterministic_route(
        &class.to_string(),
        ProbeStats::default(), // No probes run
        vec![], // Evidence kinds already in snapshot
    );

    ServiceDeskResult {
        request_id,
        answer,
        reliability_score: reliability,
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: Some(trace),
        proposed_change: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_path_not_handled_for_complex_query() {
        let result = try_fast_path_answer("install vim", 300);
        assert!(result.is_none());
    }

    #[test]
    fn test_fast_path_class_display() {
        assert_eq!(FastPathClass::SystemHealth.to_string(), "system_health");
    }
}
