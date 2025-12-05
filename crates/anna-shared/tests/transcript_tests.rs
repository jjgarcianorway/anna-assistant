//! Transcript event model tests.
//!
//! These tests verify wire format, FinalAnswer discriminator, forward compatibility,
//! and COST resource caps.

use anna_shared::resource_limits::MAX_TRANSCRIPT_EVENTS;
use anna_shared::transcript::{Actor, StageOutcome, Transcript, TranscriptEvent, TranscriptEventKind};

#[test]
fn test_actor_display() {
    assert_eq!(format!("{}", Actor::You), "you");
    assert_eq!(format!("{}", Actor::Anna), "anna");
    assert_eq!(format!("{}", Actor::System), "system");
}

#[test]
fn test_transcript_event_creation() {
    let event = TranscriptEvent::message(100, Actor::You, Actor::Anna, "test query");
    assert_eq!(event.elapsed_ms, 100);
    assert_eq!(event.from, Actor::You);
    assert_eq!(event.to, Some(Actor::Anna));
}

#[test]
fn test_is_debug_only() {
    let note = TranscriptEvent::note(0, "debug info");
    assert!(note.is_debug_only());

    let message = TranscriptEvent::message(0, Actor::Anna, Actor::You, "answer");
    assert!(!message.is_debug_only());
}

#[test]
fn test_transcript_push_and_len() {
    let mut transcript = Transcript::new();
    assert!(transcript.is_empty());

    transcript.push(TranscriptEvent::message(
        0,
        Actor::You,
        Actor::Anna,
        "hello",
    ));
    assert_eq!(transcript.len(), 1);

    transcript.push(TranscriptEvent::stage_start(100, "translator"));
    assert_eq!(transcript.len(), 2);
}

#[test]
fn test_transcript_serialization() {
    let mut transcript = Transcript::new();
    transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "test"));
    transcript.push(TranscriptEvent::stage_end(
        100,
        "translator",
        StageOutcome::Ok,
    ));

    let json = serde_json::to_string(&transcript).unwrap();
    assert!(json.contains("\"type\":\"message\""));
    assert!(json.contains("\"type\":\"stage_end\""));

    let parsed: Transcript = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 2);
}

#[test]
fn test_probe_events() {
    let start = TranscriptEvent::probe_start(50, "top_mem", "ps aux --sort=-%mem");
    let end = TranscriptEvent::probe_end(100, "top_mem", 0, 50, Some("first line".to_string()));

    if let TranscriptEventKind::ProbeStart { probe_id, command } = &start.kind {
        assert_eq!(probe_id, "top_mem");
        assert_eq!(command, "ps aux --sort=-%mem");
    } else {
        panic!("Expected ProbeStart");
    }

    if let TranscriptEventKind::ProbeEnd {
        exit_code,
        timing_ms,
        ..
    } = &end.kind
    {
        assert_eq!(*exit_code, 0);
        assert_eq!(*timing_ms, 50);
    } else {
        panic!("Expected ProbeEnd");
    }
}

#[test]
fn test_stage_outcome_display() {
    assert_eq!(format!("{}", StageOutcome::Ok), "ok");
    assert_eq!(format!("{}", StageOutcome::Timeout), "timeout");
    assert_eq!(format!("{}", StageOutcome::Error), "error");
    assert_eq!(format!("{}", StageOutcome::Skipped), "skipped");
    assert_eq!(format!("{}", StageOutcome::Deterministic), "deterministic");
}

/// GUARDRAIL: FinalAnswer is the authoritative answer discriminator
#[test]
fn test_final_answer_creation() {
    let event = TranscriptEvent::final_answer(500, "This is the answer");
    assert_eq!(event.from, Actor::Anna);
    assert_eq!(event.to, Some(Actor::You));
    assert!(!event.is_debug_only());

    if let TranscriptEventKind::FinalAnswer { text } = &event.kind {
        assert_eq!(text, "This is the answer");
    } else {
        panic!("Expected FinalAnswer kind");
    }
}

/// GUARDRAIL: FinalAnswer serializes with type=final_answer
#[test]
fn test_final_answer_serialization() {
    let event = TranscriptEvent::final_answer(100, "test answer");
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("\"type\":\"final_answer\""));
    assert!(json.contains("\"text\":\"test answer\""));

    // Round-trip
    let parsed: TranscriptEvent = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed.kind, TranscriptEventKind::FinalAnswer { .. }));
}

/// GUARDRAIL: Unknown event kinds deserialize safely (wire compatibility)
#[test]
fn test_unknown_event_kind_forward_compat() {
    // Simulate a future event type that old clients don't know about
    let json = r#"{
        "elapsed_ms": 100,
        "from": "system",
        "kind": {"type": "future_event_type", "data": "whatever"}
    }"#;

    // Should NOT panic - deserializes to Unknown
    let result: Result<TranscriptEvent, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Unknown event type should deserialize");

    let event = result.unwrap();
    assert!(matches!(event.kind, TranscriptEventKind::Unknown));
}

