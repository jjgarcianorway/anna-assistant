//! Core types for ticket history (Phase 69)
//!
//! Defines ticket outcomes, historical tickets, and ticket history storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Outcome of a ticket
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketOutcome {
    /// Successfully resolved
    Resolved,
    /// Escalated to senior/human
    Escalated,
    /// User cancelled the request
    Cancelled,
    /// Could not resolve
    Failed,
    /// Still in progress
    InProgress,
    /// Awaiting user input
    AwaitingInput,
}

impl TicketOutcome {
    /// Display string for the outcome
    pub fn display(&self) -> &'static str {
        match self {
            Self::Resolved => "Resolved",
            Self::Escalated => "Escalated",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
            Self::InProgress => "In Progress",
            Self::AwaitingInput => "Awaiting Input",
        }
    }

    /// Symbol for compact display
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Resolved => "[OK]",
            Self::Escalated => "[UP]",
            Self::Cancelled => "[--]",
            Self::Failed => "[!!]",
            Self::InProgress => "[..]",
            Self::AwaitingInput => "[??]",
        }
    }

    /// Whether the ticket is considered closed
    pub fn is_closed(&self) -> bool {
        matches!(
            self,
            Self::Resolved | Self::Escalated | Self::Cancelled | Self::Failed
        )
    }
}

/// A historical ticket entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTicket {
    /// Ticket ID (e.g., CN-0001-13122025)
    pub id: String,
    /// Original user question/request
    pub query: String,
    /// When the ticket was created (Unix timestamp)
    pub created_at: u64,
    /// When the ticket was closed (if closed)
    pub closed_at: Option<u64>,
    /// Outcome of the ticket
    pub outcome: TicketOutcome,
    /// Department that handled the ticket
    pub department: Option<String>,
    /// Specialists who worked on the ticket
    pub specialists: Vec<String>,
    /// Number of interactions for this ticket
    pub interaction_count: u32,
    /// Whether Anna solved it alone
    pub anna_solo: bool,
    /// Brief summary of the resolution (if any)
    pub resolution_summary: Option<String>,
    /// Category/topic of the ticket
    pub category: Option<String>,
}

impl HistoricalTicket {
    /// Create a new historical ticket
    pub fn new(id: impl Into<String>, query: impl Into<String>, created_at: u64) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            created_at,
            closed_at: None,
            outcome: TicketOutcome::InProgress,
            department: None,
            specialists: Vec::new(),
            interaction_count: 0,
            anna_solo: false,
            resolution_summary: None,
            category: None,
        }
    }

    /// Mark the ticket as resolved
    pub fn resolve(&mut self, closed_at: u64, summary: Option<String>) {
        self.closed_at = Some(closed_at);
        self.outcome = TicketOutcome::Resolved;
        self.resolution_summary = summary;
    }

    /// Duration in milliseconds (or None if still open)
    pub fn duration_ms(&self) -> Option<u64> {
        self.closed_at.map(|end| {
            if end > self.created_at {
                (end - self.created_at) * 1000
            } else {
                0
            }
        })
    }
}

/// Ticket history storage and query
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketHistory {
    /// All tickets (most recent first)
    pub tickets: Vec<HistoricalTicket>,
    /// Total tickets ever created
    pub total_created: u64,
    /// Index by outcome for quick filtering
    pub by_outcome: HashMap<String, u64>,
    /// Index by department
    pub by_department: HashMap<String, u64>,
}

impl TicketHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a ticket to history
    pub fn add(&mut self, ticket: HistoricalTicket) {
        // Update indices
        let outcome_key = ticket.outcome.display().to_string();
        *self.by_outcome.entry(outcome_key).or_insert(0) += 1;

        if let Some(ref dept) = ticket.department {
            *self.by_department.entry(dept.clone()).or_insert(0) += 1;
        }

        self.total_created += 1;
        self.tickets.insert(0, ticket); // Most recent first

        // Keep only last 1000 tickets in memory
        if self.tickets.len() > 1000 {
            self.tickets.truncate(1000);
        }
    }

    /// Get recent tickets (default last 10)
    pub fn recent(&self, count: usize) -> &[HistoricalTicket] {
        let end = count.min(self.tickets.len());
        &self.tickets[..end]
    }

    /// Get tickets by outcome
    pub fn by_outcome(&self, outcome: &TicketOutcome) -> Vec<&HistoricalTicket> {
        self.tickets
            .iter()
            .filter(|t| &t.outcome == outcome)
            .collect()
    }

    /// Get open tickets
    pub fn open_tickets(&self) -> Vec<&HistoricalTicket> {
        self.tickets
            .iter()
            .filter(|t| !t.outcome.is_closed())
            .collect()
    }

    /// Get tickets by department
    pub fn by_department(&self, department: &str) -> Vec<&HistoricalTicket> {
        self.tickets
            .iter()
            .filter(|t| t.department.as_deref() == Some(department))
            .collect()
    }

    /// Count resolved tickets
    pub fn resolved_count(&self) -> u64 {
        *self.by_outcome.get("Resolved").unwrap_or(&0)
    }

    /// Count failed tickets
    pub fn failed_count(&self) -> u64 {
        *self.by_outcome.get("Failed").unwrap_or(&0)
    }

    /// Success rate (resolved / total closed)
    pub fn success_rate(&self) -> f64 {
        let closed: u64 = self.tickets.iter().filter(|t| t.outcome.is_closed()).count() as u64;
        if closed == 0 {
            return 0.0;
        }
        (self.resolved_count() as f64 / closed as f64) * 100.0
    }

    /// Get most active department
    pub fn most_active_department(&self) -> Option<(&String, u64)> {
        self.by_department.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }
}
