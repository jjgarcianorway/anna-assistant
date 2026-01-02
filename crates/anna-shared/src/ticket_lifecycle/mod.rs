//! Strict ticket lifecycle state machine (v0.0.426).
//!
//! This module enforces a finite state machine for each ticket with:
//! - Strict state transitions (no skipping states)
//! - Clear "resolved" vs "failed" semantics
//! - Connection to specialist JSON outcomes
//! - Honest metrics based on actual outcomes

mod errors;
mod metrics;
mod record;
mod states;

// Re-export all public types
pub use errors::InternalError;
pub use metrics::{
    compute_specialist_metrics, format_specialist_roster, ReliabilityMetrics, SpecialistMetrics,
};
pub use record::{StateTransition, TicketRecord};
pub use states::{TicketLifecycleState, TicketResolution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_v3::SpecialistResponse;

    #[test]
    fn test_ticket_lifecycle_success() {
        let mut ticket = TicketRecord::new("DSK-001", "Why is my boot slow?");
        assert_eq!(ticket.state, TicketLifecycleState::New);

        ticket.start_processing("desktop.junior").unwrap();
        assert_eq!(ticket.state, TicketLifecycleState::InProgress);

        let response = SpecialistResponse::success("DSK-001", "Boot time is normal at 15s");
        ticket.mark_answered(&response).unwrap();
        assert_eq!(ticket.state, TicketLifecycleState::Answered);

        ticket
            .mark_user_satisfied("Your boot time is normal")
            .unwrap();
        assert_eq!(ticket.state, TicketLifecycleState::UserSatisfied);
        assert_eq!(ticket.resolution(), TicketResolution::ResolvedSuccess);
    }

    #[test]
    fn test_ticket_lifecycle_failure() {
        let mut ticket = TicketRecord::new("DSK-002", "Test query");
        ticket.start_processing("desktop.junior").unwrap();

        let error = InternalError::ParseError {
            attempts: 2,
            last_error: "Invalid JSON".to_string(),
        };
        ticket.mark_failed(error).unwrap();

        assert_eq!(ticket.state, TicketLifecycleState::Failed);
        assert_eq!(ticket.resolution(), TicketResolution::Failed);
        assert!(ticket.internal_error.is_some());
    }

    #[test]
    fn test_ticket_lifecycle_honest_unknown() {
        let mut ticket = TicketRecord::new("DSK-003", "Unknown topic");
        ticket.start_processing("desktop.junior").unwrap();

        let response = SpecialistResponse::no_data("DSK-003", "No data available");
        ticket.mark_answered(&response).unwrap();
        ticket
            .mark_user_satisfied("I couldn't find information about this")
            .unwrap();

        assert_eq!(ticket.resolution(), TicketResolution::ResolvedHonestUnknown);
    }

    #[test]
    fn test_invalid_transition() {
        let mut ticket = TicketRecord::new("DSK-004", "Test");
        // Can't go directly to Answered from New
        let result = ticket.transition(TicketLifecycleState::Answered, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_reliability_metrics() {
        let tickets = vec![
            create_success_ticket("T1"),
            create_success_ticket("T2"),
            create_partial_ticket("T3"),
            create_failed_ticket("T4"),
            create_honest_unknown_ticket("T5"),
        ];

        let metrics = ReliabilityMetrics::compute(&tickets);
        assert_eq!(metrics.total_tickets, 5);
        assert_eq!(metrics.resolved_success, 2);
        assert_eq!(metrics.resolved_partial, 1);
        assert_eq!(metrics.honest_unknown, 1);
        assert_eq!(metrics.failed, 1);
        assert_eq!(metrics.success_rate, 40.0); // 2/5
    }

    #[test]
    fn test_specialist_metrics() {
        let tickets = vec![
            create_success_ticket_with_specialist("T1", "desktop.junior"),
            create_success_ticket_with_specialist("T2", "desktop.junior"),
            create_failed_ticket_with_specialist("T3", "desktop.junior"),
            create_success_ticket_with_specialist("T4", "desktop.senior"),
        ];

        let metrics = compute_specialist_metrics(&tickets);

        let junior = metrics.get("desktop.junior").unwrap();
        assert_eq!(junior.tickets_lead, 3);
        assert_eq!(junior.success_count, 2);
        assert_eq!(junior.failed_count, 1);
        assert!((junior.success_rate - 66.67).abs() < 1.0);

        let senior = metrics.get("desktop.senior").unwrap();
        assert_eq!(senior.tickets_lead, 1);
        assert_eq!(senior.success_count, 1);
        assert_eq!(senior.success_rate, 100.0);
    }

    #[test]
    fn test_xp_and_title() {
        let mut m = SpecialistMetrics {
            specialist_id: "test".to_string(),
            tickets_lead: 100,
            success_count: 90,
            xp: 2000,
            success_rate: 90.0,
            ..Default::default()
        };
        assert_eq!(m.title(), "Senior");

        m.xp = 500;
        m.success_rate = 75.0;
        assert_eq!(m.title(), "Proficient");

        m.xp = 100;
        m.success_rate = 50.0;
        assert_eq!(m.title(), "Apprentice");
    }

    // Helper functions for tests
    fn create_success_ticket(id: &str) -> TicketRecord {
        create_success_ticket_with_specialist(id, "desktop.junior")
    }

    fn create_success_ticket_with_specialist(id: &str, specialist: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing(specialist).unwrap();
        let response = SpecialistResponse::success(id, "Success");
        ticket.mark_answered(&response).unwrap();
        ticket.mark_user_satisfied("Answer").unwrap();
        ticket
    }

    fn create_partial_ticket(id: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing("desktop.junior").unwrap();
        let response = SpecialistResponse::partial(id, "Partial");
        ticket.mark_answered(&response).unwrap();
        ticket.mark_user_satisfied("Partial answer").unwrap();
        ticket
    }

    fn create_failed_ticket(id: &str) -> TicketRecord {
        create_failed_ticket_with_specialist(id, "desktop.junior")
    }

    fn create_failed_ticket_with_specialist(id: &str, specialist: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing(specialist).unwrap();
        ticket
            .mark_failed(InternalError::ParseError {
                attempts: 2,
                last_error: "Invalid JSON".to_string(),
            })
            .unwrap();
        ticket
    }

    fn create_honest_unknown_ticket(id: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing("desktop.junior").unwrap();
        let response = SpecialistResponse::no_data(id, "No data");
        ticket.mark_answered(&response).unwrap();
        ticket.mark_user_satisfied("I don't know").unwrap();
        ticket
    }
}
