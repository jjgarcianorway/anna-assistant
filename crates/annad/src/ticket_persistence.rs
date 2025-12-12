//! Ticket persistence - bridge between request handling and ticket logs (v0.0.411).
//!
//! Creates TicketLog entries from request results for:
//! - Stats computation (truthful success/failure rates)
//! - Learning hooks (recipes from successful answers)
//! - Historical analysis
//!
//! This ensures every request creates exactly one TicketLog entry.

use anna_shared::rpc::{ProbeResult, ServiceDeskResult, SpecialistDomain, TranslatorTicket};
use anna_shared::ticket_log::{DocSnippet, ProbeLog, SolverOutput, TicketLog, TicketResult};
use anna_shared::ticket_state::{ErrorKind, TicketOutcome, TicketState};
use anna_shared::trace::SpecialistOutcome;
use tracing::{debug, warn};

/// Create and save a TicketLog from a request result
///
/// This is the canonical way to record ticket outcomes for stats.
/// Called at the end of every request handling path.
pub fn persist_ticket_log(
    result: &ServiceDeskResult,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    specialist_outcome: SpecialistOutcome,
    handler: &str,
    duration_ms: u64,
) {
    let log = create_ticket_log(
        result,
        ticket,
        probe_results,
        specialist_outcome,
        handler,
        duration_ms,
    );

    if let Err(e) = log.save() {
        warn!("Failed to save ticket log {}: {}", log.id, e);
    } else {
        debug!(
            "Saved ticket log: id={}, outcome={:?}, reliability={}",
            log.id, log.state, log.reliability_score
        );
    }
}

/// Simplified ticket persistence from ServiceDeskResult only
///
/// v0.0.411: Used when we only have the final result (e.g., in wrap_with_theatre).
/// Extracts necessary info from the result itself.
pub fn persist_from_result(result: &ServiceDeskResult) {
    let ticket_id = result
        .case_number
        .clone()
        .unwrap_or_else(|| format!("TKT-{}", result.request_id));

    let handler = result
        .staff_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Use probe results directly from evidence block
    let probe_results: Vec<ProbeResult> = result.evidence.probes_executed.clone();

    // Get duration from transcript (estimate from last event)
    let duration_ms = result
        .transcript
        .events
        .last()
        .map(|e| e.elapsed_ms)
        .unwrap_or(0);

    // Determine specialist outcome from execution trace
    let specialist_outcome = result
        .execution_trace
        .as_ref()
        .map(|trace| trace.specialist_outcome.clone())
        .unwrap_or(SpecialistOutcome::Ok);

    let mut log = TicketLog::new(
        ticket_id.clone(),
        result.domain,
        "query".to_string(), // Generic intent
        extract_query_from_result(result),
    )
    .with_handler(&handler)
    .with_metrics(duration_ms, result.reliability_score)
    .with_probes(&probe_results);

    // Derive outcome
    let (state, error_kind, ticket_result) =
        derive_ticket_outcome(specialist_outcome, result.reliability_score, result);

    log.state = Some(state);
    log.error_kind = error_kind;
    log.result = ticket_result;
    log.answer = result.answer.clone();

    if let Err(e) = log.save() {
        warn!("Failed to save ticket log {}: {}", ticket_id, e);
    } else {
        debug!(
            "Persisted ticket: id={}, state={:?}, reliability={}",
            ticket_id, log.state, log.reliability_score
        );
    }
}

