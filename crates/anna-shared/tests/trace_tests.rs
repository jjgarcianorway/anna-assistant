//! Golden tests for execution trace (v0.0.23 TRACE phase).
//!
//! Tests the three canonical scenarios:
//! 1. Deterministic route answers directly (specialist skipped)
//! 2. Specialist LLM answers successfully
//! 3. Specialist timeout with deterministic fallback

use anna_shared::trace::{
    evidence_kinds_from_route, EvidenceKind, ExecutionTrace, FallbackUsed, ProbeStats,
    SpecialistOutcome,
};

// =============================================================================
// Scenario 1: Deterministic route answers directly
// Query: "how much memory" -> memory_usage route -> specialist skipped
// =============================================================================

#[test]
fn test_deterministic_route_memory_usage() {
    let probe_stats = ProbeStats {
        planned: 1,
        succeeded: 1,
        failed: 0,
        timed_out: 0,
    };
    let evidence = evidence_kinds_from_route("memory_usage");

    let trace = ExecutionTrace::deterministic_route("memory_usage", probe_stats.clone(), evidence);

    assert_eq!(trace.specialist_outcome, SpecialistOutcome::Skipped);
    assert_eq!(trace.fallback_used, FallbackUsed::None);
    assert!(trace.answer_is_deterministic);
    assert_eq!(trace.probe_stats.planned, 1);
    assert_eq!(trace.probe_stats.succeeded, 1);
    assert!(trace.evidence_kinds.contains(&EvidenceKind::Memory));

    // Display format
    let display = trace.to_string();
    assert!(display.contains("deterministic route"));
    assert!(display.contains("skipped"));
    assert!(display.contains("memory"));
}

#[test]
fn test_deterministic_route_system_health() {
    let probe_stats = ProbeStats {
        planned: 4,
        succeeded: 4,
        failed: 0,
        timed_out: 0,
    };
    let evidence = evidence_kinds_from_route("system_health_summary");

    let trace = ExecutionTrace::deterministic_route("system_health_summary", probe_stats, evidence.clone());

    assert_eq!(trace.specialist_outcome, SpecialistOutcome::Skipped);
    assert!(trace.answer_is_deterministic);
    assert_eq!(trace.evidence_kinds.len(), 4);
    assert!(evidence.contains(&EvidenceKind::Memory));
    assert!(evidence.contains(&EvidenceKind::Disk));
    assert!(evidence.contains(&EvidenceKind::Cpu));
    assert!(evidence.contains(&EvidenceKind::BlockDevices));
}

// =============================================================================
// Scenario 2: Specialist LLM answers successfully
// Query: complex question -> triage path -> specialist answers
// =============================================================================

#[test]
fn test_specialist_ok() {
    let probe_stats = ProbeStats {
        planned: 2,
        succeeded: 2,
        failed: 0,
        timed_out: 0,
    };

    let trace = ExecutionTrace::specialist_ok(probe_stats);

    assert_eq!(trace.specialist_outcome, SpecialistOutcome::Ok);
    assert_eq!(trace.fallback_used, FallbackUsed::None);
    assert!(!trace.answer_is_deterministic);
    assert!(trace.evidence_kinds.is_empty());

    // Display format
    let display = trace.to_string();
    assert!(display.contains("specialist"));
    assert!(display.contains("ok"));
    assert!(!display.contains("deterministic"));
}

#[test]
fn test_specialist_ok_with_failures() {
    let probe_stats = ProbeStats {
        planned: 3,
        succeeded: 2,
        failed: 1,
        timed_out: 0,
    };

    let trace = ExecutionTrace::specialist_ok(probe_stats.clone());

    assert_eq!(trace.specialist_outcome, SpecialistOutcome::Ok);
    assert_eq!(trace.probe_stats.succeeded, 2);
    assert_eq!(trace.probe_stats.failed, 1);

    // ProbeStats display
    let stats_display = probe_stats.to_string();
    assert!(stats_display.contains("2/3"));
    assert!(stats_display.contains("1 failed"));
}

// =============================================================================
// Scenario 3: Specialist timeout with deterministic fallback
// Query: known class -> specialist times out -> fallback to deterministic
// =============================================================================

#[test]
fn test_specialist_timeout_with_fallback() {
    let probe_stats = ProbeStats {
        planned: 1,
        succeeded: 1,
        failed: 0,
        timed_out: 0,
    };
    let evidence = evidence_kinds_from_route("disk_usage");

    let trace = ExecutionTrace::specialist_timeout_with_fallback("disk_usage", probe_stats, evidence);

    assert_eq!(trace.specialist_outcome, SpecialistOutcome::Timeout);
    assert!(matches!(
        trace.fallback_used,
        FallbackUsed::Deterministic { ref route_class } if route_class == "disk_usage"
    ));
    assert!(trace.answer_is_deterministic);
    assert!(trace.evidence_kinds.contains(&EvidenceKind::Disk));

    // Display format
    let display = trace.to_string();
    assert!(display.contains("deterministic fallback"));
    assert!(display.contains("disk_usage"));
    assert!(display.contains("timeout"));
    assert!(display.contains("disk"));
}

