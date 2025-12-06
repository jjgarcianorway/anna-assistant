//! Golden tests for triage router behavior.
//!
//! Verifies confidence thresholds, probe caps, and clarification fallbacks.

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};

// Re-implement triage logic for testing
mod triage {
    use super::*;

    pub const MAX_TRIAGE_PROBES: usize = 3;
    pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.7;

    pub struct TriageResult {
        pub ticket: TranslatorTicket,
        pub probe_cap_applied: bool,
        pub needs_immediate_clarification: bool,
        pub clarification_question: Option<String>,
    }

    pub fn apply_triage_rules(mut ticket: TranslatorTicket) -> TriageResult {
        let mut result = TriageResult {
            ticket: ticket.clone(),
            probe_cap_applied: false,
            needs_immediate_clarification: false,
            clarification_question: None,
        };

        // Rule 1: Cap probes
        if result.ticket.needs_probes.len() > MAX_TRIAGE_PROBES {
            result.ticket.needs_probes.truncate(MAX_TRIAGE_PROBES);
            result.probe_cap_applied = true;
        }

        // Rule 2: Low confidence triggers clarification
        if result.ticket.confidence < MIN_CONFIDENCE_THRESHOLD {
            result.needs_immediate_clarification = true;
            result.clarification_question = result.ticket.clarification_question.clone()
                .or_else(|| Some(generate_fallback_clarification(&result.ticket)));
        }

        result
    }

    pub fn generate_fallback_clarification(ticket: &TranslatorTicket) -> String {
        match ticket.domain {
            SpecialistDomain::Network => "Could you clarify what network information you need?".to_string(),
            SpecialistDomain::Storage => "What storage information are you looking for?".to_string(),
            SpecialistDomain::Security => "What security aspect are you concerned about?".to_string(),
            SpecialistDomain::Packages => "What package operation do you need help with?".to_string(),
            SpecialistDomain::System => "Could you provide more details?".to_string(),
        }
    }
}

// === Confidence Threshold Tests ===

#[test]
fn test_high_confidence_proceeds() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec!["top_cpu".to_string()],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.85,
    };

    let result = triage::apply_triage_rules(ticket);
    assert!(!result.needs_immediate_clarification);
    assert!(result.clarification_question.is_none());
}

#[test]
fn test_low_confidence_triggers_clarification() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec!["top_cpu".to_string()],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.5,
    };

    let result = triage::apply_triage_rules(ticket);
    assert!(result.needs_immediate_clarification);
    assert!(result.clarification_question.is_some());
}

#[test]
fn test_threshold_boundary() {
    // Exactly 0.7 should NOT trigger clarification
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.7,
    };
    let result = triage::apply_triage_rules(ticket);
    assert!(!result.needs_immediate_clarification);

    // Just below should trigger
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.69,
    };
    let result = triage::apply_triage_rules(ticket);
    assert!(result.needs_immediate_clarification);
}

// === Probe Cap Tests ===

#[test]
fn test_probe_cap_at_three() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![
            "top_cpu".to_string(),
            "top_memory".to_string(),
            "disk_usage".to_string(),
            "network_addrs".to_string(),
            "listening_ports".to_string(),
        ],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.9,
    };

    let result = triage::apply_triage_rules(ticket);
    assert!(result.probe_cap_applied);
    assert_eq!(result.ticket.needs_probes.len(), 3);
}

#[test]
fn test_probe_cap_not_applied_under_limit() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec!["top_cpu".to_string(), "top_memory".to_string()],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.9,
    };

    let result = triage::apply_triage_rules(ticket);
    assert!(!result.probe_cap_applied);
    assert_eq!(result.ticket.needs_probes.len(), 2);
}

#[test]
fn test_probe_cap_exactly_three() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![
            "top_cpu".to_string(),
            "top_memory".to_string(),
            "disk_usage".to_string(),
        ],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.9,
    };

    let result = triage::apply_triage_rules(ticket);
    assert!(!result.probe_cap_applied);
    assert_eq!(result.ticket.needs_probes.len(), 3);
}

// === Fallback Clarification Tests ===

#[test]
fn test_fallback_clarification_by_domain() {
    for (domain, expected_keyword) in [
        (SpecialistDomain::Network, "network"),
        (SpecialistDomain::Storage, "storage"),
        (SpecialistDomain::Security, "security"),
        (SpecialistDomain::Packages, "package"),
        (SpecialistDomain::System, "details"),
    ] {
        let ticket = TranslatorTicket {
            intent: QueryIntent::Question,
            domain,
            entities: vec![],
            needs_probes: vec![],
            clarification_question: None,
            answer_contract: None,
            confidence: 0.3,
        };

        let result = triage::apply_triage_rules(ticket);
        assert!(result.needs_immediate_clarification);
        let question = result.clarification_question.unwrap();
        assert!(
            question.to_lowercase().contains(expected_keyword),
            "Domain {:?} should have question containing '{}', got: {}",
            domain,
            expected_keyword,
            question
        );
    }
}

#[test]
fn test_llm_clarification_preserved() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: Some("Which specific process are you asking about?".to_string()),
        answer_contract: None,
            confidence: 0.4,
    };

    let result = triage::apply_triage_rules(ticket);
    assert!(result.needs_immediate_clarification);
    assert_eq!(
        result.clarification_question,
        Some("Which specific process are you asking about?".to_string())
    );
}

// === Combined Scenario Tests ===

#[test]
fn test_low_confidence_with_probe_cap() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ],
        clarification_question: None,
        answer_contract: None,
            confidence: 0.3,
    };

    let result = triage::apply_triage_rules(ticket);
    assert!(result.probe_cap_applied);
    assert!(result.needs_immediate_clarification);
    assert_eq!(result.ticket.needs_probes.len(), 3);
}
