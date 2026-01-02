//! Main ticket verification loop runner (v0.0.401).
//!
//! Wraps the service desk answer with:
//! - Junior verification (bounded by junior_rounds_max)
//! - Senior escalation when junior exhausted
//! - v0.0.297: LLM-based self-healing for failed validations
//! - v0.0.376: Domain-specific validation thresholds
//! - v0.0.401: Specialist learning capture (learn from escalations)
//! - Revision application between rounds
//! - Full transcript visibility

use anna_shared::reliability::ReliabilityInput;
use anna_shared::rpc::{ProbeResult, TranslatorTicket};
use anna_shared::ticket::TicketStatus;
use anna_shared::transcript::Transcript;
use tracing::{info, warn};

use crate::ticket_service::{
    add_status_change_event, add_ticket_created_event, create_ticket_from_translator,
    TicketServiceConfig,
};

use super::evidence::evidence_kinds_from_route;
use super::junior::{run_junior_loop, JuniorLoopResult};
use super::senior::{attempt_llm_healing, run_senior_loop};
use super::types::TicketLoopResult;

/// Run the ticket verification loop on an answer (v0.0.297: async with LLM self-healing).
///
/// Flow:
/// 1. Create ticket from translator output
/// 2. Run junior verification
/// 3. If not verified, apply revision and retry (up to junior_rounds_max)
/// 4. If junior exhausted, escalate to senior with LLM self-healing
/// 5. Apply senior revision (up to senior_rounds_max)
/// 6. Return final result with ticket state
pub async fn run_ticket_loop(
    request_id: &str,
    user_request: &str,
    answer: &str,
    translator_ticket: &TranslatorTicket,
    route_class: &str,
    probe_results: &[ProbeResult],
    reliability_input: &ReliabilityInput,
    transcript: &mut Transcript,
    elapsed_ms: u64,
    config: Option<TicketServiceConfig>,
    model: &str,
    timeout_secs: u64,
) -> TicketLoopResult {
    let config = config.unwrap_or_default();

    // Derive evidence kinds from route class
    let evidence_kinds = evidence_kinds_from_route(route_class);

    // Step 1: Create ticket
    let mut ticket = create_ticket_from_translator(
        request_id,
        user_request,
        translator_ticket,
        route_class,
        evidence_kinds,
    );

    // Override ticket limits with config values
    ticket.junior_rounds_max = config.junior_rounds_max;
    ticket.senior_rounds_max = config.senior_rounds_max;

    add_ticket_created_event(transcript, elapsed_ms, &ticket);
    info!(
        "Ticket created: id={}, domain={}, intent={}",
        ticket.ticket_id, ticket.domain, ticket.intent
    );

    // Step 2: Junior verification loop
    let mut current_answer = answer.to_string();

    let junior_history = match run_junior_loop(
        &mut current_answer,
        &mut ticket,
        probe_results,
        user_request,
        reliability_input,
        transcript,
        elapsed_ms,
        &config,
    ) {
        JuniorLoopResult::Verified(result) => return result,
        JuniorLoopResult::Exhausted(history) => history,
    };

    // Step 3: Junior exhausted - escalate to senior
    warn!(
        "Junior verification exhausted after {} rounds",
        ticket.junior_rounds_max
    );
    ticket.status = TicketStatus::Escalated;
    add_status_change_event(
        transcript,
        elapsed_ms,
        &ticket,
        TicketStatus::AnswerDrafted,
        TicketStatus::Escalated,
    );

    // Step 4: Senior escalation with LLM self-healing (v0.0.297)
    if let Some(result) = attempt_llm_healing(
        &current_answer,
        user_request,
        probe_results,
        reliability_input,
        &mut ticket,
        transcript,
        elapsed_ms,
        model,
        timeout_secs,
    )
    .await
    {
        return result;
    }

    // Step 4b: Deterministic senior escalation fallback
    if let Some(result) = run_senior_loop(
        &mut current_answer,
        &mut ticket,
        &junior_history,
        probe_results,
        user_request,
        reliability_input,
        transcript,
        elapsed_ms,
        &config,
    ) {
        return result;
    }

    // Step 5: All rounds exhausted - mark as failed
    warn!(
        "Ticket verification failed after {} junior + {} senior rounds",
        ticket.junior_rounds_max, ticket.senior_rounds_max
    );
    ticket.status = TicketStatus::Failed;
    add_status_change_event(
        transcript,
        elapsed_ms,
        &ticket,
        TicketStatus::Escalated,
        TicketStatus::Failed,
    );

    // Return with score 0 for failed tickets
    TicketLoopResult {
        answer: current_answer,
        ticket,
        verified: false,
        score: 0,
    }
}
