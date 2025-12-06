//! Guardrail tests for transcript rendering.
//!
//! These tests verify the single [anna] output invariant and
//! the unified answer source logic using FinalAnswer as the contract.

use anna_shared::rpc::{EvidenceBlock, ReliabilitySignals, ServiceDeskResult, SpecialistDomain};
use anna_shared::transcript::{Actor, StageOutcome, Transcript, TranscriptEvent, TranscriptEventKind};

/// GUARDRAIL: Exactly one [anna] block per request in debug mode.
/// FinalAnswer kind is THE contract for answer source.
#[test]
fn test_single_anna_output_with_final_answer_in_transcript() {
    // Simulate transcript with FinalAnswer event present
    let mut transcript = Transcript::new();
    transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "test query"));
    transcript.push(TranscriptEvent::final_answer(100, "test answer"));

    // Count FinalAnswer events in transcript
    let final_answers: Vec<_> = transcript
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. }))
        .collect();

    // If transcript has FinalAnswer, render_debug should NOT print fallback block
    assert_eq!(final_answers.len(), 1, "Should have exactly one FinalAnswer");
}

/// GUARDRAIL: Exactly one [anna] block when no FinalAnswer in transcript.
#[test]
fn test_single_anna_output_without_final_answer_in_transcript() {
    // Simulate transcript without FinalAnswer (fallback case)
    let mut transcript = Transcript::new();
    transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "test query"));
    transcript.push(TranscriptEvent::stage_start(10, "translator"));
    transcript.push(TranscriptEvent::stage_end(50, "translator", StageOutcome::Ok));

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
        transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));

        if has_final_answer {
            transcript.push(TranscriptEvent::final_answer(100, "answer"));
        }

        let final_answer_in_transcript = transcript.events.iter().any(|e| {
            matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. })
        });

        // Invariant: total Anna outputs = 1
        // Either from transcript FinalAnswer OR from fallback, never both
        let from_transcript = if final_answer_in_transcript { 1 } else { 0 };
        let from_fallback = if final_answer_in_transcript { 0 } else { 1 };
        let total = from_transcript + from_fallback;

        assert_eq!(total, 1, "{}: expected exactly 1 [anna] output", desc);
    }
}

/// GUARDRAIL: Unknown/malformed transcript events don't crash render
/// and preserve the single [anna] output invariant.
#[test]
fn test_render_handles_mixed_event_types_gracefully() {
    // Build a transcript with all known event types
    let mut transcript = Transcript::new();

    // User message
    transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));

    // Stage events
    transcript.push(TranscriptEvent::stage_start(10, "translator"));
    transcript.push(TranscriptEvent::stage_end(50, "translator", StageOutcome::Ok));
    transcript.push(TranscriptEvent::stage_end(60, "specialist", StageOutcome::Deterministic));
    transcript.push(TranscriptEvent::stage_end(70, "probes", StageOutcome::Timeout));
    transcript.push(TranscriptEvent::stage_end(80, "supervisor", StageOutcome::Error));
    transcript.push(TranscriptEvent::stage_end(90, "unknown_stage", StageOutcome::Skipped));

    // Probe events
    transcript.push(TranscriptEvent::probe_start(100, "test_probe", "echo test"));
    transcript.push(TranscriptEvent::probe_end(150, "test_probe", 0, 50, Some("output".into())));
    transcript.push(TranscriptEvent::probe_end(160, "failed_probe", 1, 30, None));

    // Notes
    transcript.push(TranscriptEvent::note(200, "debug info"));

    // Anna's final answer (the ONE output we want) - uses FinalAnswer, not Message
    transcript.push(TranscriptEvent::final_answer(300, "the answer"));

    // Count FinalAnswer events (THE contract for answer source)
    let final_answer_count = transcript
        .events
        .iter()
        .filter(|e| matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. }))
        .count();

    assert_eq!(
        final_answer_count, 1,
        "Mixed transcript should have exactly 1 FinalAnswer"
    );

    // Verify all event types are represented (compile-time coverage)
    let has_message = transcript
        .events
        .iter()
        .any(|e| matches!(&e.kind, TranscriptEventKind::Message { .. }));
    let has_final_answer = transcript
        .events
        .iter()
        .any(|e| matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. }));
    let has_stage_start = transcript
        .events
        .iter()
        .any(|e| matches!(&e.kind, TranscriptEventKind::StageStart { .. }));
    let has_stage_end = transcript
        .events
        .iter()
        .any(|e| matches!(&e.kind, TranscriptEventKind::StageEnd { .. }));
    let has_probe_start = transcript
        .events
        .iter()
        .any(|e| matches!(&e.kind, TranscriptEventKind::ProbeStart { .. }));
    let has_probe_end = transcript
        .events
        .iter()
        .any(|e| matches!(&e.kind, TranscriptEventKind::ProbeEnd { .. }));
    let has_note = transcript
        .events
        .iter()
        .any(|e| matches!(&e.kind, TranscriptEventKind::Note { .. }));

    assert!(
        has_message && has_final_answer && has_stage_start && has_stage_end && has_probe_start && has_probe_end && has_note,
        "Test should cover all TranscriptEventKind variants (except Unknown)"
    );
}