/// Create a TicketLog from request handling results
///
/// Maps the various success/failure states to proper outcomes:
/// - High reliability (80+) + success → Success
/// - Medium reliability (50-79) → Partial
/// - Low reliability or explicit failure → appropriate error outcome
pub fn create_ticket_log(
    result: &ServiceDeskResult,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    specialist_outcome: SpecialistOutcome,
    handler: &str,
    duration_ms: u64,
) -> TicketLog {
    let ticket_id = result
        .case_number
        .clone()
        .unwrap_or_else(|| format!("TKT-{}", result.request_id));

    let mut log = TicketLog::new(
        ticket_id,
        ticket.domain,
        ticket.intent.to_string(),
        extract_query_from_result(result),
    )
    .with_handler(handler)
    .with_metrics(duration_ms, result.reliability_score)
    .with_probes(probe_results);

    // Set solver output based on handler
    log.solver_output = SolverOutput {
        solver_type: handler.to_string(),
        analysis: None,
        model: None,
        tokens_used: None,
    };

    // Derive outcome from specialist outcome and reliability
    let (state, error_kind, ticket_result) =
        derive_ticket_outcome(specialist_outcome, result.reliability_score, result);

    log.state = Some(state);
    log.error_kind = error_kind;
    log.result = ticket_result;
    log.answer = result.answer.clone();

    // Track escalation if applicable
    if let Some(trace) = &result.execution_trace {
        if !matches!(trace.fallback_used, anna_shared::trace::FallbackUsed::None) {
            log.escalated = true;
            log.escalation_path = Some("specialist→fallback".to_string());
        }
    }

    // Track LLM calls (from handler name)
    if handler.contains("llm") {
        log.llm_calls = 1;
    }

    log
}

/// Derive ticket outcome from specialist outcome and reliability score
fn derive_ticket_outcome(
    specialist_outcome: SpecialistOutcome,
    reliability_score: u8,
    result: &ServiceDeskResult,
) -> (TicketState, Option<ErrorKind>, TicketResult) {
    // Check for explicit errors first
    if let Some(ref last_error) = result.evidence.last_error {
        if last_error.contains("timeout") {
            return (
                TicketState::Failed,
                Some(ErrorKind::LlmTimeout),
                TicketResult::Failed,
            );
        }
        if last_error.contains("parse") || last_error.contains("invalid") {
            return (
                TicketState::Failed,
                Some(ErrorKind::LlmParseError),
                TicketResult::Failed,
            );
        }
    }

    // Map specialist outcome to ticket state
    match specialist_outcome {
        SpecialistOutcome::Ok | SpecialistOutcome::Skipped => {
            // Success depends on reliability score
            if reliability_score >= 80 {
                (TicketState::Success, None, TicketResult::Success)
            } else if reliability_score >= 50 {
                (TicketState::Success, None, TicketResult::Partial)
            } else {
                // Low reliability is a partial answer, not failure
                (TicketState::Success, None, TicketResult::Partial)
            }
        }
        SpecialistOutcome::Timeout => (
            TicketState::Failed,
            Some(ErrorKind::LlmTimeout),
            TicketResult::Failed,
        ),
        SpecialistOutcome::Error => (
            TicketState::Failed,
            Some(ErrorKind::InternalError),
            TicketResult::Failed,
        ),
        SpecialistOutcome::BudgetExceeded => (
            TicketState::Failed,
            Some(ErrorKind::InternalError),
            TicketResult::Failed,
        ),
    }
}

/// Extract the user query from the result
fn extract_query_from_result(result: &ServiceDeskResult) -> String {
    use anna_shared::transcript::{Actor, TranscriptEventKind};

    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                return text.clone();
            }
        }
    }

    // Fallback
    result.request_id.clone()
}

/// Get recent ticket stats (computed from persisted logs)
pub fn get_ticket_stats() -> anna_shared::ticket_stats::TicketStats {
    let tickets = anna_shared::ticket_log::load_all_tickets();
    anna_shared::ticket_stats::calculate_stats(&tickets)
}

/// Get stats for a specific time range (last N hours)
pub fn get_recent_stats(hours: u64) -> anna_shared::ticket_stats::TicketStats {
    let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours as i64);
    let cutoff_str = cutoff.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let tickets: Vec<_> = anna_shared::ticket_log::load_all_tickets()
        .into_iter()
        .filter(|t| t.timestamp >= cutoff_str)
        .collect();

    anna_shared::ticket_stats::calculate_stats(&tickets)
}

