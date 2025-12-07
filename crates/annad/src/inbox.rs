//! Inbox file watcher for async queries (v0.0.115).
//!
//! Users can submit async queries by writing to ~/.anna/inbox:
//!   echo "? how much disk space do I have" >> ~/.anna/inbox
//!
//! Anna will process these queries and create tickets, then
//! notify the user via email if configured.
//!
//! Format:
//!   - Lines starting with "?" are new queries
//!   - Lines starting with "#" are comments (ignored)
//!   - Empty lines are ignored
//!   - Processed queries are moved to ~/.anna/inbox.done

use anna_shared::email::{inbox_path, send_notification, EmailConfig, EmailNotification};
use anna_shared::ticket_tracker::{Ticket, TicketStatus, TicketTracker};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Result of processing inbox
pub struct InboxResult {
    /// Number of queries processed
    pub processed: usize,
    /// Number of tickets created
    pub tickets_created: usize,
    /// Any errors encountered
    pub errors: Vec<String>,
}

/// Process any pending queries in the inbox file
pub fn process_inbox() -> InboxResult {
    let inbox = inbox_path();
    let mut result = InboxResult {
        processed: 0,
        tickets_created: 0,
        errors: Vec::new(),
    };

    // Check if inbox exists
    if !inbox.exists() {
        debug!("Inbox file does not exist: {:?}", inbox);
        return result;
    }

    // Read inbox contents
    let content = match fs::read_to_string(&inbox) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(format!("Failed to read inbox: {}", e));
            return result;
        }
    };

    // Parse queries from inbox
    let queries: Vec<&str> = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    if queries.is_empty() {
        return result;
    }

    info!("Processing {} queries from inbox", queries.len());

    let tracker = TicketTracker::for_user();
    let config = EmailConfig::load();
    let mut processed_queries = Vec::new();

    for query_line in queries {
        result.processed += 1;

        // Clean up query (remove leading "?" if present)
        let query = query_line
            .trim()
            .strip_prefix('?')
            .map(|s| s.trim())
            .unwrap_or(query_line.trim());

        if query.is_empty() {
            continue;
        }

        // Create a ticket for this query
        let case_number = generate_case_number();
        let mut ticket = Ticket::new(
            case_number.clone(),
            query.to_string(),
            "Support".to_string(), // Default team
        );

        // Mark as async ticket
        ticket.is_async = true;
        if let Some(ref email) = config.user_email {
            ticket.user_email = Some(email.clone());
        }

        // Save ticket
        match tracker.save_ticket(&ticket) {
            Ok(_) => {
                result.tickets_created += 1;
                info!("Created ticket {} from inbox: {}", case_number, query);

                // Send email notification
                let _ = send_notification(EmailNotification::TicketCreated(&ticket));

                processed_queries.push(query_line.to_string());
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to create ticket: {}", e));
            }
        }
    }

    // Move processed queries to done file
    if !processed_queries.is_empty() {
        let done_path = done_path();
        if let Err(e) = append_to_done(&done_path, &processed_queries) {
            warn!("Failed to write to done file: {}", e);
        }

        // Clear inbox (keep comments and blank lines)
        let remaining: String = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.is_empty()
                    || trimmed.starts_with('#')
                    || !processed_queries.contains(&line.to_string())
            })
            .collect::<Vec<_>>()
            .join("\n");

        if let Err(e) = fs::write(&inbox, remaining.trim_end().to_string() + "\n") {
            result.errors.push(format!("Failed to update inbox: {}", e));
        }
    }

    result
}

/// Generate a new case number
fn generate_case_number() -> String {
    use chrono::Utc;
    let now = Utc::now();
    let seq = now.timestamp() % 10000;
    format!("CN-{:04}-{}", seq, now.format("%d%m%Y"))
}

/// Path to done file
fn done_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".anna").join("inbox.done")
}

/// Append processed queries to done file
fn append_to_done(path: &PathBuf, queries: &[String]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    for query in queries {
        writeln!(file, "[{}] {}", timestamp, query)?;
    }

    Ok(())
}

/// Check pending tickets for updates (called periodically)
pub fn check_pending_tickets() -> usize {
    let tracker = TicketTracker::for_user();
    let mut updated = 0;

    // Get pending user tickets (where we asked a question)
    if let Ok(pending) = tracker.pending_user() {
        for ticket in pending {
            // Check if user replied via annactl
            if ticket.user_reply.is_some() {
                // Update ticket status
                let mut updated_ticket = ticket.clone();
                updated_ticket.status = TicketStatus::InProgress;

                if let Ok(()) = tracker.update_ticket(&updated_ticket) {
                    updated += 1;
                    info!("Ticket {} received user reply", ticket.case_number);
                }
            }
        }
    }

    updated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_case_number() {
        let cn = generate_case_number();
        assert!(cn.starts_with("CN-"));
        assert!(cn.contains('-'));
    }

    #[test]
    fn test_done_path() {
        let path = done_path();
        assert!(path.to_string_lossy().contains(".anna"));
        assert!(path.to_string_lossy().contains("inbox.done"));
    }
}
