//! Ticket command handlers for annactl (v0.0.344).
//!
//! Commands for async ticket workflow:
//! - annactl reply <case> <message> - Reply to an open ticket
//! - annactl ticket <case> - Show ticket conversation
//! - annactl email <address> - Configure email notifications
//! - annactl health - Check Anna's health and dependencies (v0.0.114)
//! v0.0.340: Use centralized UI helpers for consistency.
//! v0.0.344: Use print_title() for header display.

use anna_shared::email::{EmailConfig, EmailHealth};
use anna_shared::ticket_tracker::{TicketStatus, TicketTracker};
use anna_shared::ui::{colors, kv, print_footer, print_hint, print_label, print_section_header, print_title, symbols};
use anyhow::Result;
use std::io::{self, Write};

/// Handle reply command - add user reply to a ticket
pub async fn handle_reply(case: &str, message: &str) -> Result<()> {
    let tracker = TicketTracker::for_user();

    // Find the ticket
    let ticket = match tracker.find_by_case(case)? {
        Some(t) => t,
        None => {
            print_label("error", &format!("Ticket {} not found", case), colors::ERR);
            print_hint("To see open tickets, ask Anna: \"show my tickets\"");
            return Ok(());
        }
    };

    // Check if ticket is open
    if !ticket.is_open() {
        print_label("note", &format!("Ticket {} is already {}", case, ticket.status), colors::WARN);
        print_hint("To ask a new question, just talk to Anna directly.");
        return Ok(());
    }

    // Add user reply
    let mut updated = ticket;
    updated.add_user_reply(message.to_string());

    // Save updated ticket
    tracker.update_ticket(&updated)?;

    println!();
    print_label("ok", &format!("Reply added to {}", case), colors::OK);
    println!();
    kv("your message", message);
    println!();
    println!("The IT team will review and respond. You'll be notified by email if configured.");
    print_hint(&format!("To check status: annactl ticket {}", case));

    Ok(())
}

/// Handle ticket command - show ticket conversation
pub async fn handle_ticket(case: &str) -> Result<()> {
    let tracker = TicketTracker::for_user();

    // Find the ticket
    let ticket = match tracker.find_by_case(case)? {
        Some(t) => t,
        None => {
            print_label("error", &format!("Ticket {} not found", case), colors::ERR);
            println!();
            // Show recent tickets
            let recent = tracker.recent(5)?;
            if !recent.is_empty() {
                print_section_header("recent tickets");
                for t in recent {
                    let status_color = match t.status {
                        TicketStatus::Resolved => colors::OK,
                        TicketStatus::PendingUser => colors::WARN,
                        _ => colors::DIM,
                    };
                    println!(
                        "  {} {} [{}{}{}]",
                        symbols::ARROW, t.case_number, status_color, t.status, colors::RESET
                    );
                    println!("    {}", t.query);
                }
            }
            return Ok(());
        }
    };

    // Display ticket header
    println!();
    print_title(&format!("Ticket {}", ticket.case_number));
    println!();

    // Status info
    print_section_header("details");
    let status_color = match ticket.status {
        TicketStatus::Resolved => colors::OK,
        TicketStatus::PendingUser => colors::WARN,
        TicketStatus::Escalated => colors::CYAN,
        _ => colors::DIM,
    };
    kv("status", &format!("{}{}{}", status_color, ticket.status, colors::RESET));
    kv("team", &ticket.team);

    if let Some(ref assigned) = ticket.assigned_to {
        kv("assigned_to", assigned);
    }

    kv("created", &ticket.created_at.format("%Y-%m-%d %H:%M").to_string());

    if let Some(reliability) = ticket.reliability {
        kv("reliability", &format!("{}%", reliability));
    }
    println!();

    // Show conversation
    print_section_header("conversation");
    for msg in &ticket.messages {
        let (prefix, color): (&str, &str) = match msg.sender.as_str() {
            "user" => ("you", colors::CYAN),
            "anna" => ("anna", colors::OK),
            _ => ("staff", colors::DIM),
        };
        println!("  {}[{}]{} {}", color, prefix, colors::RESET, msg.content);
        println!("    {}{}{}", colors::DIM, msg.timestamp.format("%H:%M"), colors::RESET);
    }
    println!();

    // Show pending question if any
    if ticket.status == TicketStatus::PendingUser {
        if let Some(ref question) = ticket.pending_question {
            print_label("waiting", "Needs your reply", colors::WARN);
            println!("  {}", question);
            print_hint(&format!("To reply: annactl reply {} \"your answer\"", ticket.case_number));
        }
    }

    print_footer();
    Ok(())
}

