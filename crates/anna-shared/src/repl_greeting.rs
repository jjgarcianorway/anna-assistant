//! REPL Greeting - Stats-based personalized greetings (v0.0.413).
//!
//! Generates a greeting for the REPL that uses real ticket stats
//! to create a personalized "IT department" welcome.

use crate::ticket_log::{load_recent_tickets, TicketResult};
use crate::ui::colors;
use std::collections::HashMap;

/// REPL greeting data
#[derive(Debug, Clone)]
pub struct ReplGreeting {
    /// User's name (from env or "there")
    pub user_name: String,
    /// Number of tickets handled recently
    pub tickets_handled: usize,
    /// Top topics/domains
    pub top_topics: Vec<String>,
    /// System health status
    pub system_status: SystemStatus,
    /// Status summary line
    pub status_summary: String,
    /// Number of active staff (domains with activity)
    pub active_staff: usize,
    /// Departments with activity
    pub departments: Vec<String>,
    /// Is this the user's first time?
    pub first_time: bool,
}

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemStatus {
    Ok,
    Warn,
    Critical,
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemStatus::Ok => write!(f, "OK"),
            SystemStatus::Warn => write!(f, "WARN"),
            SystemStatus::Critical => write!(f, "CRIT"),
        }
    }
}

impl SystemStatus {
    pub fn color(&self) -> &'static str {
        match self {
            SystemStatus::Ok => colors::OK,
            SystemStatus::Warn => colors::WARN,
            SystemStatus::Critical => colors::ERR,
        }
    }
}

impl ReplGreeting {
    /// Build greeting from recent ticket data
    pub fn build() -> Self {
        let user_name = get_user_name();
        let tickets = load_recent_tickets(100);

        if tickets.is_empty() {
            return Self::first_time(&user_name);
        }

        // Count domains
        let mut domain_counts: HashMap<String, usize> = HashMap::new();
        let mut success_count = 0;
        let mut total_count = 0;

        for ticket in &tickets {
            *domain_counts.entry(ticket.domain.clone()).or_default() += 1;
            total_count += 1;
            if ticket.result == TicketResult::Success {
                success_count += 1;
            }
        }

        // Top topics (domains sorted by frequency)
        let mut sorted_domains: Vec<_> = domain_counts.iter().collect();
        sorted_domains.sort_by(|a, b| b.1.cmp(a.1));
        let top_topics: Vec<String> = sorted_domains
            .iter()
            .take(3)
            .map(|(d, _)| capitalize_domain(d))
            .collect();

        // Departments with activity
        let departments: Vec<String> = sorted_domains
            .iter()
            .take(5)
            .map(|(d, _)| d.to_string())
            .collect();

        // System status based on recent success rate
        let success_rate = if total_count > 0 {
            success_count as f32 / total_count as f32
        } else {
            1.0
        };

        let (system_status, status_summary) = if success_rate >= 0.9 {
            (SystemStatus::Ok, "All systems nominal".to_string())
        } else if success_rate >= 0.7 {
            (
                SystemStatus::Warn,
                format!("Some issues detected ({:.0}% success rate)", success_rate * 100.0),
            )
        } else {
            (
                SystemStatus::Critical,
                format!("Multiple failures ({:.0}% success rate)", success_rate * 100.0),
            )
        };

        Self {
            user_name,
            tickets_handled: total_count,
            top_topics,
            system_status,
            status_summary,
            active_staff: departments.len(),
            departments,
            first_time: false,
        }
    }

    /// First time greeting (no ticket history)
    fn first_time(user_name: &str) -> Self {
        Self {
            user_name: user_name.to_string(),
            tickets_handled: 0,
            top_topics: vec![],
            system_status: SystemStatus::Ok,
            status_summary: "Ready to help".to_string(),
            active_staff: 6,
            departments: vec!["System", "Network", "Storage", "Desktop", "Security", "Packages"]
                .into_iter()
                .map(String::from)
                .collect(),
            first_time: true,
        }
    }

    /// Render the greeting for display
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "\n{}Hello {},{}\n\n",
            colors::CYAN, self.user_name, colors::RESET
        ));

        if self.first_time {
            output.push_str("First time here? I'm Anna, your local IT department.\n");
            output.push_str("Ask me anything about your system - disk, memory, services, config.\n\n");
        } else {
            // Last session stats
            output.push_str(&format!(
                "Last session: {} tickets handled",
                self.tickets_handled
            ));
            if !self.top_topics.is_empty() {
                output.push_str(&format!(", top topics: {}", self.top_topics.join(", ")));
            }
            output.push_str("\n");

            // System status
            output.push_str(&format!(
                "System status: {}{}{} - {}\n",
                self.system_status.color(),
                self.system_status,
                colors::RESET,
                self.status_summary
            ));
        }

        // Active staff
        output.push_str(&format!(
            "Active staff: {} across {} departments\n",
            self.active_staff,
            self.departments.len()
        ));

        // Prompt
        output.push_str(&format!(
            "\n{}What can I help with?{}\n",
            colors::DIM, colors::RESET
        ));

        output
    }

    /// Render compact greeting (one line)
    pub fn render_compact(&self) -> String {
        if self.first_time {
            format!(
                "{}Anna{} - Your local IT department. Ask anything!",
                colors::CYAN, colors::RESET
            )
        } else {
            format!(
                "{}Anna{} - {} tickets, status: {}{}{}",
                colors::CYAN,
                colors::RESET,
                self.tickets_handled,
                self.system_status.color(),
                self.system_status,
                colors::RESET
            )
        }
    }
}

/// Get user's name from environment
fn get_user_name() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "there".to_string())
}

/// Capitalize domain name for display
fn capitalize_domain(domain: &str) -> String {
    let mut chars = domain.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Session context for REPL memory
#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    /// Questions asked in this session
    pub questions: Vec<String>,
    /// Last domain context
    pub last_domain: Option<String>,
    /// Last ticket ID
    pub last_ticket_id: Option<String>,
    /// Session start time
    pub started_at: u64,
}

impl SessionContext {
    pub fn new() -> Self {
        Self {
            started_at: now_secs(),
            ..Default::default()
        }
    }

    /// Record a question
    pub fn add_question(&mut self, question: &str, domain: Option<&str>, ticket_id: Option<&str>) {
        self.questions.push(question.to_string());
        if self.questions.len() > 10 {
            self.questions.remove(0); // Keep last 10
        }
        if let Some(d) = domain {
            self.last_domain = Some(d.to_string());
        }
        if let Some(t) = ticket_id {
            self.last_ticket_id = Some(t.to_string());
        }
    }

    /// Get session duration
    pub fn duration_secs(&self) -> u64 {
        now_secs().saturating_sub(self.started_at)
    }

    /// Context hint for internal comms
    pub fn context_hint(&self) -> Option<String> {
        if let Some(ref domain) = self.last_domain {
            Some(format!("Continuing from {} context", domain))
        } else {
            None
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_time_greeting() {
        let greeting = ReplGreeting::first_time("test_user");
        assert!(greeting.first_time);
        assert_eq!(greeting.user_name, "test_user");
        let rendered = greeting.render();
        assert!(rendered.contains("First time here?"));
    }

    #[test]
    fn test_session_context() {
        let mut ctx = SessionContext::new();
        ctx.add_question("what's my disk usage?", Some("storage"), Some("STG-001"));

        assert_eq!(ctx.questions.len(), 1);
        assert_eq!(ctx.last_domain, Some("storage".to_string()));
        assert!(ctx.context_hint().unwrap().contains("storage"));
    }
}
