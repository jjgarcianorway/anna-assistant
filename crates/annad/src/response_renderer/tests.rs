//! Tests for response renderer.

use anna_shared::specialist_contract::{
    Answer, Evidence, Mood, ResponseStatus, Severity, SpecialistResponse, StaffView,
};

use super::*;

#[test]
fn test_render_ok_response() {
    let response = SpecialistResponse {
        ticket_id: "DSK-0101".to_string(),
        status: ResponseStatus::Ok,
        answer: Answer {
            short: "No, there is no active swap configured.".to_string(),
            detail: Some("Both free -h and /proc/swaps show 0B swap.".to_string()),
            domain_summary: None,
        },
        evidence: vec![Evidence {
            probe: "swap_files".to_string(),
            snippet: "Filename Type Size Used Priority".to_string(),
            interpretation: "No entries listed.".to_string(),
        }],
        confidence: 0.95,
        staff_view: Some(StaffView {
            assignee_role: "System Specialist".to_string(),
            severity: Severity::Info,
            mood: Mood::Confident,
            short_note: Some("No swap configured.".to_string()),
            complexity: 1,
        }),
        next_steps: None,
        discovery: None,
        missing_probes: vec![],
        can_answer: true,
        evidence_references: vec![],
        knowledge_used: vec![],
        citations: vec![],
    };

    let rendered = render_response(&response, "system");

    assert!(rendered.answer.contains("No, there is no active swap"));
    assert_eq!(rendered.reliability, 95);
    // v0.0.406: System domain now maps to Kari (Performance team)
    assert!(rendered.internal_comms.contains("Kari"));
    assert!(rendered.internal_comms.contains("No swap configured"));
}

#[test]
fn test_render_needs_more_data() {
    let response = SpecialistResponse {
        ticket_id: "DSK-0102".to_string(),
        status: ResponseStatus::NeedsMoreData,
        answer: Answer {
            short: "I cannot determine if zram is enabled.".to_string(),
            detail: None,
            domain_summary: None,
        },
        evidence: vec![],
        confidence: 0.3,
        staff_view: Some(StaffView {
            assignee_role: "System Specialist".to_string(),
            severity: Severity::Unknown,
            mood: Mood::Blocked,
            short_note: Some("Need zram probes.".to_string()),
            complexity: 2,
        }),
        next_steps: None,
        discovery: None,
        missing_probes: vec!["zram_devices".to_string()],
        can_answer: false,
        evidence_references: vec![],
        knowledge_used: vec![],
        citations: vec![],
    };

    let rendered = render_response(&response, "system");

    assert!(rendered.answer.contains("cannot determine"));
    assert!(rendered.internal_comms.contains("Need more data"));
}

#[test]
fn test_format_for_display() {
    let rendered = RenderedResponse {
        answer: "Test answer.".to_string(),
        evidence_lines: vec!["- probe1: snippet1".to_string()],
        // v0.0.406: Using Kari from Performance team
        internal_comms: "Kari (Performance Analyst): Looks good.".to_string(),
        reliability: 90,
        status_message: "System Status | Verified | 90%".to_string(),
        citation_lines: vec![],
    };

    let output = format_for_display(&rendered, "DSK-0101");

    assert!(output.contains("--- internal comms ---"));
    assert!(output.contains("[anna]"));
    assert!(output.contains("Test answer"));
    assert!(output.contains("ticket: DSK-0101"));
}

#[test]
fn test_format_with_citations() {
    let rendered = RenderedResponse {
        answer: "systemctl restarts services.".to_string(),
        evidence_lines: vec![],
        internal_comms: "Hugo: All clear.".to_string(),
        reliability: 85,
        status_message: "System Status | 85%".to_string(),
        citation_lines: vec![
            "- systemctl(1) (man page): \"systemctl restart SERVICE...\"".to_string(),
        ],
    };

    let output = format_for_display(&rendered, "DSK-0201");

    assert!(output.contains("Sources:"));
    assert!(output.contains("systemctl(1)"));
}
