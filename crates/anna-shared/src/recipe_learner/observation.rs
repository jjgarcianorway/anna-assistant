//! Ticket observation types for recipe learning.

use crate::canonical_intents::CanonicalIntent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A ticket observation for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketObservation {
    /// Ticket ID
    pub ticket_id: String,
    /// Canonical intent
    pub intent: CanonicalIntent,
    /// Domain
    pub domain: String,
    /// Probes that were used
    pub probes_used: Vec<String>,
    /// Probe outputs (sanitized)
    pub probe_outputs: HashMap<String, String>,
    /// Answer summary
    pub answer_summary: String,
    /// Answer confidence
    pub confidence: f32,
    /// Was successful (user feedback or status ok)
    pub successful: bool,
    /// Timestamp
    pub timestamp: u64,
}
