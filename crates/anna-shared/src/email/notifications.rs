//! Email notifications (v0.0.206).

use std::process::Command;

use crate::ticket_tracker::Ticket;

use super::config::EmailConfig;

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
pub fn format_email(notif: &EmailNotification<'_>) -> (String, String) {
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
            let question = ticket
                .pending_question
                .as_deref()
                .unwrap_or("We need more information.");
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
                ticket.case_number, ticket.query, question, ticket.case_number
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
                ticket.case_number, ticket.query, answer, reliability
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
                ticket.case_number, message, ticket.case_number
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
