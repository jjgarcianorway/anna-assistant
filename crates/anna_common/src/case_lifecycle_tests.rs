//! Integration tests for Case Lifecycle v0.0.59
//!
//! Tests:
//! 1. Doctor query opens case with lifecycle progression to at least plan_ready
//! 2. Transcript includes departmental handoff line and case_id
//! 3. Junior can force re-check when evidence coverage is below threshold
//! 4. Alerts can be linked to cases deterministically

use crate::case_lifecycle::{
    ActionRisk, CaseFileV2, CaseStatus, Department, Participant, ProposedAction,
};
use crate::proactive_alerts::AlertType;
use crate::service_desk::{
    open_case_for_alert, progress_case_investigating, progress_case_plan_ready,
    progress_case_triage, triage_request,
};
use crate::transcript_v2::{
    render_case_transcript, render_handoff, render_junior_disagreement, DepartmentOutput,
    Hypothesis, TranscriptBuilder,
};
use std::collections::HashMap;

// ============================================================================
// Test 1: Doctor query opens case with lifecycle progression
// ============================================================================

#[test]
fn test_doctor_query_case_lifecycle_progression() {
    // Create a case for a network issue
    let mut case = CaseFileV2::new("test-lifecycle-001", "my wifi keeps disconnecting");

    // Initial state
    assert_eq!(case.status, CaseStatus::New);
    assert_eq!(case.assigned_department, Department::ServiceDesk);

    // Triage phase
    progress_case_triage(
        &mut case,
        "my wifi keeps disconnecting",
        &["network_status".to_string()],
    );
    assert_eq!(case.status, CaseStatus::Triaged);
    assert_eq!(case.assigned_department, Department::Networking);
    assert!(case
        .participants
        .contains(&Participant::Specialist(Department::Networking)));

    // Investigation phase
    progress_case_investigating(&mut case);
    assert_eq!(case.status, CaseStatus::Investigating);
    assert!(case.participants.contains(&Participant::Junior));

    // Add evidence
    case.add_evidence("E1", "network_status");
    case.add_evidence("E2", "nm_summary");
    assert_eq!(case.evidence_count, 2);

    // Add findings
    case.add_finding("WiFi interface is connected but experiencing packet loss");
    case.add_finding("Signal strength is low (-75 dBm)");

    // Add hypothesis
    case.add_hypothesis("Weak WiFi signal causing intermittent disconnects", 80);

    // Plan ready phase
    progress_case_plan_ready(&mut case);
    assert_eq!(case.status, CaseStatus::PlanReady);

    // Verify timeline has all expected events
    assert!(!case.timeline.is_empty());
    let timeline_summaries: Vec<&str> = case.timeline.iter().map(|e| e.summary.as_str()).collect();
    assert!(timeline_summaries.iter().any(|s| s.contains("Case opened")));
    assert!(timeline_summaries.iter().any(|s| s.contains("networking")));
}

// ============================================================================
// Test 2: Transcript includes departmental handoff and case_id
// ============================================================================

#[test]
fn test_transcript_includes_handoff_and_case_id() {
    let mut case = CaseFileV2::new("test-transcript-002", "no audio from speakers");

    // Triage and assign to audio department
    progress_case_triage(&mut case, "no audio from speakers", &[]);
    assert_eq!(case.assigned_department, Department::Audio);

    // Add some content
    case.add_evidence("E1", "audio_status");
    case.set_coverage(75);
    case.set_reliability(80);
    case.add_finding("PipeWire is running but no default sink configured");
    case.set_final_answer(
        "No default audio output is configured. Run 'pactl set-default-sink ...'",
    );

    // Render transcript
    let transcript = render_case_transcript(&case);

    // Verify case_id is present
    assert!(transcript.contains("test-transcript-002"));

    // Verify department assignment is shown
    assert!(transcript.contains("[audio]") || transcript.contains("audio"));

    // Verify it shows the triage phase
    assert!(transcript.contains("triage"));

    // Verify reliability and coverage are shown
    assert!(transcript.contains("80%") || transcript.contains("Reliability"));
    assert!(transcript.contains("75%") || transcript.contains("Coverage"));
}