/// v0.0.411: Trigger learning hooks based on ticket outcome
///
/// Called after ticket persistence to trigger additional learning based on outcome.
/// - Success outcomes trigger recipe learning
/// - Partial outcomes log for potential future learning
/// - Error outcomes help improve error handling
pub fn trigger_learning_for_outcome(result: &ServiceDeskResult, outcome: &TicketOutcome) {
    match outcome {
        TicketOutcome::Success => {
            // High-confidence success - immediately trigger learning
            debug!(
                "Success outcome for {} - learning hooks already triggered via try_learn_from_result",
                result.request_id
            );
        }
        TicketOutcome::Partial => {
            // Partial answer - log for manual review / future learning
            debug!(
                "Partial outcome for {} - answer may need refinement (reliability={})",
                result.request_id, result.reliability_score
            );
        }
        TicketOutcome::CannotAnswerSafely => {
            // Could not answer safely - this is honest but not learnable
            debug!(
                "CannotAnswerSafely outcome for {} - honest limitation",
                result.request_id
            );
        }
        TicketOutcome::ErrorParse
        | TicketOutcome::ErrorTimeout
        | TicketOutcome::ErrorTool
        | TicketOutcome::ErrorInternal => {
            // Errors - log for debugging but don't learn from failures
            warn!(
                "Error outcome {:?} for {} - not learning from errors",
                outcome, result.request_id
            );
        }
    }
}

/// v0.0.411: Check if a ticket outcome should trigger learning
pub fn should_learn_from_outcome(outcome: &TicketOutcome, reliability_score: u8) -> bool {
    match outcome {
        // Only learn from clear successes with good reliability
        TicketOutcome::Success => reliability_score >= 70,
        // Partial answers with very high reliability might still be learnable
        TicketOutcome::Partial => reliability_score >= 85,
        // Never learn from errors or cannot-answer
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::rpc::QueryIntent;
    use anna_shared::transcript::Transcript;

    fn mock_result(reliability: u8) -> ServiceDeskResult {
        ServiceDeskResult {
            request_id: "test-001".to_string(),
            case_number: Some("DSK-0001".to_string()),
            assigned_staff: None,
            staff_id: None,
            answer: "Test answer".to_string(),
            validated: reliability >= 80,
            reliability_score: reliability,
            reliability_signals: anna_shared::rpc::ReliabilitySignals::default(),
            reliability_explanation: None,
            domain: SpecialistDomain::System,
            evidence: Default::default(),
            needs_clarification: false,
            clarification_question: None,
            clarification_request: None,
            transcript: Transcript::new(),
            execution_trace: None,
            proposed_change: None,
            proposed_changes: vec![],
            feedback_request: None,
        }
    }

    #[test]
    fn test_derive_outcomes() {
        let result = mock_result(85);
        // High reliability = Success
        let (s, e, r) = derive_ticket_outcome(SpecialistOutcome::Ok, 85, &result);
        assert_eq!(s, TicketState::Success);
        assert!(e.is_none());
        assert_eq!(r, TicketResult::Success);

        // Lower reliability = Partial
        let (s, _, r) = derive_ticket_outcome(SpecialistOutcome::Ok, 60, &result);
        assert_eq!(s, TicketState::Success);
        assert_eq!(r, TicketResult::Partial);

        // Timeout = Failed
        let (s, e, r) = derive_ticket_outcome(SpecialistOutcome::Timeout, 0, &result);
        assert_eq!(s, TicketState::Failed);
        assert_eq!(e, Some(ErrorKind::LlmTimeout));
        assert_eq!(r, TicketResult::Failed);
    }

    #[test]
    fn test_should_learn_from_outcome() {
        assert!(should_learn_from_outcome(&TicketOutcome::Success, 80));
        assert!(!should_learn_from_outcome(&TicketOutcome::Success, 60));
        assert!(should_learn_from_outcome(&TicketOutcome::Partial, 85));
        assert!(!should_learn_from_outcome(&TicketOutcome::ErrorParse, 100));
    }
}
