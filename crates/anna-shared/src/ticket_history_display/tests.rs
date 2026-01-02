//! Tests for ticket history display (Phase 69)

#[cfg(test)]
mod tests {
    use crate::ticket_history_display::formatters::{
        format_duration, format_ticket_history, format_ticket_history_compact,
        format_ticket_history_oneline, format_timestamp, is_ticket_history_query,
        ticket_history_fun_fact,
    };
    use crate::ticket_history_display::types::{HistoricalTicket, TicketHistory, TicketOutcome};

    #[test]
    fn test_ticket_outcome_display() {
        assert_eq!(TicketOutcome::Resolved.display(), "Resolved");
        assert_eq!(TicketOutcome::Escalated.symbol(), "[UP]");
        assert!(TicketOutcome::Resolved.is_closed());
        assert!(!TicketOutcome::InProgress.is_closed());
    }

    #[test]
    fn test_historical_ticket_new() {
        let ticket = HistoricalTicket::new("CN-0001", "How do I install vim?", 1000);
        assert_eq!(ticket.id, "CN-0001");
        assert_eq!(ticket.query, "How do I install vim?");
        assert_eq!(ticket.outcome, TicketOutcome::InProgress);
        assert!(ticket.closed_at.is_none());
    }

    #[test]
    fn test_historical_ticket_resolve() {
        let mut ticket = HistoricalTicket::new("CN-0002", "Enable syntax highlighting", 1000);
        ticket.resolve(2000, Some("Added syntax on to .vimrc".to_string()));

        assert_eq!(ticket.outcome, TicketOutcome::Resolved);
        assert_eq!(ticket.closed_at, Some(2000));
        assert!(ticket.resolution_summary.is_some());
        assert_eq!(ticket.duration_ms(), Some(1000000)); // 1000 seconds * 1000
    }

    #[test]
    fn test_ticket_history_add() {
        let mut history = TicketHistory::new();
        let ticket = HistoricalTicket::new("CN-0001", "Test query", 1000);
        history.add(ticket);

        assert_eq!(history.total_created, 1);
        assert_eq!(history.tickets.len(), 1);
    }

    #[test]
    fn test_ticket_history_recent() {
        let mut history = TicketHistory::new();
        for i in 0..15 {
            let ticket = HistoricalTicket::new(format!("CN-{:04}", i), "Query", i as u64 * 1000);
            history.add(ticket);
        }

        let recent = history.recent(5);
        assert_eq!(recent.len(), 5);
        // Most recent first
        assert_eq!(recent[0].id, "CN-0014");
    }

    #[test]
    fn test_ticket_history_by_outcome() {
        let mut history = TicketHistory::new();

        let mut resolved = HistoricalTicket::new("CN-0001", "Query 1", 1000);
        resolved.outcome = TicketOutcome::Resolved;
        history.add(resolved);

        let mut failed = HistoricalTicket::new("CN-0002", "Query 2", 2000);
        failed.outcome = TicketOutcome::Failed;
        history.add(failed);

        let resolved_tickets = history.by_outcome(&TicketOutcome::Resolved);
        assert_eq!(resolved_tickets.len(), 1);
        assert_eq!(resolved_tickets[0].id, "CN-0001");
    }

    #[test]
    fn test_ticket_history_open_tickets() {
        let mut history = TicketHistory::new();

        let mut resolved = HistoricalTicket::new("CN-0001", "Query 1", 1000);
        resolved.outcome = TicketOutcome::Resolved;
        history.add(resolved);

        let in_progress = HistoricalTicket::new("CN-0002", "Query 2", 2000);
        history.add(in_progress);

        let open = history.open_tickets();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, "CN-0002");
    }

    #[test]
    fn test_success_rate() {
        let mut history = TicketHistory::new();

        // Add 3 resolved, 1 failed
        for i in 0..3 {
            let mut ticket = HistoricalTicket::new(format!("CN-{:04}", i), "Query", i as u64);
            ticket.outcome = TicketOutcome::Resolved;
            history.add(ticket);
        }

        let mut failed = HistoricalTicket::new("CN-0003", "Failed query", 3000);
        failed.outcome = TicketOutcome::Failed;
        history.add(failed);

        // 3 resolved out of 4 closed = 75%
        assert!((history.success_rate() - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_format_timestamp() {
        // Just now case
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        assert_eq!(format_timestamp(now), "just now");
        assert!(format_timestamp(now - 120).contains("m ago"));
        assert!(format_timestamp(now - 7200).contains("h ago"));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(90000), "1.5m");
        assert_eq!(format_duration(5400000), "1.5h");
    }

    #[test]
    fn test_format_ticket_history() {
        let mut history = TicketHistory::new();
        let ticket = HistoricalTicket::new("CN-0001", "How do I list files?", 1000);
        history.add(ticket);

        let output = format_ticket_history(&history);
        assert!(output.contains("Ticket History"));
        assert!(output.contains("CN-0001"));
        assert!(output.contains("list files"));
    }

    #[test]
    fn test_format_ticket_history_compact() {
        let mut history = TicketHistory::new();

        let mut ticket = HistoricalTicket::new("CN-0001", "Short query", 1000);
        ticket.outcome = TicketOutcome::Resolved;
        history.add(ticket);

        let output = format_ticket_history_compact(&history);
        assert!(output.contains("[OK]"));
        assert!(output.contains("Short query"));
    }

    #[test]
    fn test_format_ticket_history_oneline() {
        let mut history = TicketHistory::new();

        let mut ticket = HistoricalTicket::new("CN-0001", "Query", 1000);
        ticket.outcome = TicketOutcome::Resolved;
        history.add(ticket);

        let output = format_ticket_history_oneline(&history);
        assert!(output.contains("Tickets:"));
        assert!(output.contains("1 total"));
        assert!(output.contains("1 resolved"));
    }

    #[test]
    fn test_ticket_history_fun_fact() {
        let mut history = TicketHistory::new();

        // Empty history
        assert!(ticket_history_fun_fact(&history).is_none());

        // Add one ticket
        let mut ticket = HistoricalTicket::new("CN-0001", "Query", 1000);
        ticket.outcome = TicketOutcome::Resolved;
        history.add(ticket);

        let fact = ticket_history_fun_fact(&history);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_ticket_history_query() {
        assert!(is_ticket_history_query("show me my ticket history"));
        assert!(is_ticket_history_query("what are my past tickets?"));
        assert!(is_ticket_history_query("list my recent tickets"));
        assert!(is_ticket_history_query("show open tickets"));
        assert!(!is_ticket_history_query("how do I install vim?"));
        assert!(!is_ticket_history_query("restart docker"));
    }

    #[test]
    fn test_most_active_department() {
        let mut history = TicketHistory::new();

        for i in 0..5 {
            let mut ticket = HistoricalTicket::new(format!("CN-{:04}", i), "Query", i as u64);
            ticket.department = Some("Desktop".to_string());
            history.add(ticket);
        }

        for i in 5..7 {
            let mut ticket = HistoricalTicket::new(format!("CN-{:04}", i), "Query", i as u64);
            ticket.department = Some("Network".to_string());
            history.add(ticket);
        }

        let (dept, count) = history.most_active_department().unwrap();
        assert_eq!(dept, "Desktop");
        assert_eq!(count, 5);
    }
}
