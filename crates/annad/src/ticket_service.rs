//! Ticket service with bounded verification and escalation.
//!
//! Wraps the existing service desk pipeline with:
//! - Ticket lifecycle management
//! - Deterministic review gate (v0.0.26)
//! - Junior verification with bounded retries (team-specialized v0.0.25)
//! - Senior escalation when junior rounds exhausted
//! - Full transcript visibility

use anna_shared::reliability::{compute_reliability, ReliabilityInput, ReliabilityOutput};
use anna_shared::review::ReviewDecision;
use anna_shared::review_gate::{deterministic_review_gate, GateOutcome, ReviewContext};
use anna_shared::revision::{
    JuniorVerification, RevisionInstruction, RevisionIssue, SeniorEscalation,
};
use anna_shared::rpc::ProbeResult;
use anna_shared::teams::team_from_domain_intent;
use anna_shared::ticket::{
    RiskLevel, Ticket, TicketStatus, DEFAULT_JUNIOR_ROUNDS_MAX, DEFAULT_RELIABILITY_THRESHOLD,
    DEFAULT_SENIOR_ROUNDS_MAX,
};
use anna_shared::trace::{EvidenceKind, ReviewerOutcome};
use anna_shared::transcript::{Transcript, TranscriptEvent};
use anna_shared::transcript_ext::TranscriptEventExt;
use tracing::{debug, info, warn};

/// Ticket service configuration
pub struct TicketServiceConfig {
    /// Reliability threshold for verification (default 80)
    pub reliability_threshold: u8,
    /// Maximum junior verification rounds
    pub junior_rounds_max: u8,
    /// Maximum senior escalation rounds
    pub senior_rounds_max: u8,
}

impl Default for TicketServiceConfig {
    fn default() -> Self {
        Self {
            reliability_threshold: DEFAULT_RELIABILITY_THRESHOLD,
            junior_rounds_max: DEFAULT_JUNIOR_ROUNDS_MAX,
            senior_rounds_max: DEFAULT_SENIOR_ROUNDS_MAX,
        }
    }
}

/// Build ReviewContext from ReliabilityInput (v0.0.26)
pub fn build_review_context(input: &ReliabilityInput) -> ReviewContext {
    ReviewContext::new(compute_reliability(input).score)
        .with_grounding(input.grounding_ratio, input.total_claims)
        .with_guard(!input.no_invention, 0, 0)
        .with_evidence_required(input.evidence_required)
        .with_budget_exceeded(input.budget_exceeded)
}

/// Run deterministic review gate (v0.0.26)
/// Returns gate outcome and optional reviewer outcome for trace
pub fn run_review_gate(
    input: &ReliabilityInput,
    transcript: &mut Transcript,
    elapsed_ms: u64,
) -> (GateOutcome, ReviewerOutcome) {
    let ctx = build_review_context(input);
    let outcome = deterministic_review_gate(&ctx);

    // Add transcript event
    let decision_str = match outcome.decision {
        ReviewDecision::Accept => "accept",
        ReviewDecision::Revise => "revise",
        ReviewDecision::EscalateToSenior => "escalate",
        ReviewDecision::ClarifyUser => "clarify",
    };

    transcript.push(TranscriptEvent::review_gate_decision(
        elapsed_ms,
        decision_str,
        ctx.reliability_score,
        outcome.requires_llm_review,
    ));

    // Map to reviewer outcome for trace
    let reviewer_outcome = match (&outcome.decision, outcome.requires_llm_review) {
        (ReviewDecision::Accept, false) => ReviewerOutcome::DeterministicAccept,
        (ReviewDecision::Revise, false) | (ReviewDecision::EscalateToSenior, false) => {
            ReviewerOutcome::DeterministicReject
        }
        (_, true) => ReviewerOutcome::JuniorOk, // Will be updated by LLM review if needed
        (ReviewDecision::ClarifyUser, _) => ReviewerOutcome::DeterministicReject,
    };

    debug!(
        "Review gate: decision={}, score={}, requires_llm={}",
        decision_str, ctx.reliability_score, outcome.requires_llm_review
    );

    (outcome, reviewer_outcome)
}

