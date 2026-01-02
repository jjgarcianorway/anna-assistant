//! Building logic for REPL greetings.

use crate::ticket_log::{load_recent_tickets, TicketResult};
use std::collections::HashMap;

use super::helpers::{capitalize_domain, get_user_name};
use super::types::{ReplGreeting, SystemError, SystemStatus};

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
                format!(
                    "Some issues detected ({:.0}% success rate)",
                    success_rate * 100.0
                ),
            )
        } else {
            (
                SystemStatus::Critical,
                format!(
                    "Multiple failures ({:.0}% success rate)",
                    success_rate * 100.0
                ),
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
            errors: Vec::new(),
        }
    }

    /// First time greeting (no ticket history)
    pub(super) fn first_time(user_name: &str) -> Self {
        Self {
            user_name: user_name.to_string(),
            tickets_handled: 0,
            top_topics: vec![],
            system_status: SystemStatus::Ok,
            status_summary: "Ready to help".to_string(),
            active_staff: 6,
            departments: vec![
                "System", "Network", "Storage", "Desktop", "Security", "Packages",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            first_time: true,
            errors: Vec::new(),
        }
    }

    /// Add an error to the greeting
    pub fn add_error(&mut self, category: &str, message: &str, fix_hint: Option<&str>) {
        self.errors.push(SystemError {
            category: category.to_string(),
            message: message.to_string(),
            fix_hint: fix_hint.map(String::from),
        });
        // Update status if we have errors
        if self.system_status == SystemStatus::Ok {
            self.system_status = SystemStatus::Warn;
            self.status_summary = "Issues detected".to_string();
        }
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}
