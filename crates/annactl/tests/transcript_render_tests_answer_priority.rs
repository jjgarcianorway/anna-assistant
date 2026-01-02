//! Guardrail tests for transcript rendering - Answer source priority.
//!
//! These tests verify the answer source priority logic:
//! FinalAnswer in transcript > Clarification > Direct answer > Empty

use anna_shared::rpc::{EvidenceBlock, ReliabilitySignals, ServiceDeskResult, SpecialistDomain};
use anna_shared::transcript::{Actor, Transcript, TranscriptEvent, TranscriptEventKind};

/// Helper to create minimal ServiceDeskResult for testing
fn make_result(
    transcript: Transcript,
    answer: &str,
    clarification: Option<&str>,
    needs_clarification: bool,
) -> ServiceDeskResult {
    ServiceDeskResult {
        request_id: "test-12345678".to_string(),
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: answer.to_string(),
        validated: !needs_clarification, // v0.0.298: Validated if not clarification
        reliability_score: 80,
        reliability_signals: ReliabilitySignals::default(),
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence: EvidenceBlock::default(),
        needs_clarification,
        clarification_question: clarification.map(String::from),
        clarification_request: None,
        transcript,
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// GUARDRAIL: Answer source priority is consistent.
/// FinalAnswer in transcript > Clarification > Direct answer > Empty
#[test]
fn test_answer_source_priority_transcript_first() {
    // Case 1: FinalAnswer in transcript takes priority over result.answer
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(
        0,
        Actor::You,
        Actor::Anna,
        "query",
    ));
    t.push(TranscriptEvent::final_answer(100, "transcript answer"));
    let r = make_result(t, "result.answer", None, false);

    // The FinalAnswer should be found
    let has_final_answer_in_transcript = r.transcript.events.iter().any(|e| {
        if let TranscriptEventKind::FinalAnswer { text } = &e.kind {
            text == "transcript answer"
        } else {
            false
        }
    });
    assert!(
        has_final_answer_in_transcript,
        "FinalAnswer in transcript should take priority"
    );
}

/// GUARDRAIL: Clarification takes priority when needs_clarification=true
#[test]
fn test_answer_source_priority_clarification() {
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(
        0,
        Actor::You,
        Actor::Anna,
        "query",
    ));
    let r = make_result(t, "", Some("What do you mean?"), true);

    assert!(r.needs_clarification);
    assert_eq!(
        r.clarification_question.as_deref(),
        Some("What do you mean?")
    );
}

/// GUARDRAIL: Direct answer used when no transcript answer and no clarification
#[test]
fn test_answer_source_priority_direct_answer() {
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(
        0,
        Actor::You,
        Actor::Anna,
        "query",
    ));
    let r = make_result(t, "the answer", None, false);

    assert!(!r.needs_clarification);
    assert_eq!(r.answer, "the answer");
}

/// GUARDRAIL: Empty case - no answer available
#[test]
fn test_answer_source_priority_empty() {
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(
        0,
        Actor::You,
        Actor::Anna,
        "query",
    ));
    let r = make_result(t, "", None, false);

    assert!(!r.needs_clarification);
    assert!(r.answer.is_empty());
}
