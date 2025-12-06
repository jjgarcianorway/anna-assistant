//! Ticket verification loop with bounded retries and escalation.
//!
//! Wraps the service desk answer with:
//! - Junior verification (bounded by junior_rounds_max)
//! - Senior escalation when junior exhausted
//! - Revision application between rounds
//! - Full transcript visibility

use anna_shared::reliability::ReliabilityInput;
use anna_shared::rpc::{ProbeResult, TranslatorTicket};
use anna_shared::ticket::{Ticket, TicketStatus};
use anna_shared::trace::EvidenceKind;
use anna_shared::transcript::Transcript;
use tracing::{info, warn};

use crate::ticket_service::{
    self, add_junior_review_event, add_revision_event, add_senior_escalation_event,
    add_status_change_event, add_ticket_created_event, create_ticket_from_translator,
    TicketServiceConfig,
};

/// Result of ticket verification loop
pub struct TicketLoopResult {
    /// Final answer after revisions
    pub answer: String,
    /// Final ticket state
    pub ticket: Ticket,
    /// Whether verification passed
    pub verified: bool,
    /// Final reliability score
    pub score: u8,
}

/// Run the ticket verification loop on an answer.
///
/// Flow:
/// 1. Create ticket from translator output
/// 2. Run junior verification
/// 3. If not verified, apply revision and retry (up to junior_rounds_max)
/// 4. If junior exhausted, escalate to senior
/// 5. Apply senior revision (up to senior_rounds_max)
/// 6. Return final result with ticket state
pub fn run_ticket_loop(
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
    let mut junior_history = Vec::new();

    while ticket.junior_attempt < ticket.junior_rounds_max {
        ticket.junior_attempt += 1;
        let old_status = ticket.status;
        ticket.status = TicketStatus::AnswerDrafted;

        if old_status != ticket.status {
            add_status_change_event(transcript, elapsed_ms, &ticket, old_status, ticket.status);
        }

        // Run junior verification
        let verification = ticket_service::junior_verify(
            &current_answer,
            &ticket,
            probe_results,
            user_request,
            reliability_input,
            &config,
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
                &ticket,
                TicketStatus::AnswerDrafted,
                TicketStatus::Verified,
            );

            return TicketLoopResult {
                answer: current_answer,
                ticket,
                verified: true,
                score: verification.score,
            };
        }

        // Apply revision if instruction has changes
        if verification.instruction.has_changes() {
            let (revised, changes) =
                ticket_service::apply_revision(&current_answer, &verification.instruction, probe_results);

            if !changes.is_empty() {
                add_revision_event(transcript, elapsed_ms, changes);
                current_answer = revised;
            }
        }

        info!(
            "Junior attempt {} failed: score={}, retrying...",
            ticket.junior_attempt, verification.score
        );
    }

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

    // Step 4: Senior escalation loop
    while ticket.senior_attempt < ticket.senior_rounds_max {
        ticket.senior_attempt += 1;

        let escalation =
            ticket_service::senior_escalate(&current_answer, &ticket, &junior_history, probe_results);

        add_senior_escalation_event(transcript, elapsed_ms, &escalation);

        if escalation.successful && escalation.instruction.has_changes() {
            // Apply senior revision
            let (revised, changes) =
                ticket_service::apply_revision(&current_answer, &escalation.instruction, probe_results);

            if !changes.is_empty() {
                add_revision_event(transcript, elapsed_ms, changes);
                current_answer = revised;
            }

            // Re-verify with junior after senior guidance
            let final_verification = ticket_service::junior_verify(
                &current_answer,
                &ticket,
                probe_results,
                user_request,
                reliability_input,
                &config,
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
                    &ticket,
                    TicketStatus::Escalated,
                    TicketStatus::Verified,
                );

                return TicketLoopResult {
                    answer: current_answer,
                    ticket,
                    verified: true,
                    score: final_verification.score,
                };
            }
        } else {
            warn!("Senior escalation did not provide useful guidance");
        }
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

    // Return with last known score from junior history
    let last_score = junior_history.last().map(|v| v.score).unwrap_or(0);

    TicketLoopResult {
        answer: current_answer,
        ticket,
        verified: false,
        score: last_score,
    }
}

/// Derive evidence kinds from route class (for ticket creation)
fn evidence_kinds_from_route(route_class: &str) -> Vec<EvidenceKind> {
    match route_class {
        "MemoryUsage" | "MemoryInfo" | "memory_usage" | "ram_info" => vec![EvidenceKind::Memory],
        "DiskUsage" | "DiskInfo" | "disk_usage" | "disk_space" => vec![EvidenceKind::Disk],
        "CpuInfo" | "CpuUsage" | "cpu_info" => vec![EvidenceKind::Cpu],
        "SystemServices" | "ServiceStatus" | "service_status" => vec![EvidenceKind::Services],
        "BlockDevices" | "lsblk" => vec![EvidenceKind::BlockDevices],
        "SystemHealth" | "system_health_summary" => vec![
            EvidenceKind::Memory,
            EvidenceKind::Disk,
            EvidenceKind::Cpu,
        ],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::rpc::{QueryIntent, SpecialistDomain};

    fn make_test_ticket() -> TranslatorTicket {
        TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec!["memory".to_string()],
            needs_probes: vec!["free -h".to_string()],
            clarification_question: None,
            answer_contract: None,
            confidence: 0.9,
        }
    }

    fn make_high_reliability_input() -> ReliabilityInput {
        ReliabilityInput {
            planned_probes: 1,
            succeeded_probes: 1,
            failed_probes: 0,
            timed_out_probes: 0,
            translator_confidence: 0.95,
            translator_used: true,
            answer_grounded: true,
            no_invention: true,
            grounding_ratio: 1.0,
            total_claims: 1,
            evidence_required: true,
            used_deterministic: false,
            parsed_data_count: 0,
            prompt_truncated: false,
            transcript_capped: false,
            budget_exceeded: false,
            exceeded_stage: None,
            stage_budget_ms: 0,
            stage_elapsed_ms: 0,
            used_deterministic_fallback: false,
            fallback_route_class: String::new(),
            evidence_kinds: vec![],
            specialist_outcome: None,
            fallback_used: None,
        }
    }

    #[test]
    fn test_ticket_loop_passes_high_reliability() {
        let translator_ticket = make_test_ticket();
        let reliability_input = make_high_reliability_input();
        let mut transcript = Transcript::new();

        let result = run_ticket_loop(
            "test-123",
            "how much memory do I have?",
            "You have 16GB of RAM with 8GB available.",
            &translator_ticket,
            "MemoryUsage",
            &[],
            &reliability_input,
            &mut transcript,
            100,
            None,
        );

        assert!(result.verified);
        assert_eq!(result.ticket.status, TicketStatus::Verified);
        assert!(result.score >= 80);
    }

    #[test]
    fn test_evidence_kinds_mapping() {
        assert_eq!(
            evidence_kinds_from_route("MemoryUsage"),
            vec![EvidenceKind::Memory]
        );
        assert_eq!(
            evidence_kinds_from_route("DiskUsage"),
            vec![EvidenceKind::Disk]
        );
        assert_eq!(
            evidence_kinds_from_route("service_status"),
            vec![EvidenceKind::Services]
        );
        assert!(evidence_kinds_from_route("Unknown").is_empty());
    }
}
