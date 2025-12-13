//! Ticket History Display (Phase 69)
//!
//! Provides display functions for viewing past ticket history with outcomes,
//! specialists involved, and resolution details.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Outcome of a ticket
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketOutcome {
    /// Successfully resolved
    Resolved,
    /// Escalated to senior/human
    Escalated,
    /// User cancelled the request
    Cancelled,
    /// Could not resolve
    Failed,
    /// Still in progress
    InProgress,
    /// Awaiting user input
    AwaitingInput,
}

impl TicketOutcome {
    /// Display string for the outcome
    pub fn display(&self) -> &'static str {
        match self {
            Self::Resolved => "Resolved",
            Self::Escalated => "Escalated",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
            Self::InProgress => "In Progress",
            Self::AwaitingInput => "Awaiting Input",
        }
    }

    /// Symbol for compact display
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Resolved => "[OK]",
            Self::Escalated => "[UP]",
            Self::Cancelled => "[--]",
            Self::Failed => "[!!]",
            Self::InProgress => "[..]",
            Self::AwaitingInput => "[??]",
        }
    }

    /// Whether the ticket is considered closed
    pub fn is_closed(&self) -> bool {
        matches!(
            self,
            Self::Resolved | Self::Escalated | Self::Cancelled | Self::Failed
        )
    }
}

/// A historical ticket entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTicket {
    /// Ticket ID (e.g., CN-0001-13122025)
    pub id: String,
    /// Original user question/request
    pub query: String,
    /// When the ticket was created (Unix timestamp)
    pub created_at: u64,
    /// When the ticket was closed (if closed)
    pub closed_at: Option<u64>,
    /// Outcome of the ticket
    pub outcome: TicketOutcome,
    /// Department that handled the ticket
    pub department: Option<String>,
    /// Specialists who worked on the ticket
    pub specialists: Vec<String>,
    /// Number of interactions for this ticket
    pub interaction_count: u32,
    /// Whether Anna solved it alone
    pub anna_solo: bool,
    /// Brief summary of the resolution (if any)
    pub resolution_summary: Option<String>,
    /// Category/topic of the ticket
    pub category: Option<String>,
}

impl HistoricalTicket {
    /// Create a new historical ticket
    pub fn new(id: impl Into<String>, query: impl Into<String>, created_at: u64) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            created_at,
            closed_at: None,
            outcome: TicketOutcome::InProgress,
            department: None,
            specialists: Vec::new(),
            interaction_count: 0,
            anna_solo: false,
            resolution_summary: None,
            category: None,
        }
    }

    /// Mark the ticket as resolved
    pub fn resolve(&mut self, closed_at: u64, summary: Option<String>) {
        self.closed_at = Some(closed_at);
        self.outcome = TicketOutcome::Resolved;
        self.resolution_summary = summary;
    }

    /// Duration in milliseconds (or None if still open)
    pub fn duration_ms(&self) -> Option<u64> {
        self.closed_at.map(|end| {
            if end > self.created_at {
                (end - self.created_at) * 1000
            } else {
                0
            }
        })
    }
}

/// Ticket history storage and query
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketHistory {
    /// All tickets (most recent first)
    pub tickets: Vec<HistoricalTicket>,
    /// Total tickets ever created
    pub total_created: u64,
    /// Index by outcome for quick filtering
    pub by_outcome: HashMap<String, u64>,
    /// Index by department
    pub by_department: HashMap<String, u64>,
}

impl TicketHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a ticket to history
    pub fn add(&mut self, ticket: HistoricalTicket) {
        // Update indices
        let outcome_key = ticket.outcome.display().to_string();
        *self.by_outcome.entry(outcome_key).or_insert(0) += 1;

        if let Some(ref dept) = ticket.department {
            *self.by_department.entry(dept.clone()).or_insert(0) += 1;
        }

        self.total_created += 1;
        self.tickets.insert(0, ticket); // Most recent first

        // Keep only last 1000 tickets in memory
        if self.tickets.len() > 1000 {
            self.tickets.truncate(1000);
        }
    }

    /// Get recent tickets (default last 10)
    pub fn recent(&self, count: usize) -> &[HistoricalTicket] {
        let end = count.min(self.tickets.len());
        &self.tickets[..end]
    }

    /// Get tickets by outcome
    pub fn by_outcome(&self, outcome: &TicketOutcome) -> Vec<&HistoricalTicket> {
        self.tickets
            .iter()
            .filter(|t| &t.outcome == outcome)
            .collect()
    }

    /// Get open tickets
    pub fn open_tickets(&self) -> Vec<&HistoricalTicket> {
        self.tickets
            .iter()
            .filter(|t| !t.outcome.is_closed())
            .collect()
    }

    /// Get tickets by department
    pub fn by_department(&self, department: &str) -> Vec<&HistoricalTicket> {
        self.tickets
            .iter()
            .filter(|t| t.department.as_deref() == Some(department))
            .collect()
    }

    /// Count resolved tickets
    pub fn resolved_count(&self) -> u64 {
        *self.by_outcome.get("Resolved").unwrap_or(&0)
    }

    /// Count failed tickets
    pub fn failed_count(&self) -> u64 {
        *self.by_outcome.get("Failed").unwrap_or(&0)
    }

    /// Success rate (resolved / total closed)
    pub fn success_rate(&self) -> f64 {
        let closed: u64 = self.tickets.iter().filter(|t| t.outcome.is_closed()).count() as u64;
        if closed == 0 {
            return 0.0;
        }
        (self.resolved_count() as f64 / closed as f64) * 100.0
    }

    /// Get most active department
    pub fn most_active_department(&self) -> Option<(&String, u64)> {
        self.by_department.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }
}

