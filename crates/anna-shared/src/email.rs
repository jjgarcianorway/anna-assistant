//! Email notification system for Service Desk Theatre (v0.0.113).
//!
//! Sends email notifications to users when:
//! - A new async ticket is created
//! - IT staff needs clarification (ticket pending user)
//! - Ticket is resolved with answer
//!
//! Uses the system's `sendmail` or `mail` command.
//! User can configure their email with `annactl config email user@example.com`
//!
//! v0.0.114: Added health check, auto-install, and Anna's email address.
//! v0.0.115: Replaced email inbox with file-based inbox (~/.anna/inbox)

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::ticket_tracker::Ticket;

/// Path to Anna's inbox file for async queries
/// Users can write questions here and Anna will process them
pub fn inbox_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".anna").join("inbox")
}

/// Legacy constant for backwards compatibility
pub const ANNA_EMAIL: &str = "anna@localhost";

/// Check if email system is available
pub fn is_email_available() -> bool {
    // Check for mail command
    Command::new("which")
        .arg("mail")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the package name for email support on this distro
pub fn email_package_name() -> &'static str {
    // Check for Arch
    if PathBuf::from("/etc/arch-release").exists() {
        return "s-nail"; // Arch Linux
    }
    // Check for Debian/Ubuntu
    if PathBuf::from("/etc/debian_version").exists() {
        return "mailutils";
    }
    // Check for Fedora/RHEL
    if PathBuf::from("/etc/fedora-release").exists()
        || PathBuf::from("/etc/redhat-release").exists()
    {
        return "mailx";
    }
    // Default
    "mailutils"
}

/// Install email package (returns command to run)
pub fn install_email_command() -> String {
    let pkg = email_package_name();

    // Detect package manager
    if PathBuf::from("/usr/bin/pacman").exists() {
        format!("sudo pacman -S --noconfirm {}", pkg)
    } else if PathBuf::from("/usr/bin/apt").exists() {
        format!("sudo apt install -y {}", pkg)
    } else if PathBuf::from("/usr/bin/dnf").exists() {
        format!("sudo dnf install -y {}", pkg)
    } else if PathBuf::from("/usr/bin/yum").exists() {
        format!("sudo yum install -y {}", pkg)
    } else {
        format!("# Install {} using your package manager", pkg)
    }
}

/// Email health status
#[derive(Debug, Clone)]
pub struct EmailHealth {
    /// Is email sending available?
    pub can_send: bool,
    /// Package name needed
    pub package_name: &'static str,
    /// Install command
    pub install_cmd: String,
    /// User's configured email
    pub user_email: Option<String>,
    /// Inbox path for async queries
    pub inbox_path: PathBuf,
    /// Does inbox exist?
    pub inbox_exists: bool,
    /// Pending queries in inbox
    pub inbox_count: usize,
}

impl EmailHealth {
    /// Check email system health
    pub fn check() -> Self {
        let config = EmailConfig::load();
        let inbox = inbox_path();
        let inbox_exists = inbox.exists();
        let inbox_count = if inbox_exists {
            count_inbox_queries(&inbox)
        } else {
            0
        };
        Self {
            can_send: is_email_available(),
            package_name: email_package_name(),
            install_cmd: install_email_command(),
            user_email: config.user_email,
            inbox_path: inbox,
            inbox_exists,
            inbox_count,
        }
    }

    /// Is everything ready for email notifications?
    pub fn is_ready(&self) -> bool {
        self.can_send && self.user_email.is_some()
    }
}

/// Count queries in inbox file (lines starting with "?")
fn count_inbox_queries(path: &PathBuf) -> usize {
    fs::read_to_string(path)
        .map(|content| {
            content
                .lines()
                .filter(|line| {
                    line.starts_with('?') || (!line.trim().is_empty() && !line.starts_with('#'))
                })
                .count()
        })
        .unwrap_or(0)
}

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
