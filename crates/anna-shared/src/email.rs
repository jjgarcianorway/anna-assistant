//! Email notification system for Service Desk Theatre (v0.0.113).
//!
//! Sends email notifications to users when:
//! - A new async ticket is created
//! - IT staff needs clarification (ticket pending user)
//! - Ticket is resolved with answer
//!
//! Uses the system's `sendmail` or `mail` command.
//! User can configure their email with `annactl config email user@example.com`

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::ticket_tracker::Ticket;

/// Email configuration
#[derive(Debug, Clone, Default)]
pub struct EmailConfig {
    /// User's email address
    pub user_email: Option<String>,
    /// Send email notifications
    pub enabled: bool,
}

impl EmailConfig {
    /// Load email config from disk
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => {
                let email = content.trim().to_string();
                if email.is_empty() || !email.contains('@') {
                    Self::default()
                } else {
                    Self {
                        user_email: Some(email),
                        enabled: true,
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }

    /// Save email config
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(ref email) = self.user_email {
            fs::write(&path, email)?;
        }
        Ok(())
    }

    /// Config file path
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".anna").join("email.conf")
    }

    /// Set email address
    pub fn set_email(&mut self, email: &str) {
        self.user_email = Some(email.to_string());
        self.enabled = true;
    }

    /// Clear email (disable notifications)
    pub fn clear(&mut self) {
        self.user_email = None;
        self.enabled = false;
    }
}

/// Email notification types
pub enum EmailNotification<'a> {
    /// New async ticket created
    TicketCreated(&'a Ticket),
    /// IT needs clarification
    NeedsClarification(&'a Ticket),
    /// Ticket resolved
    TicketResolved(&'a Ticket),
    /// General update
    TicketUpdate(&'a Ticket, &'a str),
}

/// Send an email notification
pub fn send_notification(notif: EmailNotification<'_>) -> Result<(), String> {
    let config = EmailConfig::load();

    // Get email from config or ticket
    let email = match &notif {
        EmailNotification::TicketCreated(t)
        | EmailNotification::NeedsClarification(t)
        | EmailNotification::TicketResolved(t)
        | EmailNotification::TicketUpdate(t, _) => {
            t.user_email.as_deref().or(config.user_email.as_deref())
        }
    };

    let email = match email {
        Some(e) => e,
        None => return Ok(()), // No email configured, skip silently
    };

    let (subject, body) = format_email(&notif);

    send_email(email, &subject, &body)
}

/// Format email subject and body
fn format_email(notif: &EmailNotification<'_>) -> (String, String) {
    match notif {
        EmailNotification::TicketCreated(ticket) => {
            let subject = format!("[Anna] Ticket {} created", ticket.case_number);
            let body = format!(
                r#"Hi there,

Your support request has been received and assigned case number {}.

Question: {}

Team: {} Team
Status: {}

We're looking into this and will get back to you soon.

To check status: annactl ticket {}
To reply: annactl reply {} "your message"

--
Anna Service Desk
Your local IT department
"#,
                ticket.case_number,
                ticket.query,
                ticket.team,
                ticket.status,
                ticket.case_number,
                ticket.case_number
            );
            (subject, body)
        }

        EmailNotification::NeedsClarification(ticket) => {
            let question = ticket.pending_question.as_deref().unwrap_or("We need more information.");
            let subject = format!("[Anna] {} - Clarification needed", ticket.case_number);
            let body = format!(
                r#"Hi there,

Regarding your ticket {}:

{}

Our question: {}

To reply: annactl reply {} "your answer"
Or simply reply to this email.

--
Anna Service Desk
"#,
                ticket.case_number,
                ticket.query,
                question,
                ticket.case_number
            );
            (subject, body)
        }

        EmailNotification::TicketResolved(ticket) => {
            let answer = ticket.resolution.as_deref().unwrap_or("Resolved.");
            let reliability = ticket.reliability.unwrap_or(0);
            let subject = format!("[Anna] {} - Resolved", ticket.case_number);
            let body = format!(
                r#"Hi there,

Your ticket {} has been resolved!

Original question: {}

Answer:
{}

Reliability: {}%

If you have follow-up questions, just ask Anna directly.

--
Anna Service Desk
"#,
                ticket.case_number,
                ticket.query,
                answer,
                reliability
            );
            (subject, body)
        }

        EmailNotification::TicketUpdate(ticket, message) => {
            let subject = format!("[Anna] {} - Update", ticket.case_number);
            let body = format!(
                r#"Hi there,

Update on ticket {}:

{}

To reply: annactl reply {} "your message"

--
Anna Service Desk
"#,
                ticket.case_number,
                message,
                ticket.case_number
            );
            (subject, body)
        }
    }
}

/// Send email using system mail command
fn send_email(to: &str, subject: &str, body: &str) -> Result<(), String> {
    // Try sendmail first, then mail
    let result = Command::new("mail")
        .arg("-s")
        .arg(subject)
        .arg(to)
        .stdin(std::process::Stdio::piped())
        .spawn();

    match result {
        Ok(mut child) => {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(body.as_bytes());
            }
            let status = child.wait().map_err(|e| e.to_string())?;
            if status.success() {
                Ok(())
            } else {
                Err("mail command failed".to_string())
            }
        }
        Err(_) => {
            // mail not available, try echo | sendmail
            let echo = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "echo 'Subject: {}' | cat - /dev/stdin | sendmail {}",
                    subject, to
                ))
                .stdin(std::process::Stdio::piped())
                .spawn();

            match echo {
                Ok(mut child) => {
                    use std::io::Write;
                    if let Some(ref mut stdin) = child.stdin {
                        let _ = stdin.write_all(body.as_bytes());
                    }
                    let status = child.wait().map_err(|e| e.to_string())?;
                    if status.success() {
                        Ok(())
                    } else {
                        // No mail system available - that's OK
                        Ok(())
                    }
                }
                Err(_) => Ok(()), // No mail system, skip silently
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert!(config.user_email.is_none());
        assert!(!config.enabled);
    }

    #[test]
    fn test_format_ticket_created_email() {
        let ticket = Ticket::new(
            "CN-0001-06122025".to_string(),
            "How much RAM?".to_string(),
            "Hardware".to_string(),
        );
        let (subject, body) = format_email(&EmailNotification::TicketCreated(&ticket));
        assert!(subject.contains("CN-0001"));
        assert!(body.contains("How much RAM"));
        assert!(body.contains("annactl reply"));
    }

    #[test]
    fn test_format_needs_clarification_email() {
        let mut ticket = Ticket::new(
            "CN-0002-06122025".to_string(),
            "Configure vim".to_string(),
            "Desktop".to_string(),
        );
        ticket.pending_question = Some("Which vim feature?".to_string());

        let (subject, body) = format_email(&EmailNotification::NeedsClarification(&ticket));
        assert!(subject.contains("Clarification"));
        assert!(body.contains("Which vim feature"));
    }

    #[test]
    fn test_format_resolved_email() {
        let mut ticket = Ticket::new(
            "CN-0003-06122025".to_string(),
            "Disk space?".to_string(),
            "Storage".to_string(),
        );
        ticket.resolution = Some("50GB free on /home".to_string());
        ticket.reliability = Some(95);

        let (subject, body) = format_email(&EmailNotification::TicketResolved(&ticket));
        assert!(subject.contains("Resolved"));
        assert!(body.contains("50GB free"));
        assert!(body.contains("95%"));
    }
}
