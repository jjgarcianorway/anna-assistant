//! Junior verification loop logic.

use anna_shared::reliability::ReliabilityInput;
use anna_shared::rpc::ProbeResult;
use anna_shared::ticket::{Ticket, TicketStatus};
use anna_shared::transcript::Transcript;
use tracing::info;

use crate::ticket_service::{
    add_junior_review_event, add_revision_event, add_status_change_event, TicketServiceConfig,
};

use super::types::TicketLoopResult;

/// Result of junior verification loop
pub enum JuniorLoopResult {
    /// Verification succeeded
    Verified(TicketLoopResult),
    /// Junior exhausted, contains history for senior escalation
    Exhausted(Vec<anna_shared::revision::JuniorVerification>),
}

/// Run junior verification loop on an answer.
///
/// Returns JuniorLoopResult indicating success or exhaustion with history.
pub fn run_junior_loop(
    current_answer: &mut String,
    ticket: &mut Ticket,
    probe_results: &[ProbeResult],
    user_request: &str,
    reliability_input: &ReliabilityInput,
    transcript: &mut Transcript,
    elapsed_ms: u64,
    config: &TicketServiceConfig,
) -> JuniorLoopResult {
    let mut junior_history = Vec::new();

    while ticket.junior_attempt < ticket.junior_rounds_max {
        ticket.junior_attempt += 1;
        let old_status = ticket.status;
        ticket.status = TicketStatus::AnswerDrafted;

        if old_status != ticket.status {
            add_status_change_event(transcript, elapsed_ms, ticket, old_status, ticket.status);
        }

        // Run junior verification
        let verification = crate::ticket_service::junior_verify(
            current_answer,
            ticket,
            probe_results,
            user_request,
            reliability_input,
            config,
        );

        add_junior_review_event(transcript, elapsed_ms, ticket.junior_attempt, &verification);
        junior_history.push(verification.clone());

        if verification.verified {
            info!(
                "Junior verified on attempt {}: score={}",
                ticket.junior_attempt, verification.score
            );
            ticket.status = TicketStatus::Verified;
            add_status_change_event(
                transcript,
                elapsed_ms,
                ticket,
                TicketStatus::AnswerDrafted,
                TicketStatus::Verified,
            );

            return JuniorLoopResult::Verified(TicketLoopResult {
                answer: current_answer.clone(),
                ticket: ticket.clone(),
                verified: true,
                score: verification.score,
            });
        }

        // Apply revision if instruction has changes
        if verification.instruction.has_changes() {
            let (revised, changes) = crate::ticket_service::apply_revision(
                current_answer,
                &verification.instruction,
                probe_results,
            );

            if !changes.is_empty() {
                add_revision_event(transcript, elapsed_ms, changes);
                *current_answer = revised;
            }
        }

        info!(
            "Junior attempt {} failed: score={}, retrying...",
            ticket.junior_attempt, verification.score
        );
    }

    JuniorLoopResult::Exhausted(junior_history)
}
