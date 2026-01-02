//! Integration tests for service desk response format.
//!
//! These tests verify:
//! - Response format consistency
//! - Evidence block structure
//! - Domain and intent display
//! - ProbeResult structure

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

fn make_evidence(ticket: TranslatorTicket, probes: Vec<ProbeResult>) -> EvidenceBlock {
    EvidenceBlock {
        hardware_fields: vec!["cpu_model".to_string(), "ram_gb".to_string()],
        probes_executed: probes,
        translator_ticket: ticket,
        last_error: None,
    }
}

fn make_signals(confident: bool, coverage: bool, grounded: bool) -> ReliabilitySignals {
    ReliabilitySignals {
        translator_confident: confident,
        probe_coverage: coverage,
        answer_grounded: grounded,
        no_invention: true,
        clarification_not_needed: true,
    }
}

// === ServiceDeskResult Format Tests ===

#[test]
fn test_service_desk_result_structure() {
    let ticket = make_ticket(SpecialistDomain::System, vec!["top_memory"], 0.85);
    let probes = vec![make_probe_result("ps aux --sort=-%mem", 0, "output")];
    let evidence = make_evidence(ticket, probes);
    let signals = make_signals(true, true, true);

    let result = ServiceDeskResult {
        request_id: "test-id".to_string(),
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: "Test answer".to_string(),
        validated: true, // v0.0.298
        reliability_score: signals.score(),
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript: Transcript::new(),
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    };

    assert!(!result.answer.is_empty());
    assert!(result.reliability_score <= 100);
    assert!(!result.evidence.probes_executed.is_empty());
    assert!(!result.needs_clarification);
}

#[test]
fn test_clarification_response_format() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: Some("Could you provide more details?".to_string()),
        confidence: 0.2,
        answer_contract: None,
    };
    let evidence = make_evidence(ticket, vec![]);
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: false,
    };

    let result = ServiceDeskResult {
        request_id: "test-id".to_string(),
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: String::new(),
        validated: false, // v0.0.298: Clarification responses are not validated
        reliability_score: signals.score(),
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some("Could you provide more details?".to_string()),
        clarification_request: None,
        transcript: Transcript::new(),
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    };

    assert!(result.needs_clarification);
    assert!(result.clarification_question.is_some());
    assert!(result.answer.is_empty());
    assert_eq!(result.reliability_score, 20); // Only no_invention is true
}

// === Evidence Block Tests ===

#[test]
fn test_evidence_block_contains_probes() {
    let ticket = make_ticket(SpecialistDomain::System, vec!["top_memory"], 0.9);
    let probes = vec![
        make_probe_result("ps aux --sort=-%mem", 0, "USER PID %MEM"),
        make_probe_result("free -h", 0, "total 16G"),
    ];
    let evidence = make_evidence(ticket, probes);

    assert_eq!(evidence.probes_executed.len(), 2);
    assert!(evidence.probes_executed.iter().all(|p| p.exit_code == 0));
}

#[test]
fn test_evidence_block_includes_ticket() {
    let ticket = make_ticket(SpecialistDomain::Network, vec!["network_addrs"], 0.75);
    let evidence = make_evidence(ticket, vec![]);

    assert_eq!(evidence.translator_ticket.domain, SpecialistDomain::Network);
    assert!(evidence.translator_ticket.confidence >= 0.7);
}

#[test]
fn test_evidence_block_hardware_fields() {
    let ticket = make_ticket(SpecialistDomain::System, vec![], 0.8);
    let evidence = make_evidence(ticket, vec![]);

    assert!(evidence.hardware_fields.contains(&"cpu_model".to_string()));
    assert!(evidence.hardware_fields.contains(&"ram_gb".to_string()));
}

// === Domain Display Tests ===

#[test]
fn test_domain_display() {
    assert_eq!(format!("{}", SpecialistDomain::System), "system");
    assert_eq!(format!("{}", SpecialistDomain::Network), "network");
    assert_eq!(format!("{}", SpecialistDomain::Storage), "storage");
    assert_eq!(format!("{}", SpecialistDomain::Security), "security");
    assert_eq!(format!("{}", SpecialistDomain::Packages), "packages");
}

#[test]
fn test_intent_display() {
    assert_eq!(format!("{}", QueryIntent::Question), "question");
    assert_eq!(format!("{}", QueryIntent::Request), "request");
    assert_eq!(format!("{}", QueryIntent::Investigate), "investigate");
}

// === ProbeResult Tests ===

#[test]
fn test_probe_result_success() {
    let result = make_probe_result("df -h", 0, "Filesystem Size Used");
    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

#[test]
fn test_probe_result_failure() {
    let result = ProbeResult {
        command: "invalid_cmd".to_string(),
        exit_code: 127,
        stdout: String::new(),
        stderr: "command not found".to_string(),
        timing_ms: 50,
    };
    assert_ne!(result.exit_code, 0);
    assert!(!result.stderr.is_empty());
}

// === Golden Tests ===

#[test]
fn test_response_has_all_required_fields() {
    let ticket = make_ticket(SpecialistDomain::System, vec!["top_memory"], 0.8);
    let probes = vec![make_probe_result("ps aux --sort=-%mem", 0, "output")];
    let evidence = make_evidence(ticket, probes);
    let signals = make_signals(true, true, true);

    let result = ServiceDeskResult {
        request_id: "test-id".to_string(),
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: "The top memory process is...".to_string(),
        validated: true, // v0.0.298
        reliability_score: signals.score(),
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript: Transcript::new(),
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    };

    // All required fields exist and are accessible
    let _ = &result.request_id;
    let _ = &result.answer;
    let _ = &result.reliability_score;
    let _ = &result.reliability_signals;
    let _ = &result.domain;
    let _ = &result.evidence;
    let _ = &result.needs_clarification;
    let _ = &result.evidence.translator_ticket;
    let _ = &result.evidence.probes_executed;
    let _ = &result.evidence.hardware_fields;
    let _ = &result.transcript;
}