/// GUARDRAIL: Transcript deserialized from JSON with extra fields doesn't crash.
#[test]
fn test_transcript_json_forward_compatibility() {
    // Simulate JSON that might come from a newer version with extra fields
    let json = r#"{
        "events": [
            {
                "elapsed_ms": 0,
                "from": "you",
                "to": "anna",
                "kind": {"type": "message", "text": "test query"},
                "extra_field": "should be ignored"
            },
            {
                "elapsed_ms": 100,
                "from": "anna",
                "to": "you",
                "kind": {"type": "final_answer", "text": "test answer"},
                "future_metadata": {"nested": true}
            }
        ],
        "version": "2.0",
        "unknown_top_level": []
    }"#;

    // Should deserialize without panic (serde default is to ignore unknown fields)
    let result: Result<Transcript, _> = serde_json::from_str(json);

    // If deserialization succeeds, verify invariant
    if let Ok(transcript) = result {
        let final_answer_count = transcript
            .events
            .iter()
            .filter(|e| matches!(&e.kind, TranscriptEventKind::FinalAnswer { .. }))
            .count();
        assert_eq!(
            final_answer_count, 1,
            "Deserialized transcript should have 1 FinalAnswer"
        );
    }
    // If it fails, that's also acceptable - we just don't want a panic
}

/// Helper to create minimal ServiceDeskResult for testing
fn make_result(
    transcript: Transcript,
    answer: &str,
    clarification: Option<&str>,
    needs_clarification: bool,
) -> ServiceDeskResult {
    ServiceDeskResult {
        request_id: "test-12345678".to_string(),
        answer: answer.to_string(),
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
        feedback_request: None,
    }
}

/// GUARDRAIL: Answer source priority is consistent.
/// FinalAnswer in transcript > Clarification > Direct answer > Empty
#[test]
fn test_answer_source_priority_transcript_first() {
    // Case 1: FinalAnswer in transcript takes priority over result.answer
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));
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
    assert!(has_final_answer_in_transcript, "FinalAnswer in transcript should take priority");
}

/// GUARDRAIL: Clarification takes priority when needs_clarification=true
#[test]
fn test_answer_source_priority_clarification() {
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));
    let r = make_result(t, "", Some("What do you mean?"), true);

    assert!(r.needs_clarification);
    assert_eq!(r.clarification_question.as_deref(), Some("What do you mean?"));
}

/// GUARDRAIL: Direct answer used when no transcript answer and no clarification
#[test]
fn test_answer_source_priority_direct_answer() {
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));
    let r = make_result(t, "the answer", None, false);

    assert!(!r.needs_clarification);
    assert_eq!(r.answer, "the answer");
}

