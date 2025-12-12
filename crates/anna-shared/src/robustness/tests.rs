//! Robustness acceptance tests (v0.0.433).

use super::contract::{EvidenceRef, ProposedStep, SpecialistResult, TicketMetrics, TicketOutcome};
use super::fallback::{FallbackGenerator, ProbeEvidence};
use super::json_enforce::{JsonEnforcer, ParseResult, SchemaHint};
use super::lifecycle::{TicketLifecycle, TicketState};
use super::outcome_messages::OutcomeRenderer;
use super::performance::{PerformanceTracker, ProbeStatus, StreamingUpdate};
use super::stats_engine::{StatsEngine, TicketStats, TruthfulStats};
use super::timeouts::{TimeoutConfig, TimeoutEnforcer, TimeoutStage};

/// Scenario 1: RAM query with successful LLM.
/// Probes collect memory info, LLM works, outcome is Success.
#[test]
fn test_ram_query_success() {
    // Setup
    let mut lifecycle = TicketLifecycle::new("DSK-001", "how much free ram do I have right now?");
    let mut stats_engine = StatsEngine::new();
    let mut tracker = PerformanceTracker::new();

    // Start processing
    lifecycle.start_processing().unwrap();
    assert_eq!(lifecycle.state, TicketState::InProgress);

    // Simulate probe run
    tracker.start_stage(TimeoutStage::Probes);
    tracker.record_probe("proc_meminfo", ProbeStatus::Ok, 30);
    tracker.end_current_stage();

    // Simulate LLM success
    tracker.start_stage(TimeoutStage::JuniorLlm);
    let result =
        SpecialistResult::success("You have 17.0 GiB free out of 31.0 GiB total (54% available).")
            .with_handler("Sofia", "Desktop")
            .with_confidence(0.95)
            .with_evidence(EvidenceRef::new("proc_meminfo", "mem_001"));
    tracker.end_current_stage();

    // Apply outcome
    lifecycle.apply_outcome(result.outcome).unwrap();
    assert_eq!(lifecycle.state, TicketState::ResolvedSuccess);
    assert!(lifecycle.state.is_success());

    // Record stats
    let ticket_stats = TicketStats {
        ticket_id: "DSK-001".to_string(),
        outcome: result.outcome,
        processing_time_ms: tracker.elapsed_ms(),
        timestamp_ms: 0,
        handler: Some("Sofia".to_string()),
        department: Some("Desktop".to_string()),
        probes_count: 1,
        retry_used: false,
    };
    stats_engine.record_ticket(ticket_stats);

    // Verify stats
    assert_eq!(stats_engine.stats().successes, 1);
    assert_eq!(stats_engine.stats().failures, 0);
    assert!(stats_engine.stats().total_xp > 0);
}

/// Scenario 1b: RAM query with LLM parse failure.
/// Probes succeed, LLM fails, fallback provides partial answer.
#[test]
fn test_ram_query_parse_failure_fallback() {
    let generator = FallbackGenerator::new();

    // Probe succeeded
    let evidence = vec![ProbeEvidence::new(
        "proc_meminfo",
        "MemTotal:       32896136 kB\nMemFree:         8234567 kB\nMemAvailable:   17825792 kB\n",
        true,
    )
    .with_duration(30)];

    // Generate fallback
    let result = generator.generate_result(&evidence, "LLM parse failed");

    // Should be Partial, not failure
    assert_eq!(result.outcome, TicketOutcome::Partial);
    assert!(result.human_summary.contains("GiB"));
    assert!(result.confidence > 0.0);
    assert!(!result.evidence_refs.is_empty());
}

/// Scenario 2: Boot time query with timeout.
/// Probes run, specialist times out, stats count as failure.
#[test]
fn test_boot_query_timeout() {
    let mut lifecycle = TicketLifecycle::new("DSK-002", "which service slowed down my boot?");
    let mut stats_engine = StatsEngine::new();

    lifecycle.start_processing().unwrap();

    // Simulate timeout
    let result = SpecialistResult::timeout("senior_llm_parse");

    lifecycle.apply_outcome(result.outcome).unwrap();
    assert_eq!(lifecycle.state, TicketState::ResolvedFailure);
    assert!(lifecycle.state.is_failure());

    // Record stats
    let ticket_stats = TicketStats::new("DSK-002", result.outcome);
    stats_engine.record_ticket(ticket_stats);

    // Verify failure is counted
    assert_eq!(stats_engine.stats().successes, 0);
    assert_eq!(stats_engine.stats().failures, 1);
    assert_eq!(stats_engine.stats().timeouts, 1);
    assert_eq!(stats_engine.stats().total_xp, 0); // No XP for failures
}