/// Create a ticket from translator output
pub fn create_ticket_from_translator(
    request_id: &str,
    user_request: &str,
    ticket: &anna_shared::rpc::TranslatorTicket,
    route_class: &str,
    evidence_kinds: Vec<EvidenceKind>,
) -> Ticket {
    // Determine risk level based on intent
    let risk_level = match ticket.intent {
        anna_shared::rpc::QueryIntent::Question => RiskLevel::ReadOnly,
        anna_shared::rpc::QueryIntent::Investigate => RiskLevel::ReadOnly,
        anna_shared::rpc::QueryIntent::Request => RiskLevel::LowRiskChange,
    };

    // Determine team from domain, intent, and route class (v0.0.25)
    let team = team_from_domain_intent(
        &ticket.domain.to_string(),
        &ticket.intent.to_string(),
        route_class,
    );

    Ticket::new(
        request_id.to_string(),
        user_request.to_string(),
        ticket.domain.to_string(),
        ticket.intent.to_string(),
        team,
        route_class.to_string(),
        !ticket.needs_probes.is_empty(), // evidence_required if probes needed
        ticket.needs_probes.clone(),
        evidence_kinds,
        risk_level,
    )
}

/// Junior verification using existing reliability pipeline
pub fn junior_verify(
    answer: &str,
    ticket: &Ticket,
    probe_results: &[ProbeResult],
    _query: &str,
    reliability_input: &ReliabilityInput,
    config: &TicketServiceConfig,
) -> JuniorVerification {
    // Use existing reliability computation
    let output = compute_reliability(reliability_input);

    if output.score >= config.reliability_threshold {
        info!(
            "Junior verified: score={} >= threshold={}",
            output.score, config.reliability_threshold
        );
        return JuniorVerification::verified(output.score);
    }

    // Build revision instructions based on reliability issues
    let instruction = build_revision_instruction(&output, ticket, probe_results, answer);

    info!(
        "Junior needs revision: score={} < threshold={}, issues={:?}",
        output.score, config.reliability_threshold, instruction.issues
    );

    JuniorVerification::needs_revision(output.score, instruction)
}

/// Build revision instruction from reliability output
fn build_revision_instruction(
    output: &ReliabilityOutput,
    ticket: &Ticket,
    probe_results: &[ProbeResult],
    _answer: &str,
) -> RevisionInstruction {
    let mut inst = RevisionInstruction::default();

    // Map reliability reasons to revision issues
    for reason in &output.reasons {
        match reason {
            anna_shared::reliability::ReliabilityReason::ProbeFailed
            | anna_shared::reliability::ReliabilityReason::ProbeTimeout => {
                inst = inst.with_issue(RevisionIssue::MissingProbes);
                // Recommend re-running failed probes
                for probe in &ticket.planned_probes {
                    if !probe_results.iter().any(|p| p.command == *probe && p.exit_code == 0) {
                        inst = inst.with_recommended_probe(probe.clone());
                    }
                }
            }
            anna_shared::reliability::ReliabilityReason::LowConfidence => {
                inst = inst.with_issue(RevisionIssue::TooVague);
            }
            anna_shared::reliability::ReliabilityReason::InventionDetected => {
                inst = inst.with_issue(RevisionIssue::UnverifiableClaims);
            }
            anna_shared::reliability::ReliabilityReason::NotGrounded => {
                inst = inst.with_issue(RevisionIssue::MissingEvidence);
                // Require claims based on evidence kinds
                for kind in &ticket.evidence_kinds {
                    inst = inst.with_required_claim(format!("include {} data", kind));
                }
            }
            anna_shared::reliability::ReliabilityReason::EvidenceMissing => {
                inst = inst.with_issue(RevisionIssue::MissingEvidence);
            }
            _ => {}
        }
    }

    // Add explanation
    inst.explanation = Some(format!(
        "score {} below threshold, {} issues identified",
        output.score,
        inst.issues.len()
    ));

    inst
}

