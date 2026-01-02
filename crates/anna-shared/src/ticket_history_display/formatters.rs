//! Formatting functions for ticket history display (Phase 69)
//!
//! Provides various display formats for ticket history: full, compact, one-line, and utility functions.

use super::types::{TicketHistory, TicketOutcome};

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