#[test]
fn test_specialist_error_with_fallback() {
    let probe_stats = ProbeStats {
        planned: 1,
        succeeded: 1,
        failed: 0,
        timed_out: 0,
    };
    let evidence = evidence_kinds_from_route("service_status");

    let trace = ExecutionTrace::specialist_error_with_fallback("service_status", probe_stats, evidence);

    assert_eq!(trace.specialist_outcome, SpecialistOutcome::Error);
    assert!(matches!(
        trace.fallback_used,
        FallbackUsed::Deterministic { ref route_class } if route_class == "service_status"
    ));
    assert!(trace.answer_is_deterministic);
    assert!(trace.evidence_kinds.contains(&EvidenceKind::Services));

    let display = trace.to_string();
    assert!(display.contains("error"));
    assert!(display.contains("services"));
}

#[test]
fn test_specialist_timeout_no_fallback() {
    let probe_stats = ProbeStats {
        planned: 2,
        succeeded: 1,
        failed: 0,
        timed_out: 1,
    };

    let trace = ExecutionTrace::specialist_timeout_no_fallback(probe_stats.clone());

    assert_eq!(trace.specialist_outcome, SpecialistOutcome::Timeout);
    assert_eq!(trace.fallback_used, FallbackUsed::None);
    assert!(!trace.answer_is_deterministic);
    assert!(trace.evidence_kinds.is_empty());

    // ProbeStats with timeout
    let stats_display = probe_stats.to_string();
    assert!(stats_display.contains("1/2"));
    assert!(stats_display.contains("1 timed out"));
}

// =============================================================================
// Helper function tests
// =============================================================================

#[test]
fn test_evidence_kinds_from_route() {
    assert_eq!(
        evidence_kinds_from_route("memory_usage"),
        vec![EvidenceKind::Memory]
    );
    assert_eq!(
        evidence_kinds_from_route("ram_info"),
        vec![EvidenceKind::Memory]
    );
    assert_eq!(
        evidence_kinds_from_route("disk_usage"),
        vec![EvidenceKind::Disk]
    );
    assert_eq!(
        evidence_kinds_from_route("cpu_info"),
        vec![EvidenceKind::Cpu]
    );
    assert_eq!(
        evidence_kinds_from_route("service_status"),
        vec![EvidenceKind::Services]
    );
    assert_eq!(
        evidence_kinds_from_route("system_health_summary"),
        vec![
            EvidenceKind::Memory,
            EvidenceKind::Disk,
            EvidenceKind::BlockDevices,
            EvidenceKind::Cpu,
        ]
    );
    // Unknown routes return empty
    assert!(evidence_kinds_from_route("unknown_route").is_empty());
}

#[test]
fn test_probe_stats_from_results() {
    use anna_shared::rpc::ProbeResult;

    let results = vec![
        ProbeResult {
            command: "free -h".to_string(),
            exit_code: 0,
            stdout: "Mem: 16G".to_string(),
            stderr: String::new(),
            timing_ms: 10,
        },
        ProbeResult {
            command: "df -h".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "error".to_string(),
            timing_ms: 5,
        },
        ProbeResult {
            command: "lscpu".to_string(),
            exit_code: 1,  // Non-zero exit code for timeout
            stdout: String::new(),
            stderr: "timeout".to_string(),
            timing_ms: 5000,
        },
    ];

    let stats = ProbeStats::from_results(3, &results);

    assert_eq!(stats.planned, 3);
    assert_eq!(stats.succeeded, 1); // Only first one (exit_code == 0)
    assert_eq!(stats.failed, 1); // Second one (exit_code != 0, stderr doesn't contain timeout)
    assert_eq!(stats.timed_out, 1); // Third one (exit_code != 0, stderr contains timeout)
}

// =============================================================================
// Serialization tests
// =============================================================================

#[test]
fn test_trace_serialization_roundtrip() {
    let trace = ExecutionTrace::specialist_timeout_with_fallback(
        "memory_usage",
        ProbeStats {
            planned: 1,
            succeeded: 1,
            failed: 0,
            timed_out: 0,
        },
        vec![EvidenceKind::Memory],
    );

    let json = serde_json::to_string(&trace).unwrap();
    let parsed: ExecutionTrace = serde_json::from_str(&json).unwrap();

    assert_eq!(trace, parsed);
}

#[test]
fn test_fallback_used_json_format() {
    let fallback = FallbackUsed::Deterministic {
        route_class: "disk_usage".to_string(),
    };
    let json = serde_json::to_string(&fallback).unwrap();

    // Should use tagged format
    assert!(json.contains("\"type\":\"deterministic\""));
    assert!(json.contains("\"route_class\":\"disk_usage\""));
}

#[test]
fn test_specialist_outcome_json_format() {
    let outcome = SpecialistOutcome::BudgetExceeded;
    let json = serde_json::to_string(&outcome).unwrap();

    // Should use snake_case
    assert_eq!(json, "\"budget_exceeded\"");
}
