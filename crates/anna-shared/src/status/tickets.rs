//! Ticket tracking and specialist statistics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{SpecialistStatus, TicketStatus};

/// v0.3.3: Per-specialist statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistStats {
    /// Specialist name (e.g., "Marcus", "Elena")
    pub name: String,
    /// Department (e.g., "System Administration", "Network Operations")
    pub department: String,
    /// Whether this is a senior specialist
    pub is_senior: bool,
    /// Total tickets handled
    pub tickets_handled: u64,
    /// Successfully resolved tickets
    pub tickets_resolved: u64,
    /// Tickets escalated to senior
    pub tickets_escalated: u64,
    /// Average resolution time in milliseconds
    pub avg_resolution_ms: u64,
    /// Topics this specialist excels at
    pub top_topics: Vec<String>,
    /// Current status (available, busy, offline)
    pub current_status: SpecialistStatus,
}

/// v0.3.3: Ticket tracking for numbered tickets
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketTracker {
    /// Next ticket number to assign
    pub next_number: u64,
    /// Tickets created today
    pub today_count: u64,
    /// Current date (DDMMYYYY format)
    pub current_date: String,
    /// Active tickets (not yet resolved)
    pub active_tickets: Vec<ActiveTicket>,
    /// Statistics by department
    pub dept_stats: HashMap<String, DepartmentTicketStats>,
}

impl TicketTracker {
    /// Generate next ticket ID in format CN-XXXX-DDMMYYYY
    pub fn next_ticket_id(&mut self) -> String {
        let today = chrono::Local::now().format("%d%m%Y").to_string();

        // Reset counter if new day
        if self.current_date != today {
            self.current_date = today.clone();
            self.today_count = 0;
        }

        self.today_count += 1;
        self.next_number += 1;

        format!("CN-{:04}-{}", self.today_count, today)
    }
}

/// v0.3.3: Active ticket info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTicket {
    /// Ticket ID (CN-XXXX-DDMMYYYY)
    pub id: String,
    /// Short summary
    pub summary: String,
    /// Assigned specialist
    pub assigned_to: Option<String>,
    /// Department handling this
    pub department: String,
    /// Created timestamp
    pub created_at: String,
    /// Current status
    pub status: TicketStatus,
}

/// v0.3.3: Department-level ticket statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepartmentTicketStats {
    /// Total tickets received
    pub total_received: u64,
    /// Successfully resolved
    pub resolved: u64,
    /// Average resolution time in milliseconds
    pub avg_resolution_ms: u64,
    /// Escalations to other departments
    pub escalations_out: u64,
    /// Escalations received from other departments
    pub escalations_in: u64,
}

/// v0.3.3: Team roster for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamRoster {
    /// All specialists by department
    pub specialists: HashMap<String, Vec<SpecialistStats>>,
    /// Total team size
    pub total_specialists: usize,
    /// Currently available specialists
    pub available_count: usize,
}
