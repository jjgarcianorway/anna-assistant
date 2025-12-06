//! Ticket command handlers for annactl (v0.0.113).
//!
//! Commands for async ticket workflow:
//! - annactl reply <case> <message> - Reply to an open ticket
//! - annactl ticket <case> - Show ticket conversation
//! - annactl email <address> - Configure email notifications

use anna_shared::email::EmailConfig;
use anna_shared::ticket_tracker::{TicketTracker, TicketStatus};
use anna_shared::ui::colors;
use anyhow::Result;

/// Handle reply command - add user reply to a ticket
pub async fn handle_reply(case: &str, message: &str) -> Result<()> {
    let tracker = TicketTracker::for_user();

    // Find the ticket
    let ticket = match tracker.find_by_case(case)? {
        Some(t) => t,
        None => {
            println!("{}Error:{} Ticket {} not found.", colors::ERR, colors::RESET, case);
            println!();
            println!("To see open tickets, ask Anna: \"show my tickets\"");
            return Ok(());
        }
    };

    // Check if ticket is open
    if !ticket.is_open() {
        println!("{}Note:{} Ticket {} is already {}.", colors::WARN, colors::RESET, case, ticket.status);
        println!();
        println!("To ask a new question, just talk to Anna directly.");
        return Ok(());
    }

    // Add user reply
    let mut updated = ticket;
    updated.add_user_reply(message.to_string());

    // Save updated ticket
    tracker.update_ticket(&updated)?;

    println!();
    println!("{}Reply added to {}{}",colors::OK, case, colors::RESET);
    println!();
    println!("Your message: {}", message);
    println!();
    println!("The IT team will review and respond. You'll be notified by email if configured.");
    println!();

    // Show how to check status
    println!("{}To check status:{} annactl ticket {}", colors::DIM, colors::RESET, case);

    Ok(())
}

/// Handle ticket command - show ticket conversation
pub async fn handle_ticket(case: &str) -> Result<()> {
    let tracker = TicketTracker::for_user();

    // Find the ticket
    let ticket = match tracker.find_by_case(case)? {
        Some(t) => t,
        None => {
            println!("{}Error:{} Ticket {} not found.", colors::ERR, colors::RESET, case);
            println!();
            // Show recent tickets
            let recent = tracker.recent(5)?;
            if !recent.is_empty() {
                println!("Recent tickets:");
                for t in recent {
                    let status_color = match t.status {
                        TicketStatus::Resolved => colors::OK,
                        TicketStatus::PendingUser => colors::WARN,
                        _ => colors::DIM,
                    };
                    println!("  {} [{}{}{}] {}",
                        t.case_number, status_color, t.status, colors::RESET,
                        truncate(&t.query, 40));
                }
            }
            return Ok(());
        }
    };

    // Display ticket header
    println!();
    println!("{}Ticket {}{}", colors::BOLD, ticket.case_number, colors::RESET);
    println!();

    // Status line
    let status_color = match ticket.status {
        TicketStatus::Resolved => colors::OK,
        TicketStatus::PendingUser => colors::WARN,
        TicketStatus::Escalated => colors::CYAN,
        _ => colors::DIM,
    };
    println!("Status: {}{}{}", status_color, ticket.status, colors::RESET);
    println!("Team: {}", ticket.team);

    if let Some(ref assigned) = ticket.assigned_to {
        println!("Assigned to: {}", assigned);
    }

    println!("Created: {}", ticket.created_at.format("%Y-%m-%d %H:%M"));

    if let Some(reliability) = ticket.reliability {
        println!("Reliability: {}%", reliability);
    }

    // Show conversation
    println!();
    println!("{}Conversation:{}", colors::BOLD, colors::RESET);
    println!();

    for msg in &ticket.messages {
        let (prefix, color): (&str, &str) = match msg.sender.as_str() {
            "user" => ("[you]", colors::CYAN),
            "anna" => ("[anna]", colors::OK),
            _ => ("[staff]", colors::DIM),
        };
        println!("{}{}{} {}", color, prefix, colors::RESET, msg.content);
        println!("  {}{}{}",colors::DIM, msg.timestamp.format("%H:%M"), colors::RESET);
        println!();
    }

    // Show pending question if any
    if ticket.status == TicketStatus::PendingUser {
        if let Some(ref question) = ticket.pending_question {
            println!("{}Waiting for your reply:{}", colors::WARN, colors::RESET);
            println!("  {}", question);
            println!();
            println!("{}To reply:{} annactl reply {} \"your answer\"",
                colors::DIM, colors::RESET, ticket.case_number);
        }
    }

    Ok(())
}

/// Handle email command - configure email notifications
pub async fn handle_email(address: &str) -> Result<()> {
    let mut config = EmailConfig::load();

    if address == "off" || address == "disable" || address == "none" {
        config.clear();
        config.save()?;
        println!();
        println!("{}Email notifications disabled.{}", colors::DIM, colors::RESET);
        println!();
        return Ok(());
    }

    // Basic email validation
    if !address.contains('@') || !address.contains('.') {
        println!("{}Error:{} Invalid email address: {}", colors::ERR, colors::RESET, address);
        println!();
        println!("Usage:");
        println!("  annactl email user@example.com  # Set email");
        println!("  annactl email off               # Disable notifications");
        return Ok(());
    }

    config.set_email(address);
    config.save()?;

    println!();
    println!("{}Email configured:{} {}", colors::OK, colors::RESET, address);
    println!();
    println!("You'll receive notifications when:");
    println!("  › A ticket is created");
    println!("  › IT staff needs clarification");
    println!("  › A ticket is resolved");
    println!();
    println!("{}To disable:{} annactl email off", colors::DIM, colors::RESET);

    Ok(())
}

/// Truncate a string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world this is long", 10), "hello w...");
    }
}