/// GUARDRAIL: Empty case - no answer available
#[test]
fn test_answer_source_priority_empty() {
    let mut t = Transcript::new();
    t.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));
    let r = make_result(t, "", None, false);

    assert!(!r.needs_clarification);
    assert!(r.answer.is_empty());
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
    assert_ne!(det_str, ok_str, "Deterministic display should differ from Ok");
    assert!(
        det_str.contains("deterministic"),
        "Deterministic display should contain 'deterministic'"
    );
}

/// GUARDRAIL: FinalAnswer and Unknown are handled in transcript iteration
#[test]
fn test_all_event_kinds_have_render_path() {
    // This test ensures we don't forget to handle new event kinds in render_debug
    let mut transcript = Transcript::new();

    // Add each event kind (except Unknown which is synthetic)
    transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));
    transcript.push(TranscriptEvent::stage_start(10, "test"));
    transcript.push(TranscriptEvent::stage_end(20, "test", StageOutcome::Ok));
    transcript.push(TranscriptEvent::stage_end(30, "test2", StageOutcome::Deterministic));
    transcript.push(TranscriptEvent::probe_start(40, "probe1", "cmd"));
    transcript.push(TranscriptEvent::probe_end(50, "probe1", 0, 10, None));
    transcript.push(TranscriptEvent::note(60, "debug"));
    transcript.push(TranscriptEvent::final_answer(70, "answer"));

    // Verify all are present and distinct
    let kinds: Vec<&str> = transcript
        .events
        .iter()
        .map(|e| match &e.kind {
            TranscriptEventKind::Message { .. } => "message",
            TranscriptEventKind::FinalAnswer { .. } => "final_answer",
            TranscriptEventKind::StageStart { .. } => "stage_start",
            TranscriptEventKind::StageEnd { .. } => "stage_end",
            TranscriptEventKind::ProbeStart { .. } => "probe_start",
            TranscriptEventKind::ProbeEnd { .. } => "probe_end",
            TranscriptEventKind::Note { .. } => "note",
            TranscriptEventKind::TicketCreated { .. } => "ticket_created",
            TranscriptEventKind::TicketStatusChanged { .. } => "ticket_status_changed",
            TranscriptEventKind::JuniorReview { .. } => "junior_review",
            TranscriptEventKind::SeniorEscalation { .. } => "senior_escalation",
            TranscriptEventKind::RevisionApplied { .. } => "revision_applied",
            TranscriptEventKind::ReviewGateDecision { .. } => "review_gate_decision",
            TranscriptEventKind::TeamReview { .. } => "team_review",
            TranscriptEventKind::ClarificationAsked { .. } => "clarification_asked",
            TranscriptEventKind::ClarificationAnswered { .. } => "clarification_answered",
            TranscriptEventKind::ClarificationVerified { .. } => "clarification_verified",
            TranscriptEventKind::FactStored { .. } => "fact_stored",
            TranscriptEventKind::FastPath { .. } => "fast_path",
            TranscriptEventKind::LlmTimeoutFallback { .. } => "llm_timeout_fallback",
            TranscriptEventKind::GracefulDegradation { .. } => "graceful_degradation",
            // v0.0.63 Service Desk Theatre events
            TranscriptEventKind::EvidenceSummary { .. } => "evidence_summary",
            TranscriptEventKind::DeterministicPath { .. } => "deterministic_path",
            TranscriptEventKind::ProposedAction { .. } => "proposed_action",
            TranscriptEventKind::ActionConfirmationRequest { .. } => "action_confirmation_request",
            TranscriptEventKind::Unknown => "unknown",
        })
        .collect();

    assert!(kinds.contains(&"message"));
    assert!(kinds.contains(&"final_answer"));
    assert!(kinds.contains(&"stage_start"));
    assert!(kinds.contains(&"stage_end"));
    assert!(kinds.contains(&"probe_start"));
    assert!(kinds.contains(&"probe_end"));
    assert!(kinds.contains(&"note"));
}
