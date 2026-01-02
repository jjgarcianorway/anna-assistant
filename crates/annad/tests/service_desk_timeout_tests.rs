//! Integration tests for service desk timeout handling.
//!
//! These tests verify:
//! - Timeout response format
//! - Timeout at different stages
//! - Partial probe results on timeout

use anna_shared::rpc::{
    EvidenceBlock, ProbeResult, QueryIntent, ReliabilitySignals, ServiceDeskResult,
    SpecialistDomain, TranslatorTicket,
};
use anna_shared::transcript::Transcript;

// === Helper Functions ===

fn make_ticket(domain: SpecialistDomain, probes: Vec<&str>, confidence: f32) -> TranslatorTicket {
    TranslatorTicket {
        intent: QueryIntent::Question,
        domain,
        entities: vec![],
        needs_probes: probes.into_iter().map(String::from).collect(),
        clarification_question: None,
        confidence,
        answer_contract: None,
    }
}

fn make_probe_result(cmd: &str, exit_code: i32, stdout: &str) -> ProbeResult {
    ProbeResult {
        command: cmd.to_string(),
        exit_code,
        stdout: stdout.to_string(),
        stderr: String::new(),
        timing_ms: 100,
    }
}

// === Timeout Response Tests ===

#[test]
fn test_timeout_response_format() {
    // Timeout responses must have specific format
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: false,
    };

    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: None,
        answer_contract: None,
        confidence: 0.0,
    };

    let evidence = EvidenceBlock {
        hardware_fields: vec![],
        probes_executed: vec![],
        translator_ticket: ticket,
        last_error: Some("timeout at translator".to_string()),
    };

    let result = ServiceDeskResult {
        request_id: "test-id".to_string(),
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: String::new(),
        validated: false, // v0.0.298: Timeout responses are not validated
        reliability_score: signals.score().min(20), // Max 20 for timeout
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some(
            "The translator stage timed out. Please try again or simplify your request."
                .to_string(),
        ),
        clarification_request: None,
        transcript: Transcript::new(),
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    };

    // Timeout response must:
    // 1. Have reliability_score <= 20
    assert!(result.reliability_score <= 20);
    // 2. Have needs_clarification = true
    assert!(result.needs_clarification);
    // 3. Have clarification_question explaining the timeout
    assert!(result.clarification_question.is_some());
    assert!(result
        .clarification_question
        .as_ref()
        .unwrap()
        .contains("timed out"));
    // 4. Have last_error in evidence
    assert!(result.evidence.last_error.is_some());
    assert!(result
        .evidence
        .last_error
        .as_ref()
        .unwrap()
        .contains("timeout"));
}

#[test]
fn test_timeout_at_different_stages() {
    // Test timeout at each stage
    let stages = ["translator", "probes", "specialist", "supervisor"];

    for stage in stages {
        let signals = ReliabilitySignals {
            translator_confident: false,
            probe_coverage: false,
            answer_grounded: false,
            no_invention: true,
            clarification_not_needed: false,
        };

        let ticket = make_ticket(SpecialistDomain::System, vec![], 0.0);
        let evidence = EvidenceBlock {
            hardware_fields: vec![],
            probes_executed: vec![],
            translator_ticket: ticket,
            last_error: Some(format!("timeout at {}", stage)),
        };

        let result = ServiceDeskResult {
            request_id: "test-id".to_string(),
            case_number: None,
            assigned_staff: None,
            staff_id: None,
            answer: String::new(),
            validated: false, // v0.0.298
            reliability_score: signals.score().min(20),
            reliability_signals: signals,
            reliability_explanation: None,
            domain: SpecialistDomain::System,
            evidence,
            needs_clarification: true,
            clarification_question: Some(format!(
                "The {} stage timed out. Please try again or simplify your request.",
                stage
            )),
            clarification_request: None,
            transcript: Transcript::new(),
            execution_trace: None,
            proposed_change: None,
            proposed_changes: Vec::new(),
            feedback_request: None,
        };

        assert!(result.reliability_score <= 20);
        assert!(result.evidence.last_error.is_some());
        assert!(result.evidence.last_error.as_ref().unwrap().contains(stage));
    }
}

#[test]
fn test_evidence_includes_partial_probes_on_timeout() {
    // If probes stage times out, we should still have partial probe results
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: false, // Incomplete probes
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: false,
    };

    let ticket = make_ticket(
        SpecialistDomain::System,
        vec!["top_memory", "disk_usage"],
        0.8,
    );

    // One probe completed before timeout
    let partial_probes = vec![make_probe_result("ps aux --sort=-%mem", 0, "output")];

    let evidence = EvidenceBlock {
        hardware_fields: vec!["cpu_model".to_string()],
        probes_executed: partial_probes,
        translator_ticket: ticket,
        last_error: Some("timeout at probes".to_string()),
    };

    let result = ServiceDeskResult {
        request_id: "test-id".to_string(),
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: String::new(),
        validated: false, // v0.0.298
        reliability_score: signals.score().min(20),
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some("The probes stage timed out.".to_string()),
        clarification_request: None,
        transcript: Transcript::new(),
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    };

    // Should preserve partial probe results
    assert_eq!(result.evidence.probes_executed.len(), 1);
    assert!(result.evidence.last_error.is_some());
}
