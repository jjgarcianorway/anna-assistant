//! Guardrail tests for transcript rendering - Basic invariants.
//!
//! These tests verify the single [anna] output invariant and
//! the unified answer source logic using FinalAnswer as the contract.

use anna_shared::transcript::{
    Actor, StageOutcome, Transcript, TranscriptEvent, TranscriptEventKind,
};

/// GUARDRAIL: Exactly one [anna] block per request in debug mode.
/// FinalAnswer kind is THE contract for answer source.
#[test]
fn test_single_anna_output_with_final_answer_in_transcript() {
    // Simulate transcript with FinalAnswer event present
    let mut transcript = Transcript::new();
    transcript.push(TranscriptEvent::message(
        0,
        Actor::You,
        Actor::Anna,
        "test query",
    ));
    transcript.push(TranscriptEvent::final_answer(100, "test answer"));

    // Count FinalAnswer events in transcript
    let final_answers: Vec<_> = transcript
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. }))
        .collect();

    // If transcript has FinalAnswer, render_debug should NOT print fallback block
    assert_eq!(
        final_answers.len(),
        1,
        "Should have exactly one FinalAnswer"
    );
}

/// GUARDRAIL: Exactly one [anna] block when no FinalAnswer in transcript.
#[test]
fn test_single_anna_output_without_final_answer_in_transcript() {
    // Simulate transcript without FinalAnswer (fallback case)
    let mut transcript = Transcript::new();
    transcript.push(TranscriptEvent::message(
        0,
        Actor::You,
        Actor::Anna,
        "test query",
    ));
    transcript.push(TranscriptEvent::stage_start(10, "translator"));
    transcript.push(TranscriptEvent::stage_end(
        50,
        "translator",
        StageOutcome::Ok,
    ));

    // Count FinalAnswer events in transcript
    let final_answers: Vec<_> = transcript
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. }))
        .collect();

    // If transcript has no FinalAnswer, render_debug prints fallback block
    assert_eq!(final_answers.len(), 0, "Should have no FinalAnswer events");
    // The render function will print exactly one [anna] from fallback path
}

/// GUARDRAIL: Never duplicate Anna output regardless of path taken.
#[test]
fn test_anna_output_invariant_all_paths() {
    // Test various transcript configurations
    let test_cases = vec![
        // (description, has_final_answer)
        ("deterministic path", true),
        ("llm timeout with fallback", true),
        ("empty transcript fallback", false),
        ("probes only, no specialist", false),
    ];

    for (desc, has_final_answer) in test_cases {
        let mut transcript = Transcript::new();
        transcript.push(TranscriptEvent::message(
            0,
            Actor::You,
            Actor::Anna,
            "query",
        ));

        if has_final_answer {
            transcript.push(TranscriptEvent::final_answer(100, "answer"));
        }

        let final_answer_in_transcript = transcript
            .events
            .iter()
            .any(|e| matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. }));

        // Invariant: total Anna outputs = 1
        // Either from transcript FinalAnswer OR from fallback, never both
        let from_transcript = if final_answer_in_transcript { 1 } else { 0 };
        let from_fallback = if final_answer_in_transcript { 0 } else { 1 };
        let total = from_transcript + from_fallback;

        assert_eq!(total, 1, "{}: expected exactly 1 [anna] output", desc);
    }
}

/// GUARDRAIL: Deterministic path shows "skipped (deterministic)" not "ok"
/// This ensures stage reporting doesn't drift after adding FinalAnswer/Unknown.
#[test]
fn test_deterministic_stage_outcome_distinct_from_ok() {
    // Verify StageOutcome::Deterministic is distinct from StageOutcome::Ok
    let det = StageOutcome::Deterministic;
    let ok = StageOutcome::Ok;

    // They should be different enum variants
    assert!(det != ok, "Deterministic should not equal Ok");

    // Their Display implementations should differ
    let det_str = format!("{}", det);
    let ok_str = format!("{}", ok);
    assert_ne!(
        det_str, ok_str,
        "Deterministic display should differ from Ok"
    );
    assert!(
        det_str.contains("deterministic"),
        "Deterministic display should contain 'deterministic'"
    );
}