/// Senior escalation - provides higher-level revision guidance
pub fn senior_escalate(
    _answer: &str,
    ticket: &Ticket,
    junior_history: &[JuniorVerification],
    _probe_results: &[ProbeResult],
) -> SeniorEscalation {
    // Analyze why junior verification keeps failing
    let recurring_issues: Vec<RevisionIssue> = junior_history
        .iter()
        .flat_map(|v| v.instruction.issues.clone())
        .collect();

    if recurring_issues.is_empty() {
        return SeniorEscalation::failed("no issues identified");
    }

    // Build senior-level instruction
    let mut inst = RevisionInstruction::default();

    // Senior focuses on structural issues
    if recurring_issues.contains(&RevisionIssue::MissingEvidence) {
        inst = inst
            .with_issue(RevisionIssue::MissingEvidence)
            .with_explanation("Evidence-based claims required");

        // Be more specific about required evidence
        for kind in &ticket.evidence_kinds {
            inst = inst.with_required_claim(format!("auditable {} claim with specific values", kind));
        }
    }

    if recurring_issues.contains(&RevisionIssue::UnverifiableClaims) {
        inst = inst
            .with_issue(RevisionIssue::UnverifiableClaims)
            .with_forbidden_claim("claims without probe data support");
    }

    if recurring_issues.contains(&RevisionIssue::TooVague) {
        inst = inst
            .with_issue(RevisionIssue::TooVague)
            .with_required_claim("specific values from probe output");
    }

    if inst.has_changes() {
        info!("Senior escalation successful with {} issues", inst.issues.len());
        SeniorEscalation::success(inst)
    } else {
        warn!("Senior escalation could not provide guidance");
        SeniorEscalation::failed("unable to provide actionable guidance")
    }
}

/// Apply revision instruction to answer (deterministic edits only)
pub fn apply_revision(
    answer: &str,
    instruction: &RevisionInstruction,
    _probe_results: &[ProbeResult],
) -> (String, Vec<String>) {
    let mut revised = answer.to_string();
    let mut changes = Vec::new();

    // Remove forbidden claims (simple string replacement)
    for forbidden in &instruction.forbidden_claims {
        if revised.contains(forbidden) {
            revised = revised.replace(forbidden, "[removed]");
            changes.push(format!("removed: {}", forbidden));
        }
    }

    // Note: In a full implementation, we would:
    // - Parse claims from the answer
    // - Add required claims from probe data
    // - Re-format the answer
    //
    // For now, we track that revision was attempted
    if !instruction.required_claims.is_empty() {
        changes.push(format!(
            "flagged {} required claims",
            instruction.required_claims.len()
        ));
    }

    (revised, changes)
}

/// Add ticket lifecycle events to transcript
pub fn add_ticket_created_event(
    transcript: &mut Transcript,
    elapsed_ms: u64,
    ticket: &Ticket,
) {
    transcript.push(TranscriptEvent::ticket_created(
        elapsed_ms,
        &ticket.ticket_id,
        &ticket.domain,
        &ticket.intent,
        ticket.evidence_required,
    ));
}

/// Add status change event to transcript
pub fn add_status_change_event(
    transcript: &mut Transcript,
    elapsed_ms: u64,
    ticket: &Ticket,
    from_status: TicketStatus,
    to_status: TicketStatus,
) {
    transcript.push(TranscriptEvent::ticket_status_changed(
        elapsed_ms,
        &ticket.ticket_id,
        from_status.to_string(),
        to_status.to_string(),
    ));
}

/// Add junior review event to transcript
pub fn add_junior_review_event(
    transcript: &mut Transcript,
    elapsed_ms: u64,
    attempt: u8,
    verification: &JuniorVerification,
) {
    let issues: Vec<String> = verification
        .instruction
        .issues
        .iter()
        .map(|i| i.to_string())
        .collect();

    transcript.push(TranscriptEvent::junior_review(
        elapsed_ms,
        attempt,
        verification.score,
        verification.verified,
        issues,
    ));
}

/// Add senior escalation event to transcript
pub fn add_senior_escalation_event(
    transcript: &mut Transcript,
    elapsed_ms: u64,
    escalation: &SeniorEscalation,
) {
    transcript.push(TranscriptEvent::senior_escalation(
        elapsed_ms,
        escalation.successful,
        escalation.reason.clone(),
    ));
}

/// Add revision applied event to transcript
pub fn add_revision_event(
    transcript: &mut Transcript,
    elapsed_ms: u64,
    changes: Vec<String>,
) {
    transcript.push(TranscriptEvent::revision_applied(elapsed_ms, changes));
}

// Tests moved to tests/ticket_service_tests.rs to keep this file under 400 lines
