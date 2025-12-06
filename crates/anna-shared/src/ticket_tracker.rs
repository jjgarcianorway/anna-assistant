//! Ticket tracking system for Service Desk Theatre.
//!
//! v0.0.105: Case numbers, ticket lifecycle, and history.
//!
//! Ticket format: CN-XXXX-DDMMYYYY
//! - CN: Case Number prefix
//! - XXXX: Sequential number (resets daily)
//! - DDMMYYYY: Date created

use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Ticket status in the Service Desk workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    /// Just created, awaiting triage
    New,
    /// Assigned to a team member
    Assigned,
    /// Being actively worked on
    InProgress,
    /// Waiting for user input
    PendingUser,
    /// Escalated to senior
    Escalated,
    /// Successfully resolved
    Resolved,
    /// Closed without resolution
    Closed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::Assigned => write!(f, "Assigned"),
            Self::InProgress => write!(f, "In Progress"),
            Self::PendingUser => write!(f, "Pending User"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Closed => write!(f, "Closed"),
        }
    }
}

/// A single ticket in the Service Desk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    /// Case number (e.g., "CN-0001-06122025")
    pub case_number: String,
    /// Original user query
    pub query: String,
    /// Current status
    pub status: TicketStatus,
    /// Team handling this ticket
    pub team: String,
    /// Person ID currently assigned (from roster)
    pub assigned_to: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Resolution time in milliseconds (if resolved)
    pub resolution_ms: Option<u64>,
    /// Reliability score (if resolved)
    pub reliability: Option<u8>,
    /// Was escalated to senior?
    pub was_escalated: bool,
    /// Number of interactions/rounds
    pub interaction_count: u32,
    /// Final answer (if resolved)
    pub resolution: Option<String>,
}

impl Ticket {
    /// Create a new ticket
    pub fn new(case_number: String, query: String, team: String) -> Self {
        let now = Utc::now();
        Self {
            case_number,
            query,
            status: TicketStatus::New,
            team,
            assigned_to: None,
            created_at: now,
            updated_at: now,
            resolution_ms: None,
            reliability: None,
            was_escalated: false,
            interaction_count: 0,
            resolution: None,
        }
    }

    /// Assign to a person
    pub fn assign(&mut self, person_id: &str) {
        self.assigned_to = Some(person_id.to_string());
        self.status = TicketStatus::Assigned;
        self.updated_at = Utc::now();
    }

    /// Start working on ticket
    pub fn start_work(&mut self) {
        self.status = TicketStatus::InProgress;
        self.interaction_count += 1;
        self.updated_at = Utc::now();
    }

    /// Escalate to senior
    pub fn escalate(&mut self, senior_id: &str) {
        self.assigned_to = Some(senior_id.to_string());
        self.status = TicketStatus::Escalated;
        self.was_escalated = true;
        self.interaction_count += 1;
        self.updated_at = Utc::now();
    }

    /// Resolve the ticket
    pub fn resolve(&mut self, answer: String, reliability: u8, duration_ms: u64) {
        self.status = TicketStatus::Resolved;
        self.resolution = Some(answer);
        self.reliability = Some(reliability);
        self.resolution_ms = Some(duration_ms);
        self.updated_at = Utc::now();
    }

    /// Check if ticket is still open
    pub fn is_open(&self) -> bool {
        !matches!(self.status, TicketStatus::Resolved | TicketStatus::Closed)
    }
}

/// Ticket tracker - generates case numbers and tracks history
pub struct TicketTracker {
    /// Path to ticket history file
    history_path: PathBuf,
    /// Path to counter file (for daily sequence)
    counter_path: PathBuf,
}

impl TicketTracker {
    /// Create tracker at default location
    pub fn new() -> Self {
        let base = PathBuf::from("/var/lib/anna");
        Self {
            history_path: base.join("tickets.jsonl"),
            counter_path: base.join("ticket_counter.json"),
        }
    }

