//! Resolution record types and utilities.

use serde::{Deserialize, Serialize};

/// A recorded resolution with timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRecord {
    /// Unix timestamp when request started
    pub start_time: u64,
    /// Unix timestamp when resolved
    pub end_time: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Brief description of the request
    pub description: String,
    /// Category of the request
    pub category: Option<String>,
    /// Whether resolution was successful
    pub successful: bool,
    /// Whether it required escalation
    pub escalated: bool,
}

impl ResolutionRecord {
    /// Create a new resolution record
    pub fn new(start_time: u64, end_time: u64, description: &str) -> Self {
        let duration_ms = (end_time - start_time) * 1000;
        Self {
            start_time,
            end_time,
            duration_ms,
            description: description.to_string(),
            category: None,
            successful: true,
            escalated: false,
        }
    }

    /// Create from millisecond timestamps
    pub fn from_ms(start_ms: u64, end_ms: u64, description: &str) -> Self {
        Self {
            start_time: start_ms / 1000,
            end_time: end_ms / 1000,
            duration_ms: end_ms.saturating_sub(start_ms),
            description: description.to_string(),
            category: None,
            successful: true,
            escalated: false,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Mark as failed
    pub fn mark_failed(mut self) -> Self {
        self.successful = false;
        self
    }

    /// Mark as escalated
    pub fn mark_escalated(mut self) -> Self {
        self.escalated = true;
        self
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> f64 {
        self.duration_ms as f64 / 1000.0
    }

    /// Get human-readable duration
    pub fn duration_human(&self) -> String {
        crate::resolution_time::formatting::format_duration_ms(self.duration_ms)
    }
}