/// Scenario 3: Mixed session stats.
/// Some successes, some failures - stats must reflect reality.
#[test]
fn test_mixed_session_stats() {
    let mut stats_engine = StatsEngine::new();

    // 6 successes
    for i in 0..6 {
        stats_engine.record_ticket(TicketStats::new(
            &format!("T-{}", i),
            TicketOutcome::Success,
        ));
    }

    // 2 timeouts
    stats_engine.record_ticket(TicketStats::new("T-6", TicketOutcome::Timeout));
    stats_engine.record_ticket(TicketStats::new("T-7", TicketOutcome::Timeout));

    // 1 parse error
    stats_engine.record_ticket(TicketStats::new("T-8", TicketOutcome::ParseError));

    // 1 partial
    stats_engine.record_ticket(TicketStats::new("T-9", TicketOutcome::Partial));

    let stats = stats_engine.stats();

    // Verify non-100% success rate
    assert_eq!(stats.successes, 6);
    assert_eq!(stats.failures, 3);
    assert_eq!(stats.partial_resolutions, 1);
    assert!((stats.success_rate() - 0.666).abs() < 0.01); // ~66.6%

    // Verify summary mentions failures
    let summary = stats.format_summary();
    assert!(summary.contains("66%") || summary.contains("67%"));
    assert!(summary.contains("3 tickets failed"));
}

/// Scenario 4: LLM parse errors in debug mode.
/// Parse failures should show debug info when debug_mode is ON.
#[test]
fn test_parse_error_debug_info() {
    // With debug mode OFF
    let renderer_normal = OutcomeRenderer::new(false);
    let result = SpecialistResult::parse_error("Invalid JSON at position 42");

    let msg_normal = renderer_normal.render(&result);
    assert!(msg_normal.is_failure);
    assert!(msg_normal.context.is_none()); // No debug info

    // With debug mode ON
    let renderer_debug = OutcomeRenderer::new(true);
    let msg_debug = renderer_debug.render(&result);

    assert!(msg_debug.context.is_some());
    assert!(msg_debug.context.unwrap().contains("position 42"));
}

/// Scenario 5: JSON parsing robustness.
#[test]
fn test_json_parsing_robustness() {
    let mut enforcer = JsonEnforcer::new();

    // Valid JSON
    let valid = r#"{"outcome": "Success", "human_summary": "All good", "confidence": 0.9, "steps": [], "evidence_refs": []}"#;
    let result = enforcer.parse(valid);
    assert!(result.is_ok());

    // JSON with prose before
    let with_prose = r#"Here is my analysis:
    {"outcome": "Success", "human_summary": "Test", "confidence": 0.8, "steps": [], "evidence_refs": []}
    Hope this helps!"#;
    let result2 = enforcer.parse(with_prose);
    assert!(result2.is_ok());

    // Invalid JSON
    let invalid = "This is not JSON at all, just text.";
    let result3 = enforcer.parse(invalid);
    assert!(!result3.is_ok());
}

/// Scenario 6: Clarification flow does not count as resolved.
#[test]
fn test_clarification_not_resolved() {
    let mut stats = TruthfulStats::default();

    // Clarification required
    stats.record(
        &TicketStats::new("T-1", TicketOutcome::ClarificationRequired),
        0,
    );

    // Should not count as success or failure
    assert_eq!(stats.successes, 0);
    assert_eq!(stats.failures, 0);
    assert_eq!(stats.awaiting_clarification, 1);

    // After clarification, success
    stats.record(
        &TicketStats::new("T-1-followup", TicketOutcome::Success),
        10,
    );

    assert_eq!(stats.successes, 1);
    assert_eq!(stats.total_resolved(), 1);
}

/// Scenario 7: Timeout enforcer behavior.
#[test]
fn test_timeout_enforcer() {
    let mut enforcer = TimeoutEnforcer::new();

    // Start stages
    enforcer.start_stage(TimeoutStage::Translator);
    assert!(!enforcer.is_global_timeout());
    std::thread::sleep(std::time::Duration::from_millis(1));
    enforcer.end_current_stage();

    enforcer.start_stage(TimeoutStage::JuniorLlm);
    std::thread::sleep(std::time::Duration::from_millis(1));
    enforcer.end_current_stage();

    // Check timing summary - total_ms should reflect global elapsed time
    let summary = enforcer.timing_summary();
    assert!(summary.total_ms >= 2, "total_ms was {}", summary.total_ms);
}

/// Scenario 8: Retry behavior on parse error.
#[test]
fn test_retry_on_parse_error() {
    let mut enforcer = TimeoutEnforcer::new();

    // First parse fails
    assert!(enforcer.can_retry_parse());
    enforcer.record_retry();

    // After retry, no more retries allowed
    assert!(!enforcer.can_retry_parse());
    assert_eq!(enforcer.retry_count(), 1);
}