    /// Create tracker for user-specific location
    pub fn for_user() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let base = PathBuf::from(home).join(".anna");
        Self {
            history_path: base.join("tickets.jsonl"),
            counter_path: base.join("ticket_counter.json"),
        }
    }

    /// Generate next case number
    pub fn next_case_number(&self) -> String {
        let now = Utc::now();
        let date_str = format!("{:02}{:02}{}", now.day(), now.month(), now.year());

        // Read counter
        let (last_date, last_seq) = self.read_counter();

        // Reset sequence if new day
        let seq = if last_date == date_str {
            last_seq + 1
        } else {
            1
        };

        // Write new counter
        self.write_counter(&date_str, seq);

        format!("CN-{:04}-{}", seq, date_str)
    }

    fn read_counter(&self) -> (String, u32) {
        if !self.counter_path.exists() {
            return (String::new(), 0);
        }

        #[derive(Deserialize)]
        struct Counter {
            date: String,
            seq: u32,
        }

        match fs::read_to_string(&self.counter_path) {
            Ok(json) => match serde_json::from_str::<Counter>(&json) {
                Ok(c) => (c.date, c.seq),
                Err(_) => (String::new(), 0),
            },
            Err(_) => (String::new(), 0),
        }
    }

    fn write_counter(&self, date: &str, seq: u32) {
        #[derive(Serialize)]
        struct Counter<'a> {
            date: &'a str,
            seq: u32,
        }

        if let Some(parent) = self.counter_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json = serde_json::to_string(&Counter { date, seq }).unwrap_or_default();
        let _ = fs::write(&self.counter_path, json);
    }

    /// Save a ticket to history
    pub fn save_ticket(&self, ticket: &Ticket) -> std::io::Result<()> {
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.history_path)?;

        let line = serde_json::to_string(ticket)?;
        writeln!(file, "{}", line)?;
        file.sync_all()?;

        Ok(())
    }

    /// Read all tickets
    pub fn read_all(&self) -> std::io::Result<Vec<Ticket>> {
        if !self.history_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.history_path)?;
        let reader = BufReader::new(file);
        let mut tickets = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(ticket) = serde_json::from_str(&line) {
                tickets.push(ticket);
            }
        }

        Ok(tickets)
    }

    /// Get recent tickets (last N)
    pub fn recent(&self, limit: usize) -> std::io::Result<Vec<Ticket>> {
        let mut tickets = self.read_all()?;
        tickets.reverse();
        tickets.truncate(limit);
        Ok(tickets)
    }

    /// Count tickets by status
    pub fn count_by_status(&self) -> std::io::Result<Vec<(TicketStatus, u32)>> {
        let tickets = self.read_all()?;
        let mut counts = std::collections::HashMap::new();

        for t in tickets {
            *counts.entry(t.status).or_insert(0u32) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(result)
    }

    /// Get ticket statistics
    pub fn stats(&self) -> std::io::Result<TicketStats> {
        let tickets = self.read_all()?;

        let total = tickets.len() as u64;
        let resolved = tickets.iter().filter(|t| t.status == TicketStatus::Resolved).count() as u64;
        let escalated = tickets.iter().filter(|t| t.was_escalated).count() as u64;

        let avg_resolution_ms = if resolved > 0 {
            tickets
                .iter()
                .filter_map(|t| t.resolution_ms)
                .sum::<u64>() / resolved
        } else {
            0
        };

        let avg_reliability = if resolved > 0 {
            tickets
                .iter()
                .filter_map(|t| t.reliability)
                .map(|r| r as f64)
                .sum::<f64>() / resolved as f64
        } else {
            0.0
        };

        let avg_interactions = if total > 0 {
            tickets.iter().map(|t| t.interaction_count as f64).sum::<f64>() / total as f64
        } else {
            0.0
        };

        Ok(TicketStats {
            total_tickets: total,
            resolved_tickets: resolved,
            escalated_tickets: escalated,
            avg_resolution_ms,
            avg_reliability,
            avg_interactions,
        })
    }
}

impl Default for TicketTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated ticket statistics
#[derive(Debug, Clone, Default)]
pub struct TicketStats {
    pub total_tickets: u64,
    pub resolved_tickets: u64,
    pub escalated_tickets: u64,
    pub avg_resolution_ms: u64,
    pub avg_reliability: f64,
    pub avg_interactions: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_number_format() {
        let tracker = TicketTracker::new();
        let case = tracker.next_case_number();
        assert!(case.starts_with("CN-"));
        assert!(case.len() > 10);
    }

    #[test]
    fn test_ticket_lifecycle() {
        let mut ticket = Ticket::new(
            "CN-0001-06122025".to_string(),
            "How much RAM?".to_string(),
            "hardware".to_string(),
        );

        assert_eq!(ticket.status, TicketStatus::New);
        assert!(ticket.is_open());

        ticket.assign("hardware_jr");
        assert_eq!(ticket.status, TicketStatus::Assigned);

        ticket.start_work();
        assert_eq!(ticket.status, TicketStatus::InProgress);
        assert_eq!(ticket.interaction_count, 1);

        ticket.resolve("16 GB".to_string(), 95, 150);
        assert_eq!(ticket.status, TicketStatus::Resolved);
        assert!(!ticket.is_open());
    }

    #[test]
    fn test_ticket_escalation() {
        let mut ticket = Ticket::new(
            "CN-0002-06122025".to_string(),
            "Complex network issue".to_string(),
            "network".to_string(),
        );

        ticket.assign("network_jr");
        ticket.start_work();
        ticket.escalate("network_sr");

        assert!(ticket.was_escalated);
        assert_eq!(ticket.status, TicketStatus::Escalated);
        assert_eq!(ticket.assigned_to, Some("network_sr".to_string()));
    }
}
