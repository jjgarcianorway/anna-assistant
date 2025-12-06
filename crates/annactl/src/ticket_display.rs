//! Ticket history display for Service Desk Theatre (v0.0.109).
//!
//! Shows past support tickets with status, resolution times, and staff.

use anna_shared::ticket_tracker::{Ticket, TicketStatus, TicketTracker};
use anna_shared::ui::colors;

/// Display ticket history
pub fn print_ticket_history(limit: usize) {
    println!();
    println!("{}Service Desk Ticket History{}", colors::BOLD, colors::RESET);
    println!();

    // Try system-wide first, then user-specific
    let tracker = TicketTracker::new();
    let tickets = match tracker.recent(limit) {
        Ok(t) if !t.is_empty() => t,
        _ => {
            // Try user-specific location
            let user_tracker = TicketTracker::for_user();
            match user_tracker.recent(limit) {
                Ok(t) => t,
                Err(_) => Vec::new(),
            }
        }
    };

    if tickets.is_empty() {
        println!("{}No tickets found.{}", colors::DIM, colors::RESET);
        println!();
        println!("Start asking questions to create tickets!");
        return;
    }

    // Print tickets
    for ticket in &tickets {
        print_ticket_row(ticket);
    }

    println!();

    // Print summary stats
    print_ticket_stats();
}

/// Print a single ticket row
fn print_ticket_row(ticket: &Ticket) {
    let status_color = status_color(ticket.status);
    let status_str = format_status(ticket.status);

    // Case number and status
    print!(
        "{}{}{} {}{}{}",
        colors::CYAN, ticket.case_number, colors::RESET,
        status_color, status_str, colors::RESET
    );

    // Team
    print!(" {}{}{}", colors::DIM, ticket.team, colors::RESET);

    // Resolution info
    if let Some(ms) = ticket.resolution_ms {
        let time_str = format_duration_ms(ms);
        print!(" {}({}){}", colors::DIM, time_str, colors::RESET);
    }

    // Reliability badge
    if let Some(rel) = ticket.reliability {
        let rel_color = reliability_color(rel);
        print!(" {}{}%{}", rel_color, rel, colors::RESET);
    }

    // Escalated indicator
    if ticket.was_escalated {
        print!(" {}[escalated]{}", colors::WARN, colors::RESET);
    }

    println!();

    // Query (truncated)
    let query = truncate_query(&ticket.query, 60);
    println!("  {}› {}{}", colors::DIM, query, colors::RESET);
    println!();
}

/// Print ticket statistics summary
fn print_ticket_stats() {
    let tracker = TicketTracker::new();
    let stats = match tracker.stats() {
        Ok(s) => s,
        Err(_) => {
            // Try user location
            let user_tracker = TicketTracker::for_user();
            match user_tracker.stats() {
                Ok(s) => s,
                Err(_) => return,
            }
        }
    };

    if stats.total_tickets == 0 {
        return;
    }

    println!("{}Summary{}", colors::BOLD, colors::RESET);
    println!();

    // Basic counts
    let resolve_rate = if stats.total_tickets > 0 {
        (stats.resolved_tickets as f64 / stats.total_tickets as f64) * 100.0
    } else {
        0.0
    };

    let escalation_rate = if stats.total_tickets > 0 {
        (stats.escalated_tickets as f64 / stats.total_tickets as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "  Total tickets:     {}",
        stats.total_tickets
    );
    println!(
        "  Resolved:          {} ({:.0}%)",
        stats.resolved_tickets, resolve_rate
    );
    println!(
        "  Escalated:         {} ({:.0}%)",
        stats.escalated_tickets, escalation_rate
    );

    // Average resolution time
    if stats.avg_resolution_ms > 0 {
        let time_str = format_duration_ms(stats.avg_resolution_ms);
        println!(
            "  Avg resolution:    {}",
            time_str
        );
    }

    // Average reliability
    if stats.avg_reliability > 0.0 {
        println!(
            "  Avg reliability:   {:.0}%",
            stats.avg_reliability
        );
    }

    println!();
}

/// Format status for display
fn format_status(status: TicketStatus) -> &'static str {
    match status {
        TicketStatus::New => "[new]",
        TicketStatus::Assigned => "[assigned]",
        TicketStatus::InProgress => "[working]",
        TicketStatus::PendingUser => "[pending]",
        TicketStatus::Escalated => "[escalated]",
        TicketStatus::Resolved => "[resolved]",
        TicketStatus::Closed => "[closed]",
    }
}

/// Get color for status
fn status_color(status: TicketStatus) -> &'static str {
    match status {
        TicketStatus::New | TicketStatus::Assigned => colors::CYAN,
        TicketStatus::InProgress => colors::WARN,
        TicketStatus::PendingUser => colors::WARN,
        TicketStatus::Escalated => colors::ERR,
        TicketStatus::Resolved => colors::OK,
        TicketStatus::Closed => colors::DIM,
    }
}

/// Get color for reliability score
fn reliability_color(score: u8) -> &'static str {
    match score {
        80..=100 => colors::OK,
        50..=79 => colors::WARN,
        _ => colors::ERR,
    }
}

/// Format duration in milliseconds to human-readable
fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let minutes = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        format!("{}m {}s", minutes, secs)
    }
}

/// Truncate query for display
fn truncate_query(query: &str, max_len: usize) -> String {
    let query = query.replace('\n', " ");
    if query.len() <= max_len {
        query
    } else {
        format!("{}...", &query[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(1500), "1.5s");
        assert_eq!(format_duration_ms(65000), "1m 5s");
    }

    #[test]
    fn test_truncate_query() {
        assert_eq!(truncate_query("short", 10), "short");
        assert_eq!(truncate_query("this is a very long query", 15), "this is a ve...");
    }

    #[test]
    fn test_status_color() {
        assert_eq!(status_color(TicketStatus::Resolved), colors::OK);
        assert_eq!(status_color(TicketStatus::Escalated), colors::ERR);
    }
}