/// Format a timestamp as a human-readable date
pub fn format_timestamp(ts: u64) -> String {
    // Simple formatting: days ago or date
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if ts > now {
        return "just now".to_string();
    }

    let diff = now - ts;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}

/// Format duration in milliseconds as human-readable
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3600000 {
        format!("{:.1}m", ms as f64 / 60000.0)
    } else {
        format!("{:.1}h", ms as f64 / 3600000.0)
    }
}

/// Format ticket history as full display
pub fn format_ticket_history(history: &TicketHistory) -> String {
    let mut lines = Vec::new();

    lines.push("=== Ticket History ===".to_string());
    lines.push(String::new());

    // Summary
    lines.push(format!("Total Tickets: {}", history.total_created));
    lines.push(format!("Success Rate: {:.1}%", history.success_rate()));

    let open = history.open_tickets().len();
    if open > 0 {
        lines.push(format!("Open Tickets: {}", open));
    }

    lines.push(String::new());

    // Recent tickets
    lines.push("--- Recent Tickets ---".to_string());
    let recent = history.recent(10);

    if recent.is_empty() {
        lines.push("  No tickets yet.".to_string());
    } else {
        for ticket in recent {
            let status = ticket.outcome.symbol();
            let time = format_timestamp(ticket.created_at);
            let duration = ticket
                .duration_ms()
                .map(|d| format!(" ({})", format_duration(d)))
                .unwrap_or_default();

            // Truncate query for display
            let query = if ticket.query.len() > 50 {
                format!("{}...", &ticket.query[..47])
            } else {
                ticket.query.clone()
            };

            lines.push(format!("{} {} - {}{}", status, ticket.id, query, duration));

            if !ticket.specialists.is_empty() {
                lines.push(format!(
                    "    Handled by: {}",
                    ticket.specialists.join(", ")
                ));
            } else if ticket.anna_solo {
                lines.push("    Handled by: Anna (solo)".to_string());
            }

            if let Some(ref summary) = ticket.resolution_summary {
                let summary_display = if summary.len() > 60 {
                    format!("{}...", &summary[..57])
                } else {
                    summary.clone()
                };
                lines.push(format!("    Resolution: {}", summary_display));
            }

            lines.push(format!("    Time: {}", time));
        }
    }

    // Outcome breakdown
    if !history.by_outcome.is_empty() {
        lines.push(String::new());
        lines.push("--- By Outcome ---".to_string());
        for (outcome, count) in &history.by_outcome {
            lines.push(format!("  {}: {}", outcome, count));
        }
    }

    // Department breakdown
    if !history.by_department.is_empty() {
        lines.push(String::new());
        lines.push("--- By Department ---".to_string());
        for (dept, count) in &history.by_department {
            lines.push(format!("  {}: {}", dept, count));
        }
    }

    lines.join("\n")
}

/// Format ticket history compact (for greetings)
pub fn format_ticket_history_compact(history: &TicketHistory) -> String {
    let recent = history.recent(3);
    if recent.is_empty() {
        return "No tickets yet.".to_string();
    }

    let entries: Vec<String> = recent
        .iter()
        .map(|t| {
            let query = if t.query.len() > 30 {
                format!("{}...", &t.query[..27])
            } else {
                t.query.clone()
            };
            format!("{} {}", t.outcome.symbol(), query)
        })
        .collect();

    entries.join(" | ")
}

/// Format ticket history one-line
pub fn format_ticket_history_oneline(history: &TicketHistory) -> String {
    let resolved = history.resolved_count();
    let total = history.total_created;
    let open = history.open_tickets().len();

    if open > 0 {
        format!(
            "Tickets: {} total, {} resolved, {} open ({:.0}% success)",
            total,
            resolved,
            open,
            history.success_rate()
        )
    } else {
        format!(
            "Tickets: {} total, {} resolved ({:.0}% success)",
            total,
            resolved,
            history.success_rate()
        )
    }
}

/// Generate a fun fact about ticket history
pub fn ticket_history_fun_fact(history: &TicketHistory) -> Option<String> {
    if history.tickets.is_empty() {
        return None;
    }

    let facts = vec![
        history.total_created >= 100,
        history.total_created >= 50,
        history.resolved_count() >= 10,
        history.success_rate() >= 90.0,
        history.open_tickets().is_empty() && history.total_created > 0,
    ];

    let messages = vec![
        format!(
            "Century club! You've opened {} tickets with Anna.",
            history.total_created
        ),
        format!(
            "Half century! {} tickets processed together.",
            history.total_created
        ),
        format!(
            "Double digits! {} tickets resolved successfully.",
            history.resolved_count()
        ),
        format!(
            "Quality service! {:.0}% success rate on closed tickets.",
            history.success_rate()
        ),
        "All caught up! No open tickets pending.".to_string(),
    ];

    for (i, fact) in facts.iter().enumerate() {
        if *fact {
            return Some(messages[i].clone());
        }
    }

    // Default fact
    Some(format!(
        "You've worked through {} tickets with Anna.",
        history.total_created
    ))
}

/// Check if query is asking about ticket history
pub fn is_ticket_history_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "ticket history",
        "past tickets",
        "previous tickets",
        "my tickets",
        "ticket log",
        "case history",
        "past cases",
        "recent tickets",
        "show tickets",
        "list tickets",
        "ticket status",
        "open tickets",
        "closed tickets",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

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
