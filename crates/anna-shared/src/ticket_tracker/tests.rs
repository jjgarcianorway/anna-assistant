//! Tests for ticket_tracker module (v0.0.183).

#[cfg(test)]
mod tests {
    use crate::ticket_tracker::{Ticket, TicketStatus, TicketTracker};

    #[test]
    fn test_case_number_format() {
        let tracker = TicketTracker::new();
        let case = tracker.next_case_number();
        assert!(case.starts_with("CN-"));
        assert!(case.len() > 10);
    }

    #[test]
    fn test_ticket_lifecycle() {
        let mut ticket = Ticket::new(
            "CN-0001-06122025".to_string(),
            "How much RAM?".to_string(),
            "hardware".to_string(),
        );

        assert_eq!(ticket.status, TicketStatus::New);
        assert!(ticket.is_open());

        ticket.assign("hardware_jr");
        assert_eq!(ticket.status, TicketStatus::Assigned);

        ticket.start_work();
        assert_eq!(ticket.status, TicketStatus::InProgress);
        assert_eq!(ticket.interaction_count, 1);

        ticket.resolve("16 GB".to_string(), 95, 150);
        assert_eq!(ticket.status, TicketStatus::Resolved);
        assert!(!ticket.is_open());
    }

    #[test]
    fn test_ticket_escalation() {
        let mut ticket = Ticket::new(
            "CN-0002-06122025".to_string(),
            "Complex network issue".to_string(),
            "network".to_string(),
        );

        ticket.assign("network_jr");
        ticket.start_work();
        ticket.escalate("network_sr");

        assert!(ticket.was_escalated);
        assert_eq!(ticket.status, TicketStatus::Escalated);
        assert_eq!(ticket.assigned_to, Some("network_sr".to_string()));
    }
}
