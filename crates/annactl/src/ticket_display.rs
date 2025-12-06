//! Ticket history display for Service Desk Theatre (v0.0.109).
//!
//! Shows past support tickets with status, resolution times, and staff.
//! v0.0.110: Added search and filter capabilities.

use anna_shared::ticket_tracker::{Ticket, TicketStatus, TicketTracker};
use anna_shared::ui::colors;

/// Filter options for ticket display
#[derive(Debug, Default)]
pub struct TicketFilter {
    /// Filter by team name (case-insensitive)
    pub team: Option<String>,
    /// Filter by status
    pub status: Option<TicketStatus>,
    /// Search in query text (case-insensitive)
    pub search: Option<String>,
    /// Show only escalated tickets
    pub escalated_only: bool,
}

impl TicketFilter {
    /// Check if a ticket matches the filter
    pub fn matches(&self, ticket: &Ticket) -> bool {
        // Team filter
        if let Some(ref team) = self.team {
            if !ticket.team.to_lowercase().contains(&team.to_lowercase()) {
                return false;
            }
        }

        // Status filter
        if let Some(status) = self.status {
            if ticket.status != status {
                return false;
            }
        }

        // Search in query
        if let Some(ref search) = self.search {
            if !ticket.query.to_lowercase().contains(&search.to_lowercase()) {
                return false;
            }
        }

        // Escalated only
        if self.escalated_only && !ticket.was_escalated {
            return false;
        }

        true
    }

    /// Check if any filter is active
    pub fn is_active(&self) -> bool {
        self.team.is_some() || self.status.is_some() ||
        self.search.is_some() || self.escalated_only
    }
}

/// Display ticket history with optional filters
/// v0.0.110: Added filter support
pub fn print_ticket_history(limit: usize, filter: &TicketFilter) {
    println!();
    println!("{}Service Desk Ticket History{}", colors::BOLD, colors::RESET);

    // Show active filters
    if filter.is_active() {
        print_active_filters(filter);
    }

    println!();

    // Try system-wide first, then user-specific
    let tracker = TicketTracker::new();
    let all_tickets = match tracker.recent(limit * 2) { // Get more to filter from
        Ok(t) if !t.is_empty() => t,
        _ => {
            // Try user-specific location
            let user_tracker = TicketTracker::for_user();
            match user_tracker.recent(limit * 2) {
                Ok(t) => t,
                Err(_) => Vec::new(),
            }
        }
    };

    // Apply filters
    let tickets: Vec<_> = all_tickets.iter()
        .filter(|t| filter.matches(t))
        .take(limit)
        .collect();

    if tickets.is_empty() {
        if filter.is_active() {
            println!("{}No tickets match your filter.{}", colors::DIM, colors::RESET);
        } else {
            println!("{}No tickets found.{}", colors::DIM, colors::RESET);
            println!();
            println!("Start asking questions to create tickets!");
        }
        return;
    }

    // Print tickets
    for ticket in &tickets {
        print_ticket_row(ticket);
    }

    println!();

    // Print summary stats (unfiltered)
    if !filter.is_active() {
        print_ticket_stats();
    } else {
        println!(
            "{}Showing {} of {} tickets{}",
            colors::DIM, tickets.len(), all_tickets.len(), colors::RESET
        );
        println!();
    }
}

/// Print active filter summary
fn print_active_filters(filter: &TicketFilter) {
    print!("{}Filters:", colors::DIM);
    if let Some(ref team) = filter.team {
        print!(" team={}", team);
    }
    if let Some(status) = filter.status {
        print!(" status={}", format_status(status));
    }
    if let Some(ref search) = filter.search {
        print!(" search=\"{}\"", search);
    }
    if filter.escalated_only {
        print!(" escalated");
    }
    println!("{}", colors::RESET);
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