/// Scenario 9: Streaming updates format correctly.
#[test]
fn test_streaming_updates() {
    let mut tracker = PerformanceTracker::new();

    // Emit various updates
    tracker.record_probe("memory", ProbeStatus::Ok, 50);
    tracker.record_specialist_start("Sofia", "Desktop");
    tracker.record_specialist_finish("Sofia", 200);
    tracker.record_complete(true);

    let updates = tracker.format_updates();
    assert!(updates.iter().any(|u| u.contains("memory")));
    assert!(updates.iter().any(|u| u.contains("Sofia")));
    assert!(updates.iter().any(|u| u.contains("complete")));
}

/// Scenario 10: Fallback handles no evidence.
#[test]
fn test_fallback_no_evidence() {
    let generator = FallbackGenerator::new();

    // No evidence - fallback returns error
    let result = generator.generate_result(&[], "LLM failed");
    assert_eq!(result.outcome, TicketOutcome::InternalError);
}

/// Scenario 11: Lifecycle state transitions.
#[test]
fn test_lifecycle_transitions() {
    let mut ticket = TicketLifecycle::new("DSK-010", "test query");

    // Valid transitions
    assert_eq!(ticket.state, TicketState::Opened);
    ticket.start_processing().unwrap();
    assert_eq!(ticket.state, TicketState::InProgress);

    // Clarification flow
    ticket
        .apply_outcome(TicketOutcome::ClarificationRequired)
        .unwrap();
    assert_eq!(ticket.state, TicketState::AwaitingClarification);

    ticket.provide_clarification("more info").unwrap();
    assert_eq!(ticket.state, TicketState::InProgress);

    ticket.apply_outcome(TicketOutcome::Success).unwrap();
    assert!(ticket.state.is_terminal());

    // Cannot transition from terminal
    let result = ticket.apply_outcome(TicketOutcome::Timeout);
    assert!(result.is_err());
}

/// Scenario 12: Outcome renderer produces correct messages.
#[test]
fn test_outcome_messages() {
    let renderer = OutcomeRenderer::new(false);

    // Success message
    let success = SpecialistResult::success("Memory looks good");
    let msg = renderer.render(&success);
    assert!(msg.resolved);
    assert!(!msg.is_failure);

    // Timeout message
    let timeout = SpecialistResult::timeout("senior_llm");
    let msg = renderer.render(&timeout);
    assert!(!msg.resolved);
    assert!(msg.is_failure);
    assert!(msg.body.contains("ran out of time"));

    // Unsupported message
    let unsupported = SpecialistResult::unsupported("Cannot help with that");
    let msg = renderer.render(&unsupported);
    assert!(msg.is_failure);
}

/// Scenario 13: Schema hint for prompts.
#[test]
fn test_schema_hint() {
    let hint = SchemaHint::default();
    let prompt = hint.format_for_prompt();

    assert!(prompt.contains("outcome"));
    assert!(prompt.contains("human_summary"));
    assert!(prompt.contains("CRITICAL"));

    // Stricter version
    let strict = hint.stricter_version();
    let strict_prompt = strict.format_for_prompt();
    assert!(strict_prompt.contains("RETRY"));
}

/// Scenario 14: Performance breakdown.
#[test]
fn test_performance_breakdown() {
    let mut tracker = PerformanceTracker::new();

    tracker.start_stage(TimeoutStage::Probes);
    std::thread::sleep(std::time::Duration::from_millis(5));
    tracker.end_current_stage();

    tracker.start_stage(TimeoutStage::JuniorLlm);
    std::thread::sleep(std::time::Duration::from_millis(5));
    tracker.end_current_stage();

    tracker.record_complete(true);

    let breakdown = tracker.breakdown();
    assert!(breakdown.total_ms >= 10);
    assert!(!breakdown.had_timeouts());
}

/// Scenario 15: XP calculation respects failures.
#[test]
fn test_xp_calculation() {
    let mut engine = StatsEngine::new();

    // Success gets XP
    engine.record_ticket(TicketStats {
        ticket_id: "T1".to_string(),
        outcome: TicketOutcome::Success,
        processing_time_ms: 1000, // Fast
        probes_count: 3,          // Complexity bonus
        ..TicketStats::new("T1", TicketOutcome::Success)
    });
    let xp_after_success = engine.stats().total_xp;
    assert!(xp_after_success > 0);

    // Partial gets half XP
    engine.record_ticket(TicketStats {
        ticket_id: "T2".to_string(),
        outcome: TicketOutcome::Partial,
        processing_time_ms: 5000,
        probes_count: 1,
        ..TicketStats::new("T2", TicketOutcome::Partial)
    });

    // Failure gets no XP
    let xp_before_failure = engine.stats().total_xp;
    engine.record_ticket(TicketStats::new("T3", TicketOutcome::Timeout));
    assert_eq!(engine.stats().total_xp, xp_before_failure); // Unchanged
}
