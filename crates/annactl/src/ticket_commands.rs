//! Ticket command handlers for annactl (v0.0.113).
//!
//! Commands for async ticket workflow:
//! - annactl reply <case> <message> - Reply to an open ticket
//! - annactl ticket <case> - Show ticket conversation
//! - annactl email <address> - Configure email notifications
//! - annactl health - Check Anna's health and dependencies (v0.0.114)

use anna_shared::email::{EmailConfig, EmailHealth};
use anna_shared::ticket_tracker::{TicketStatus, TicketTracker};
use anna_shared::ui::colors;
use anyhow::Result;
use std::io::{self, Write};

/// Handle reply command - add user reply to a ticket
pub async fn handle_reply(case: &str, message: &str) -> Result<()> {
    let tracker = TicketTracker::for_user();

    // Find the ticket
    let ticket = match tracker.find_by_case(case)? {
        Some(t) => t,
        None => {
            println!(
                "{}Error:{} Ticket {} not found.",
                colors::ERR,
                colors::RESET,
                case
            );
            println!();
            println!("To see open tickets, ask Anna: \"show my tickets\"");
            return Ok(());
        }
    };

    // Check if ticket is open
    if !ticket.is_open() {
        println!(
            "{}Note:{} Ticket {} is already {}.",
            colors::WARN,
            colors::RESET,
            case,
            ticket.status
        );
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
    println!("{}Reply added to {}{}", colors::OK, case, colors::RESET);
    println!();
    println!("Your message: {}", message);
    println!();
    println!("The IT team will review and respond. You'll be notified by email if configured.");
    println!();

    // Show how to check status
    println!(
        "{}To check status:{} annactl ticket {}",
        colors::DIM,
        colors::RESET,
        case
    );

    Ok(())
}

/// Handle ticket command - show ticket conversation
pub async fn handle_ticket(case: &str) -> Result<()> {
    let tracker = TicketTracker::for_user();

    // Find the ticket
    let ticket = match tracker.find_by_case(case)? {
        Some(t) => t,
        None => {
            println!(
                "{}Error:{} Ticket {} not found.",
                colors::ERR,
                colors::RESET,
                case
            );
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
                    println!(
                        "  {} [{}{}{}]",
                        t.case_number,
                        status_color,
                        t.status,
                        colors::RESET
                    );
                    println!("    {}", t.query);
                }
            }
            return Ok(());
        }
    };

    // Display ticket header
    println!();
    println!(
        "{}Ticket {}{}",
        colors::BOLD,
        ticket.case_number,
        colors::RESET
    );
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
        println!(
            "  {}{}{}",
            colors::DIM,
            msg.timestamp.format("%H:%M"),
            colors::RESET
        );
        println!();
    }

    // Show pending question if any
    if ticket.status == TicketStatus::PendingUser {
        if let Some(ref question) = ticket.pending_question {
            println!("{}Waiting for your reply:{}", colors::WARN, colors::RESET);
            println!("  {}", question);
            println!();
            println!(
                "{}To reply:{} annactl reply {} \"your answer\"",
                colors::DIM,
                colors::RESET,
                ticket.case_number
            );
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
        println!(
            "{}Email notifications disabled.{}",
            colors::DIM,
            colors::RESET
        );
        println!();
        return Ok(());
    }

    // Basic email validation
    if !address.contains('@') || !address.contains('.') {
        println!(
            "{}Error:{} Invalid email address: {}",
            colors::ERR,
            colors::RESET,
            address
        );
        println!();
        println!("Usage:");
        println!("  annactl email user@example.com  # Set email");
        println!("  annactl email off               # Disable notifications");
        return Ok(());
    }

    config.set_email(address);
    config.save()?;

    println!();
    println!(
        "{}Email configured:{} {}",
        colors::OK,
        colors::RESET,
        address
    );
    println!();
    println!("You'll receive notifications when:");
    println!("  › A ticket is created");
    println!("  › IT staff needs clarification");
    println!("  › A ticket is resolved");
    println!();
    println!(
        "{}To disable:{} annactl email off",
        colors::DIM,
        colors::RESET
    );

    Ok(())
}

/// Handle health command - check Anna's dependencies and offer to install
pub async fn handle_health() -> Result<()> {
    println!();
    println!("{}Anna Health Check{}", colors::BOLD, colors::RESET);
    println!();

    // Check email system
    let email_health = EmailHealth::check();

    println!("{}Email System:{}", colors::BOLD, colors::RESET);
    if email_health.can_send {
        println!("  {} Mail command available", ok_symbol());
    } else {
        println!("  {} Mail command not found", warn_symbol());
        println!("    Package needed: {}", email_health.package_name);
        println!();
        print!("  Install now? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("y") {
            println!();
            println!("  Running: {}", email_health.install_cmd);
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(&email_health.install_cmd)
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("  {} Email package installed!", ok_symbol());
                }
                _ => {
                    println!("  {} Installation failed. Try manually:", warn_symbol());
                    println!("    {}", email_health.install_cmd);
                }
            }
        }
    }

    // Check user email
    println!();
    println!("{}Your Email:{}", colors::BOLD, colors::RESET);
    if let Some(ref email) = email_health.user_email {
        println!("  {} Configured: {}", ok_symbol(), email);
    } else {
        println!("  {} Not configured", warn_symbol());
        println!("    Run: annactl email your@email.com");
    }

    // Show inbox status
    println!();
    println!("{}Async Inbox:{}", colors::BOLD, colors::RESET);
    if email_health.inbox_exists {
        println!(
            "  {} Inbox: {}",
            ok_symbol(),
            email_health.inbox_path.display()
        );
        if email_health.inbox_count > 0 {
            println!(
                "  {} {} pending {}",
                warn_symbol(),
                email_health.inbox_count,
                if email_health.inbox_count == 1 {
                    "query"
                } else {
                    "queries"
                }
            );
        }
    } else {
        println!("  {} Inbox not created yet", warn_symbol());
        println!("    To create: echo \"? your question\" >> ~/.anna/inbox");
    }

    // Show contact options
    println!();
    println!("{}Contact Anna:{}", colors::BOLD, colors::RESET);
    println!("  {} annactl \"your question\"     (one-shot)", bullet());
    println!("  {} annactl                      (interactive)", bullet());
    println!(
        "  {} ~/.anna/inbox                (async queries)",
        bullet()
    );

    // Open tickets
    println!();
    let tracker = TicketTracker::for_user();
    if let Ok(open) = tracker.open_tickets() {
        if !open.is_empty() {
            println!("{}Open Tickets:{}", colors::BOLD, colors::RESET);
            for t in open.iter().take(5) {
                println!("  {} {} ({})", bullet(), t.case_number, t.status);
                println!("    {}", t.query);
            }
            if open.len() > 5 {
                println!("  {} and {} more", bullet(), open.len() - 5);
            }
        }
    }

    // Summary
    println!();
    if email_health.is_ready() {
        println!("{}All systems ready!{}", colors::OK, colors::RESET);
    } else {
        println!(
            "{}Setup needed for full email support.{}",
            colors::WARN,
            colors::RESET
        );
    }

    Ok(())
}

fn ok_symbol() -> &'static str {
    "✓"
}

fn warn_symbol() -> &'static str {
    "!"
}

fn bullet() -> &'static str {
    "›"
}