/// GUARDRAIL: Mixed transcript with FinalAnswer and Message from Anna
#[test]
fn test_final_answer_distinct_from_message() {
    let mut transcript = Transcript::new();

    // User query
    transcript.push(TranscriptEvent::message(0, Actor::You, Actor::Anna, "query"));
    // Some internal Anna message (NOT the final answer)
    transcript.push(TranscriptEvent::message(50, Actor::Anna, Actor::System, "thinking..."));
    // The actual final answer
    transcript.push(TranscriptEvent::final_answer(100, "the answer"));
    // Another Anna message after (should be ignored)
    transcript.push(TranscriptEvent::message(150, Actor::Anna, Actor::You, "follow-up"));

    // Count FinalAnswer events
    let final_answers: Vec<_> = transcript
        .events
        .iter()
        .filter(|e| matches!(e.kind, TranscriptEventKind::FinalAnswer { .. }))
        .collect();

    assert_eq!(final_answers.len(), 1, "Should have exactly one FinalAnswer");

    // Count Anna messages (not FinalAnswer)
    let anna_messages: Vec<_> = transcript
        .events
        .iter()
        .filter(|e| matches!(e.kind, TranscriptEventKind::Message { .. }) && e.from == Actor::Anna)
        .collect();

    assert_eq!(
        anna_messages.len(),
        2,
        "Should have two Anna messages (not FinalAnswer)"
    );
}

// === COST: Transcript cap golden tests ===

/// GOLDEN: Transcript within cap accepts all events
#[test]
fn golden_transcript_under_cap_no_drop() {
    let mut transcript = Transcript::new();

    // Add events up to but not exceeding cap
    for i in 0..MAX_TRANSCRIPT_EVENTS {
        let added = transcript.push(TranscriptEvent::note(i as u64, "event"));
        assert!(added, "Event {} should be added", i);
    }

    assert_eq!(transcript.len(), MAX_TRANSCRIPT_EVENTS);
    assert!(!transcript.was_capped());
    assert_eq!(transcript.dropped_count(), 0);
    assert!(transcript.diagnostic().is_none());
}

/// GOLDEN: Transcript at cap drops new events with diagnostic
#[test]
fn golden_transcript_at_cap_drops_new() {
    let mut transcript = Transcript::new();

    // Fill to cap
    for i in 0..MAX_TRANSCRIPT_EVENTS {
        transcript.push(TranscriptEvent::note(i as u64, "event"));
    }

    // Try to add more - should be dropped
    let added = transcript.push(TranscriptEvent::note(9999, "overflow"));
    assert!(!added, "Event beyond cap should be rejected");

    assert_eq!(transcript.len(), MAX_TRANSCRIPT_EVENTS);
    assert!(transcript.was_capped());
    assert_eq!(transcript.dropped_count(), 1);

    // Diagnostic should be present
    let diag = transcript.diagnostic();
    assert!(diag.is_some());
    let diag = diag.unwrap();
    assert_eq!(diag.dropped, 1);
    assert!(diag.format().contains("transcript"));
    assert!(diag.format().contains("reliability penalty"));
}

/// GOLDEN: Multiple drops are tracked accurately
#[test]
fn golden_transcript_multiple_drops_tracked() {
    let mut transcript = Transcript::new();

    // Fill to cap
    for i in 0..MAX_TRANSCRIPT_EVENTS {
        transcript.push(TranscriptEvent::note(i as u64, "event"));
    }

    // Try to add 5 more
    for _ in 0..5 {
        transcript.push(TranscriptEvent::note(9999, "overflow"));
    }

    assert_eq!(transcript.dropped_count(), 5);

    let diag = transcript.diagnostic().unwrap();
    assert_eq!(diag.dropped, 5);
}

/// GOLDEN: Dropped count not serialized (wire compatibility)
#[test]
fn golden_transcript_dropped_not_serialized() {
    let mut transcript = Transcript::new();

    // Fill and overflow
    for i in 0..(MAX_TRANSCRIPT_EVENTS + 3) {
        transcript.push(TranscriptEvent::note(i as u64, "event"));
    }

    assert_eq!(transcript.dropped_count(), 3);

    // Serialize and deserialize
    let json = serde_json::to_string(&transcript).unwrap();

    // JSON should NOT contain dropped_events
    assert!(!json.contains("dropped"));

    // Deserialized transcript should have events but no drop tracking
    let parsed: Transcript = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), MAX_TRANSCRIPT_EVENTS);
    assert_eq!(parsed.dropped_count(), 0); // Not tracked across wire
    assert!(!parsed.was_capped()); // Reset on deserialize
}
