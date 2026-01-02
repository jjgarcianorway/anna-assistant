//! Tests for ticket state machine

use super::{
    error::ErrorKind, handler::HandlerType, handler::SolverTier, live_ticket::LiveTicket,
    outcome::TicketOutcome, state::TicketState,
};

#[test]
fn test_ticket_lifecycle() {
    let mut ticket = LiveTicket::new("TEST-001", "What is my disk usage?");
    assert_eq!(ticket.state, TicketState::Created);
    assert!(!ticket.is_terminal());

    ticket.mark_planned("storage", "diagnose");
    assert_eq!(ticket.state, TicketState::Planned);
    assert_eq!(ticket.domain, "storage");

    ticket.mark_probes_run(Some("df -h: 75% used".to_string()));
    assert_eq!(ticket.state, TicketState::ProbesRun);

    ticket.mark_answered("Your disk is 75% full", 0.85);
    assert_eq!(ticket.state, TicketState::Answered);
    assert!(ticket.reached_answered());

    ticket.mark_success();
    assert_eq!(ticket.state, TicketState::Success);
    assert!(ticket.is_terminal());
    assert!(ticket.is_success());
}

#[test]
fn test_ticket_outcome() {
    let mut ticket = LiveTicket::new("TEST-004", "Test outcome");
    ticket.mark_planned("system", "diagnose");
    ticket.mark_answered("Answer", 0.9);
    ticket.mark_success_with_outcome(TicketOutcome::Success, "Sofia (Jr)");

    assert_eq!(ticket.get_outcome(), TicketOutcome::Success);
    assert_eq!(ticket.handled_by, Some("Sofia (Jr)".to_string()));
    assert!(ticket.is_success());
    assert!(!ticket.is_error());
}

#[test]
fn test_ticket_partial() {
    let mut ticket = LiveTicket::new("TEST-005", "Test partial");
    ticket.mark_planned("network", "diagnose");
    ticket.mark_partial("Partial answer", "Tomas (Sr)");

    assert_eq!(ticket.get_outcome(), TicketOutcome::Partial);
    assert!(!ticket.is_success());
    assert!(!ticket.is_error());
}

#[test]
fn test_error_kind_to_outcome() {
    assert_eq!(
        ErrorKind::LlmTimeout.to_outcome(),
        TicketOutcome::ErrorTimeout
    );
    assert_eq!(
        ErrorKind::LlmParseError.to_outcome(),
        TicketOutcome::ErrorParse
    );
    assert_eq!(
        ErrorKind::ProbeFailure.to_outcome(),
        TicketOutcome::ErrorTool
    );
    assert_eq!(
        ErrorKind::MissingEvidence.to_outcome(),
        TicketOutcome::CannotAnswerSafely
    );
}

#[test]
fn test_ticket_failure() {
    let mut ticket = LiveTicket::new("TEST-002", "Test query");
    ticket.mark_planned("system", "diagnose");
    ticket.mark_probes_run(None);
    ticket.mark_llm_requested(HandlerType::LlmSolver {
        tier: SolverTier::Junior,
        model: "test".to_string(),
    });
    ticket.mark_llm_failed(ErrorKind::LlmTimeout, Some("15s exceeded".to_string()));

    assert_eq!(ticket.state, TicketState::LlmFailed);
    assert_eq!(ticket.error_kind, Some(ErrorKind::LlmTimeout));
    assert!(!ticket.is_success());
}

#[test]
fn test_state_transitions() {
    let mut ticket = LiveTicket::new("TEST-003", "Test");
    ticket.mark_planned("test", "test");
    ticket.mark_probes_run(None);

    assert_eq!(ticket.transitions.len(), 2);
    assert_eq!(ticket.transitions[0].from, TicketState::Created);
    assert_eq!(ticket.transitions[0].to, TicketState::Planned);
}

#[test]
fn test_handler_display() {
    let recipe = HandlerType::Recipe {
        name: "check_disk".to_string(),
    };
    assert_eq!(recipe.to_string(), "recipe:check_disk");

    let llm = HandlerType::LlmSolver {
        tier: SolverTier::Junior,
        model: "qwen2.5".to_string(),
    };
    assert_eq!(llm.to_string(), "llm:junior:qwen2.5");
}
