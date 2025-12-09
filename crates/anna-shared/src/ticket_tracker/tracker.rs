//! Ticket tracker and storage (v0.0.183).

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use super::status::TicketStatus;
use super::ticket::Ticket;

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

    // === v0.0.113: Async ticket methods ===

    /// Find a ticket by case number
    pub fn find_by_case(&self, case_number: &str) -> std::io::Result<Option<Ticket>> {
        let tickets = self.read_all()?;
        Ok(tickets
            .into_iter()
            .rev()
            .find(|t| t.case_number == case_number))
    }

    /// Get all open tickets (not resolved/closed)
    pub fn open_tickets(&self) -> std::io::Result<Vec<Ticket>> {
        let tickets = self.read_all()?;
        Ok(tickets.into_iter().filter(|t| t.is_open()).collect())
    }

    /// Get tickets pending user response
    pub fn pending_user(&self) -> std::io::Result<Vec<Ticket>> {
        let tickets = self.read_all()?;
        Ok(tickets
            .into_iter()
            .filter(|t| t.status == TicketStatus::PendingUser)
            .collect())
    }

    /// Update a ticket (rewrites entire file - fine for small volumes)
    pub fn update_ticket(&self, updated: &Ticket) -> std::io::Result<()> {
        let mut tickets = self.read_all()?;

        // Find and replace the ticket
        for t in &mut tickets {
            if t.case_number == updated.case_number {
                *t = updated.clone();
                break;
            }
        }

        // Rewrite the entire file
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&self.history_path)?;
        for ticket in &tickets {
            let line = serde_json::to_string(ticket)?;
            writeln!(file, "{}", line)?;
        }
        file.sync_all()?;

        Ok(())
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
        let resolved = tickets
            .iter()
            .filter(|t| t.status == TicketStatus::Resolved)
            .count() as u64;
        let escalated = tickets.iter().filter(|t| t.was_escalated).count() as u64;

        let avg_resolution_ms = if resolved > 0 {
            tickets.iter().filter_map(|t| t.resolution_ms).sum::<u64>() / resolved
        } else {
            0
        };

        let avg_reliability = if resolved > 0 {
            tickets
                .iter()
                .filter_map(|t| t.reliability)
                .map(|r| r as f64)
                .sum::<f64>()
                / resolved as f64
        } else {
            0.0
        };

        let avg_interactions = if total > 0 {
            tickets
                .iter()
                .map(|t| t.interaction_count as f64)
                .sum::<f64>()
                / total as f64
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