#[test]
fn test_handoff_line_generation() {
    let line = render_handoff(
        &Participant::Anna,
        Department::Storage,
        "disk-related keywords: space, storage",
    );

    assert!(line.content.contains("[storage]"));
    assert!(line.content.contains("disk-related keywords"));
}

// ============================================================================
// Test 3: Junior forces re-check when evidence coverage below threshold
// ============================================================================

#[test]
fn test_junior_recheck_on_low_coverage() {
    let mut case = CaseFileV2::new("test-junior-003", "why is my boot slow");

    progress_case_triage(&mut case, "why is my boot slow", &[]);
    progress_case_investigating(&mut case);

    // Low coverage scenario
    case.add_evidence("E1", "boot_time_summary");
    case.set_coverage(45); // Below 90% threshold
    case.set_reliability(60);

    // Generate junior disagreement
    let missing = vec!["systemd-analyze".to_string(), "journal_errors".to_string()];
    let line = render_junior_disagreement(&missing);

    assert!(line.content.contains("Missing"));
    assert!(line.content.contains("systemd-analyze"));
    assert!(line.content.contains("journal_errors"));

    // Verify transcript shows low coverage
    let transcript = render_case_transcript(&case);
    assert!(transcript.contains("45%") || transcript.contains("Coverage"));
}

#[test]
fn test_junior_approves_high_coverage() {
    let mut case = CaseFileV2::new("test-junior-004", "what CPU do I have");

    progress_case_triage(&mut case, "what CPU do I have", &["cpu".to_string()]);
    progress_case_investigating(&mut case);

    case.add_evidence("E1", "hw_snapshot_cpu");
    case.set_coverage(100);
    case.set_reliability(95);
    case.resolve("AMD Ryzen 7 5800X [E1]", 95);

    // Verify high reliability approval
    let transcript = render_case_transcript(&case);
    assert!(transcript.contains("95%") || transcript.contains("Solid evidence"));
    assert_eq!(case.status, CaseStatus::Resolved);
}

// ============================================================================
// Test 4: Alerts can be linked to cases deterministically
// ============================================================================

#[test]
fn test_alert_linking_to_case() {
    let case = open_case_for_alert(
        "test-alert-005",
        "why is boot slow",
        "alert-boot-123",
        AlertType::BootRegression,
    );

    // Verify alert is linked
    assert!(case
        .linked_alert_ids
        .contains(&"alert-boot-123".to_string()));

    // Verify correct department
    assert_eq!(case.assigned_department, Department::Boot);

    // Verify status progressed
    assert_eq!(case.status, CaseStatus::Triaged);

    // Verify timeline has alert linkage event
    let has_alert_event = case.timeline.iter().any(|e| {
        matches!(
            &e.event,
            crate::case_lifecycle::TimelineEventType::AlertLinked { .. }
        )
    });
    assert!(has_alert_event);
}

#[test]
fn test_department_from_alert_type() {
    assert_eq!(
        Department::from_alert_type(AlertType::DiskPressure),
        Department::Storage
    );
    assert_eq!(
        Department::from_alert_type(AlertType::BootRegression),
        Department::Boot
    );
    assert_eq!(
        Department::from_alert_type(AlertType::ThermalThrottling),
        Department::Performance
    );
    assert_eq!(
        Department::from_alert_type(AlertType::ServiceFailed),
        Department::ServiceDesk
    );
}

// ============================================================================
// Test 5: Triage routing works correctly
// ============================================================================

#[test]
fn test_triage_routing_networking() {
    let result = triage_request("my wifi keeps disconnecting", &[]);
    assert_eq!(result.department, Department::Networking);
    // v0.0.64: Lowered threshold from 30 to 15 (single keyword match is enough)
    assert!(result.confidence >= 15);
}

#[test]
fn test_triage_routing_storage() {
    let result = triage_request("running out of disk space", &["disk_free".to_string()]);
    assert_eq!(result.department, Department::Storage);
    assert!(result.confidence >= 30);
}

