//! Golden tests for ticket flow verification.
//!
//! Tests the full ticket lifecycle with bounded verification
//! and escalation paths (v0.0.25 TICKETS).

use anna_shared::reliability::ReliabilityInput;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use anna_shared::ticket::TicketStatus;
use anna_shared::trace::EvidenceKind;
use anna_shared::transcript::Transcript;

// Import ticket_loop from annad
use annad::ticket_loop::run_ticket_loop;
use annad::ticket_service::TicketServiceConfig;

/// Helper to create a test translator ticket
fn make_translator_ticket(
    domain: SpecialistDomain,
    intent: QueryIntent,
    confidence: f32,
    probes: Vec<&str>,
) -> TranslatorTicket {
    TranslatorTicket {
        intent,
        domain,
        entities: vec![],
        needs_probes: probes.iter().map(|s| s.to_string()).collect(),
        clarification_question: None,
        confidence,
        answer_contract: None,
    }
}

/// Helper to create high-reliability input (score >= 80)
fn high_reliability_input() -> ReliabilityInput {
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

/// Helper to create low-reliability input (score < 80)
fn low_reliability_input() -> ReliabilityInput {
    ReliabilityInput {
        planned_probes: 2,
        succeeded_probes: 0,  // No probes succeeded
        failed_probes: 2,
        timed_out_probes: 0,
        translator_confidence: 0.5,  // Low confidence
        translator_used: true,
        answer_grounded: false,  // Not grounded
        no_invention: false,  // Invention detected (hard cap at 40)
        grounding_ratio: 0.0,
        total_claims: 0,
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

// =============================================================================
// GOLDEN TEST 1: High-reliability answer verified on first attempt
// =============================================================================

#[test]
fn test_ticket_flow_high_reliability_verified_first_attempt() {
    let ticket = make_translator_ticket(
        SpecialistDomain::System,
        QueryIntent::Question,
        0.95,
        vec!["free -h"],
    );
    let reliability = high_reliability_input();
    let mut transcript = Transcript::new();

    let result = run_ticket_loop(
        "test-high-rel-001",
        "how much memory do I have?",
        "You have 16GB of RAM with 8GB available.",
        &ticket,
        "MemoryUsage",
        &[],
        &reliability,
        &mut transcript,
        100,
        None,
    );

    // ASSERT: Verified on first junior attempt
    assert!(result.verified, "High-reliability answer should be verified");
    assert_eq!(result.ticket.status, TicketStatus::Verified);
    assert_eq!(result.ticket.junior_attempt, 1, "Should verify on first attempt");
    assert_eq!(result.ticket.senior_attempt, 0, "Should not escalate to senior");
    assert!(result.score >= 80, "Score should be >= 80");
}

// =============================================================================
// GOLDEN TEST 2: Low-reliability exhausts junior rounds then escalates
// =============================================================================

#[test]
fn test_ticket_flow_low_reliability_exhausts_junior() {
    let ticket = make_translator_ticket(
        SpecialistDomain::System,
        QueryIntent::Question,
        0.5,
        vec!["free -h", "df -h"],
    );
    let reliability = low_reliability_input();
    let mut transcript = Transcript::new();

    // Use config with known limits
    let config = TicketServiceConfig {
        reliability_threshold: 80,
        junior_rounds_max: 3,
        senior_rounds_max: 1,
    };

    let result = run_ticket_loop(
        "test-low-rel-001",
        "how much memory do I have?",
        "I think you might have some memory.", // Vague answer
        &ticket,
        "MemoryUsage",
        &[],
        &reliability,
        &mut transcript,
        100,
        Some(config),
    );

    // ASSERT: Exhausted all rounds, escalated, and failed
    assert!(!result.verified, "Low-reliability answer should not be verified");
    assert_eq!(result.ticket.status, TicketStatus::Failed);
    assert_eq!(result.ticket.junior_attempt, 3, "Should exhaust all junior rounds");
    assert_eq!(result.ticket.senior_attempt, 1, "Should attempt senior escalation");
}

// =============================================================================
// GOLDEN TEST 3: Ticket lifecycle events in transcript
// =============================================================================

#[test]
fn test_ticket_flow_transcript_events() {
    use anna_shared::transcript::TranscriptEventKind;

    let ticket = make_translator_ticket(
        SpecialistDomain::System,
        QueryIntent::Question,
        0.95,
        vec!["free -h"],
    );
    let reliability = high_reliability_input();
    let mut transcript = Transcript::new();

    let _ = run_ticket_loop(
        "test-transcript-001",
        "how much memory?",
        "16GB RAM, 8GB available",
        &ticket,
        "MemoryUsage",
        &[],
        &reliability,
        &mut transcript,
        100,
        None,
    );

    // ASSERT: Transcript contains ticket events
    let has_ticket_created = transcript.events.iter().any(|e| {
        matches!(&e.kind, TranscriptEventKind::TicketCreated { .. })
    });
    let has_junior_review = transcript.events.iter().any(|e| {
        matches!(&e.kind, TranscriptEventKind::JuniorReview { .. })
    });
    let has_status_change = transcript.events.iter().any(|e| {
        matches!(&e.kind, TranscriptEventKind::TicketStatusChanged { .. })
    });

    assert!(has_ticket_created, "Transcript should have TicketCreated event");
    assert!(has_junior_review, "Transcript should have JuniorReview event");
    assert!(has_status_change, "Transcript should have status change event");
}

// =============================================================================
// GOLDEN TEST 4: Evidence kinds mapping
// =============================================================================

#[test]
fn test_ticket_evidence_kinds_populated() {
    let ticket = make_translator_ticket(
        SpecialistDomain::System,
        QueryIntent::Question,
        0.95,
        vec!["free -h"],
    );
    let reliability = high_reliability_input();
    let mut transcript = Transcript::new();

    let result = run_ticket_loop(
        "test-evidence-001",
        "how much memory?",
        "16GB RAM",
        &ticket,
        "MemoryUsage",
        &[],
        &reliability,
        &mut transcript,
        100,
        None,
    );

    // ASSERT: Evidence kinds populated correctly
    assert!(
        result.ticket.evidence_kinds.contains(&EvidenceKind::Memory),
        "MemoryUsage route should have Memory evidence kind"
    );
}

// =============================================================================
// GOLDEN TEST 5: Bounded iteration (junior_rounds_max respected)
// =============================================================================

#[test]
fn test_ticket_bounded_iteration() {
    let ticket = make_translator_ticket(
        SpecialistDomain::System,
        QueryIntent::Question,
        0.5,
        vec![],
    );
    let reliability = low_reliability_input();
    let mut transcript = Transcript::new();

    // Test with custom limits
    let config = TicketServiceConfig {
        reliability_threshold: 80,
        junior_rounds_max: 2,  // Only 2 junior rounds
        senior_rounds_max: 0,  // No senior escalation
    };

    let result = run_ticket_loop(
        "test-bounded-001",
        "test query",
        "vague answer",
        &ticket,
        "Unknown",
        &[],
        &reliability,
        &mut transcript,
        100,
        Some(config),
    );

    // ASSERT: Bounded by config limits
    assert_eq!(result.ticket.junior_attempt, 2, "Should stop at junior_rounds_max");
    assert_eq!(result.ticket.senior_attempt, 0, "Should not escalate when senior_rounds_max=0");
    assert_eq!(result.ticket.status, TicketStatus::Failed);
}

// =============================================================================
// GOLDEN TEST 6: Ticket domain and intent from translator
// =============================================================================

#[test]
fn test_ticket_inherits_translator_classification() {
    let ticket = make_translator_ticket(
        SpecialistDomain::Network,
        QueryIntent::Investigate,
        0.9,
        vec!["ip addr show"],
    );
    let reliability = high_reliability_input();
    let mut transcript = Transcript::new();

    let result = run_ticket_loop(
        "test-classify-001",
        "show network interfaces",
        "eth0: 192.168.1.100",
        &ticket,
        "NetworkInfo",
        &[],
        &reliability,
        &mut transcript,
        100,
        None,
    );

    // ASSERT: Ticket inherits classification from translator
    assert_eq!(result.ticket.domain, "network");
    assert_eq!(result.ticket.intent, "investigate");
}