/// Handle email command - configure email notifications
pub async fn handle_email(address: &str) -> Result<()> {
    let mut config = EmailConfig::load();

    if address == "off" || address == "disable" || address == "none" {
        config.clear();
        config.save()?;
        println!();
        print_label("email", "Notifications disabled", colors::DIM);
        return Ok(());
    }

    // Basic email validation
    if !address.contains('@') || !address.contains('.') {
        print_label("error", &format!("Invalid email address: {}", address), colors::ERR);
        println!();
        print_section_header("usage");
        println!("  {} annactl email user@example.com  # Set email", symbols::ARROW);
        println!("  {} annactl email off               # Disable", symbols::ARROW);
        return Ok(());
    }

    config.set_email(address);
    config.save()?;

    println!();
    print_label("email", &format!("Configured: {}", address), colors::OK);
    println!();
    print_section_header("notifications");
    println!("  {} A ticket is created", symbols::ARROW);
    println!("  {} IT staff needs clarification", symbols::ARROW);
    println!("  {} A ticket is resolved", symbols::ARROW);
    println!();
    print_hint("To disable: annactl email off");

    Ok(())
}

/// Handle health command - check Anna's dependencies and offer to install
pub async fn handle_health() -> Result<()> {
    println!();
    print_title("Anna Health Check");
    println!();

    // Check email system
    let email_health = EmailHealth::check();

    print_section_header("email system");
    if email_health.can_send {
        println!("  {}{}{} Mail command available", colors::OK, symbols::OK, colors::RESET);
    } else {
        println!("  {}{}{} Mail command not found", colors::WARN, symbols::WARN, colors::RESET);
        kv("package_needed", &email_health.package_name);
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
                    print_label("ok", "Email package installed", colors::OK);
                }
                _ => {
                    print_label("error", "Installation failed", colors::ERR);
                    print_hint(&format!("Try manually: {}", email_health.install_cmd));
                }
            }
        }
    }
    println!();

    // Check user email
    print_section_header("your email");
    if let Some(ref email) = email_health.user_email {
        println!("  {}{}{} Configured: {}", colors::OK, symbols::OK, colors::RESET, email);
    } else {
        println!("  {}{}{} Not configured", colors::WARN, symbols::WARN, colors::RESET);
        print_hint("Run: annactl email your@email.com");
    }
    println!();

    // Show inbox status
    print_section_header("async inbox");
    if email_health.inbox_exists {
        println!("  {}{}{} {}", colors::OK, symbols::OK, colors::RESET, email_health.inbox_path.display());
        if email_health.inbox_count > 0 {
            println!(
                "  {}{}{} {} pending {}",
                colors::WARN, symbols::WARN, colors::RESET,
                email_health.inbox_count,
                if email_health.inbox_count == 1 { "query" } else { "queries" }
            );
        }
    } else {
        println!("  {}{}{} Inbox not created yet", colors::WARN, symbols::WARN, colors::RESET);
        print_hint("To create: echo \"? your question\" >> ~/.anna/inbox");
    }
    println!();

    // Show contact options
    print_section_header("contact anna");
    println!("  {} annactl \"your question\"     (one-shot)", symbols::ARROW);
    println!("  {} annactl                      (interactive)", symbols::ARROW);
    println!("  {} ~/.anna/inbox                (async queries)", symbols::ARROW);
    println!();

    // Open tickets
    let tracker = TicketTracker::for_user();
    if let Ok(open) = tracker.open_tickets() {
        if !open.is_empty() {
            print_section_header("open tickets");
            for t in open.iter().take(5) {
                println!("  {} {} ({})", symbols::ARROW, t.case_number, t.status);
                println!("    {}", t.query);
            }
            if open.len() > 5 {
                println!("  {} and {} more", symbols::ARROW, open.len() - 5);
            }
            println!();
        }
    }

    // Summary
    if email_health.is_ready() {
        print_label("health", "All systems ready", colors::OK);
    } else {
        print_label("health", "Setup needed for full email support", colors::WARN);
    }

    print_footer();
    Ok(())
}
