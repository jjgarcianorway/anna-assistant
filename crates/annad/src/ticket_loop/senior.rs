//! Senior escalation loop logic with LLM self-healing.

use anna_shared::grounding::ParsedEvidence;
use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
use anna_shared::reliability::ReliabilityInput;
use anna_shared::rpc::ProbeResult;
use anna_shared::specialist_learning::SolutionType;
use anna_shared::ticket::{Ticket, TicketStatus};
use anna_shared::transcript::Transcript;
use tracing::{info, warn};

use crate::answer_validator;
use crate::learning_capture::{capture_lesson, parse_domain};
use crate::ticket_service::{
    add_revision_event, add_senior_escalation_event, add_status_change_event,
    TicketServiceConfig,
};

use super::types::TicketLoopResult;

/// Attempt LLM-based self-healing for senior escalation (v0.0.297).
///
/// Returns Some(TicketLoopResult) if healing succeeds, None if failed.
pub async fn attempt_llm_healing(
    current_answer: &str,
    user_request: &str,
    probe_results: &[ProbeResult],
    reliability_input: &ReliabilityInput,
    ticket: &mut Ticket,
    transcript: &mut Transcript,
    elapsed_ms: u64,
    model: &str,
    timeout_secs: u64,
) -> Option<TicketLoopResult> {
    // Build ParsedEvidence from probe results for LLM self-healing
    let parsed_probes: Vec<ParsedProbeData> =
        probe_results.iter().map(parse_probe_result).collect();
    let evidence = ParsedEvidence::from_probes(&parsed_probes);

    info!("Attempting LLM-based self-healing for senior escalation");

    let validation_result = answer_validator::validate_and_heal_with_domain(
        current_answer,
        user_request,
        &evidence,
        reliability_input,
        model,
        timeout_secs,
        Some(parse_domain(&ticket.domain)),
    )
    .await;

    // Log validation path for debugging
    for step in &validation_result.validation_path {
        info!("Validation: {}", step);
    }

    if !validation_result.passed {
        warn!(
            "LLM self-healing failed: score={}, issues={:?}",
            validation_result.score,
            validation_result
                .issues
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
        );
        return None;
    }

    info!(
        "LLM self-healing succeeded: score={}, attempts={}",
        validation_result.score, validation_result.heal_attempts
    );

    // Create a synthetic escalation event for transcript
    let escalation = anna_shared::revision::SeniorEscalation::success(
        anna_shared::revision::RevisionInstruction::default()
            .with_explanation("LLM self-healing applied"),
    );
    add_senior_escalation_event(transcript, elapsed_ms, &escalation);

    if validation_result.heal_attempts > 0 {
        add_revision_event(
            transcript,
            elapsed_ms,
            vec![format!(
                "LLM healed answer in {} attempts",
                validation_result.heal_attempts
            )],
        );
    }

    ticket.status = TicketStatus::Verified;
    add_status_change_event(
        transcript,
        elapsed_ms,
        ticket,
        TicketStatus::Escalated,
        TicketStatus::Verified,
    );

    // v0.0.401: Capture learning from LLM self-healing success
    if validation_result.heal_attempts > 0 {
        capture_lesson(
            user_request,
            &ticket.domain,
            &validation_result.answer,
            probe_results,
            SolutionType::LlmSelfHealing {
                correction_type: format!(
                    "Healed in {} attempts",
                    validation_result.heal_attempts
                ),
            },
            validation_result.score,
        );
    }

    Some(TicketLoopResult {
        answer: validation_result.answer,
        ticket: ticket.clone(),
        verified: true,
        score: validation_result.score,
    })
}

/// Run deterministic senior escalation loop.
///
/// Returns Some(TicketLoopResult) if verification succeeds, None if senior exhausted.
pub fn run_senior_loop(
    current_answer: &mut String,
    ticket: &mut Ticket,
    junior_history: &[anna_shared::revision::JuniorVerification],
    probe_results: &[ProbeResult],
    user_request: &str,
    reliability_input: &ReliabilityInput,
    transcript: &mut Transcript,
    elapsed_ms: u64,
    config: &TicketServiceConfig,
) -> Option<TicketLoopResult> {
    // v0.0.305: Track if we've already added escalation event to avoid duplicates
    let mut escalation_event_added = false;

    while ticket.senior_attempt < ticket.senior_rounds_max {
        ticket.senior_attempt += 1;

        let escalation = crate::ticket_service::senior_escalate(
            current_answer,
            ticket,
            junior_history,
            probe_results,
        );

        // v0.0.305: Only add first escalation event to transcript
        if !escalation_event_added {
            add_senior_escalation_event(transcript, elapsed_ms, &escalation);
            escalation_event_added = true;
        }

        if escalation.successful && escalation.instruction.has_changes() {
            // Apply senior revision
            let (revised, changes) = crate::ticket_service::apply_revision(
                current_answer,
                &escalation.instruction,
                probe_results,
            );

            if !changes.is_empty() {
                add_revision_event(transcript, elapsed_ms, changes);
                *current_answer = revised;
            }

            // Re-verify with junior after senior guidance
            let final_verification = crate::ticket_service::junior_verify(
                current_answer,
                ticket,
                probe_results,
                user_request,
                reliability_input,
                config,
            );

            if final_verification.verified {
                info!(
                    "Verified after senior escalation: score={}",
                    final_verification.score
                );
                ticket.status = TicketStatus::Verified;
                add_status_change_event(
                    transcript,
                    elapsed_ms,
                    ticket,
                    TicketStatus::Escalated,
                    TicketStatus::Verified,
                );

                // v0.0.401: Capture learning from senior escalation success
                capture_lesson(
                    user_request,
                    &ticket.domain,
                    current_answer,
                    probe_results,
                    SolutionType::SeniorGuidance {
                        instruction_summary: escalation.instruction.summary(),
                    },
                    final_verification.score,
                );

                return Some(TicketLoopResult {
                    answer: current_answer.clone(),
                    ticket: ticket.clone(),
                    verified: true,
                    score: final_verification.score,
                });
            }
        } else {
            warn!("Senior escalation did not provide useful guidance");
        }
    }

    None
}