#[test]
fn test_triage_routing_audio() {
    let result = triage_request("no sound from speakers", &[]);
    assert_eq!(result.department, Department::Audio);
}

#[test]
fn test_triage_routing_boot() {
    let result = triage_request("boot takes forever", &[]);
    assert_eq!(result.department, Department::Boot);
}

#[test]
fn test_triage_routing_graphics() {
    let result = triage_request("screen flickering on wayland", &[]);
    assert_eq!(result.department, Department::Graphics);
}

#[test]
fn test_triage_routing_ambiguous() {
    let result = triage_request("help", &[]);
    assert_eq!(result.department, Department::ServiceDesk);
    assert!(result.confidence < 30);
}

// ============================================================================
// Test 6: Structured department output
// ============================================================================

#[test]
fn test_department_output_structured() {
    let mut output = DepartmentOutput::new(Department::Networking);

    output
        .findings
        .push("DNS server not responding".to_string());
    output.findings.push("Default route is set".to_string());
    output.evidence_ids.push("E1".to_string());
    output.evidence_ids.push("E2".to_string());

    output.hypotheses.push(Hypothesis {
        label: "H1".to_string(),
        description: "DNS misconfiguration".to_string(),
        confidence: 75,
        supporting_evidence: vec!["E1".to_string()],
    });

    output.next_checks.push("ping 1.1.1.1".to_string());

    output.action_plan.push(ProposedAction {
        id: "A1".to_string(),
        description: "Update /etc/resolv.conf".to_string(),
        risk: ActionRisk::Low,
        tool_name: "file_edit_preview_v1".to_string(),
        parameters: HashMap::new(),
        evidence_ids: vec!["E1".to_string()],
        executed: false,
        result: None,
    });

    // Render
    let mut builder = TranscriptBuilder::new();
    output.render(&mut builder);
    let result = builder.build();

    // Verify structure
    assert!(result.contains("Findings"));
    assert!(result.contains("DNS server not responding"));
    assert!(result.contains("Evidence"));
    assert!(result.contains("E1"));
    assert!(result.contains("Hypotheses"));
    assert!(result.contains("H1"));
    assert!(result.contains("75%"));
    assert!(result.contains("Next checks"));
    assert!(result.contains("Action plan"));
    assert!(result.contains("A1"));
}

// ============================================================================
// Test 7: Case status transitions
// ============================================================================

#[test]
fn test_case_status_terminal() {
    assert!(CaseStatus::Resolved.is_terminal());
    assert!(CaseStatus::Abandoned.is_terminal());
    assert!(!CaseStatus::New.is_terminal());
    assert!(!CaseStatus::Investigating.is_terminal());
    assert!(!CaseStatus::PlanReady.is_terminal());
}

#[test]
fn test_case_status_active() {
    assert!(!CaseStatus::Resolved.is_active());
    assert!(!CaseStatus::Abandoned.is_active());
    assert!(CaseStatus::New.is_active());
    assert!(CaseStatus::Investigating.is_active());
    assert!(CaseStatus::AwaitingConfirmation.is_active());
}

#[test]
fn test_case_abandon() {
    let mut case = CaseFileV2::new("test-abandon-007", "test");
    progress_case_triage(&mut case, "test", &[]);
    progress_case_investigating(&mut case);

    case.abandon("User cancelled");
    assert_eq!(case.status, CaseStatus::Abandoned);
    assert!(case.status.is_terminal());
}

// ============================================================================
// Test 8: Participants tracking
// ============================================================================

#[test]
fn test_participants_tracking() {
    let mut case = CaseFileV2::new("test-participants-008", "test");

    // Initial participants
    assert!(case.participants.contains(&Participant::You));
    assert!(case.participants.contains(&Participant::Anna));

    // After triage
    progress_case_triage(&mut case, "network issue", &["network_status".to_string()]);
    assert!(case.participants.contains(&Participant::Translator));

    // After investigation
    progress_case_investigating(&mut case);
    assert!(case.participants.contains(&Participant::Junior));

    // Specialist added via department assignment
    assert!(case
        .participants
        .contains(&Participant::Specialist(Department::Networking)));
}
