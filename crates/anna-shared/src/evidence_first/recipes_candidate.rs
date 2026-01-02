//! Recipe candidate tracking and confirmation logic (v0.0.435).

use super::citations::CitationStore;
use super::recipes_helpers::timestamp_now;
use super::recipes_types::RecipeTemplate;
use serde::{Deserialize, Serialize};

/// A candidate recipe awaiting promotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCandidate {
    /// Template this is based on.
    pub template: RecipeTemplate,
    /// Successful executions.
    pub confirmations: Vec<Confirmation>,
    /// Failed executions.
    pub failures: Vec<Failure>,
    /// When first seen.
    pub first_seen: u64,
    /// When last confirmed.
    pub last_confirmed: Option<u64>,
}

impl RecipeCandidate {
    /// Create a new candidate.
    pub fn new(template: RecipeTemplate) -> Self {
        Self {
            template,
            confirmations: Vec::new(),
            failures: Vec::new(),
            first_seen: timestamp_now(),
            last_confirmed: None,
        }
    }

    /// Record a successful execution.
    pub fn record_success(&mut self, ticket_id: &str, citations: &CitationStore) {
        self.confirmations.push(Confirmation {
            ticket_id: ticket_id.to_string(),
            timestamp: timestamp_now(),
            citation_count: citations.citation_count(),
        });
        self.last_confirmed = Some(timestamp_now());
    }

    /// Record a failed execution.
    pub fn record_failure(&mut self, ticket_id: &str, reason: &str) {
        self.failures.push(Failure {
            ticket_id: ticket_id.to_string(),
            timestamp: timestamp_now(),
            reason: reason.to_string(),
        });
    }

    /// Check if ready for promotion.
    pub fn ready_for_promotion(&self) -> bool {
        self.confirmations.len() >= super::MIN_CONFIRMATIONS_FOR_RECIPE
    }

    /// Get confirmation count.
    pub fn confirmation_count(&self) -> usize {
        self.confirmations.len()
    }

    /// Get failure count.
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Get success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.confirmations.len() + self.failures.len();
        if total == 0 {
            0.0
        } else {
            self.confirmations.len() as f64 / total as f64
        }
    }
}

/// A confirmation of recipe success.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmation {
    /// Ticket where this was confirmed.
    pub ticket_id: String,
    /// When confirmed.
    pub timestamp: u64,
    /// Number of citations supporting.
    pub citation_count: usize,
}

/// A failure record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    /// Ticket where this failed.
    pub ticket_id: String,
    /// When failed.
    pub timestamp: u64,
    /// Reason for failure.
    pub reason: String,
}
