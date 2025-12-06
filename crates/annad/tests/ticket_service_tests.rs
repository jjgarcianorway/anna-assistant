//! Tests for ticket service with review gate integration.

use anna_shared::reliability::ReliabilityInput;
use anna_shared::review::ReviewDecision;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use anna_shared::teams::Team;
use anna_shared::ticket::{RiskLevel, Ticket};
use anna_shared::trace::{EvidenceKind, ReviewerOutcome};
use anna_shared::transcript::Transcript;
use annad::ticket_service::{
    build_review_context, junior_verify, run_review_gate, TicketServiceConfig,
};

/// Helper to create a basic reliability input
fn make_reliability_input(
    score_params: (bool, bool, f32, u32), // (no_invention, grounded, grounding_ratio, claims)
    evidence_required: bool,
) -> ReliabilityInput {
    ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 1,
        failed_probes: 0,
        timed_out_probes: 0,
        translator_confidence: 0.95,
        translator_used: true,
        answer_grounded: score_params.1,
        no_invention: score_params.0,
        grounding_ratio: score_params.2,
        total_claims: score_params.3,
        evidence_required,
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
fn test_create_ticket_from_translator() {
    use annad::ticket_service::create_ticket_from_translator;

    let translator_ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec!["memory".to_string()],
        needs_probes: vec!["free -h".to_string()],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.95,
    };

    let ticket = create_ticket_from_translator(
        "test-123",
        "how much memory do I have?",
        &translator_ticket,
        "MemoryUsage",
        vec![EvidenceKind::Memory],
    );

    assert_eq!(ticket.ticket_id, "test-123");
    assert_eq!(ticket.domain, "system");
    assert_eq!(ticket.intent, "question");
    // "MemoryUsage" route class maps to Performance team (memory routes)
    assert_eq!(ticket.team, Team::Performance);
    assert!(ticket.evidence_required);
    assert_eq!(ticket.risk_level, RiskLevel::ReadOnly);
}

#[test]
fn test_junior_verification_passes() {
    let config = TicketServiceConfig::default();
    let ticket = Ticket::default();
    let input = make_reliability_input((true, true, 1.0, 1), true);

    let result = junior_verify("test answer", &ticket, &[], "test query", &input, &config);
    assert!(result.verified);
}

#[test]
fn test_review_gate_accepts_high_score() {
    let input = make_reliability_input((true, true, 0.9, 3), true);

    let mut transcript = Transcript::new();
    let (outcome, reviewer_outcome) = run_review_gate(&input, &mut transcript, 100);

    assert_eq!(outcome.decision, ReviewDecision::Accept);
    assert!(!outcome.requires_llm_review);
    assert_eq!(reviewer_outcome, ReviewerOutcome::DeterministicAccept);
    assert_eq!(transcript.len(), 1); // Gate decision event added
}

#[test]
fn test_review_gate_escalates_invention() {
    let input = make_reliability_input((false, true, 0.9, 3), true); // invention detected

    let mut transcript = Transcript::new();
    let (outcome, reviewer_outcome) = run_review_gate(&input, &mut transcript, 100);

    assert_eq!(outcome.decision, ReviewDecision::EscalateToSenior);
    assert!(!outcome.requires_llm_review);
    assert_eq!(reviewer_outcome, ReviewerOutcome::DeterministicReject);
}

#[test]
fn test_review_context_from_input() {
    let input = make_reliability_input((true, true, 0.8, 5), true);
    let ctx = build_review_context(&input);

    assert!(!ctx.invention_detected);
    assert_eq!(ctx.grounding_ratio, 0.8);
    assert_eq!(ctx.total_claims, 5);
    assert!(ctx.evidence_required);
    assert!(!ctx.budget_exceeded);
}

#[test]
fn test_review_gate_revises_low_grounding() {
    let input = make_reliability_input((true, true, 0.3, 5), true); // low grounding

    let mut transcript = Transcript::new();
    let (outcome, _) = run_review_gate(&input, &mut transcript, 100);

    assert_eq!(outcome.decision, ReviewDecision::Revise);
    assert!(!outcome.requires_llm_review);
}

#[test]
fn test_review_gate_unclear_for_medium_score() {
    // Medium score (50-79) triggers LLM review when signals are mixed
    // Use low translator confidence + some probe failures to get medium score
    let input = ReliabilityInput {
        planned_probes: 2,
        succeeded_probes: 1,
        failed_probes: 1, // One failure to lower score
        timed_out_probes: 0,
        translator_confidence: 0.5, // Lower confidence
        translator_used: true,
        answer_grounded: true,
        no_invention: true,
        grounding_ratio: 0.7,
        total_claims: 2,
        evidence_required: false, // No evidence required so grounding check is skipped
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
    };

    let ctx = build_review_context(&input);
    // Verify we're in the medium score range (50-79)
    assert!(
        ctx.reliability_score >= 50 && ctx.reliability_score < 80,
        "Expected score in 50-79 range, got {}",
        ctx.reliability_score
    );

    let mut transcript = Transcript::new();
    let (outcome, _) = run_review_gate(&input, &mut transcript, 100);

    // Medium score → unclear → LLM review
    assert!(outcome.requires_llm_review);
}
