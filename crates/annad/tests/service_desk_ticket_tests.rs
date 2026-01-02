//! Integration tests for service desk TranslatorTicket structure.
//!
//! These tests verify:
//! - Translator ticket structure
//! - Ticket with clarification

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};

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

// === TranslatorTicket Tests ===

#[test]
fn test_translator_ticket_structure() {
    let ticket = make_ticket(SpecialistDomain::System, vec!["top_memory"], 0.85);

    assert_eq!(ticket.intent, QueryIntent::Question);
    assert_eq!(ticket.domain, SpecialistDomain::System);
    assert_eq!(ticket.needs_probes.len(), 1);
    assert!(ticket.confidence > 0.7);
    assert!(ticket.clarification_question.is_none());
}

#[test]
fn test_translator_ticket_with_clarification() {
    let ticket = TranslatorTicket {
        intent: QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: Some("Could you provide more details?".to_string()),
        confidence: 0.3,
        answer_contract: None,
    };

    assert!(ticket.clarification_question.is_some());
    assert!(ticket.confidence < 0.5);
}
