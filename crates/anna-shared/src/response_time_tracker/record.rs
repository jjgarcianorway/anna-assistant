// v0.0.538: Response Time Tracker - Record (Phase 114)
// Single response time record and operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{ComplexityLevel, ResponseType};

/// Single response time record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeRecord {
    pub id: String,
    pub ticket_id: Option<String>,
    pub response_type: ResponseType,
    pub complexity: ComplexityLevel,
    pub time_ms: u64,
    pub token_count: Option<u32>,
    pub word_count: u32,
    pub timestamp: DateTime<Utc>,
}

impl ResponseTimeRecord {
    /// Create new record
    pub fn new(id: impl Into<String>, time_ms: u64, word_count: u32) -> Self {
        Self {
            id: id.into(),
            ticket_id: None,
            response_type: ResponseType::default(),
            complexity: ComplexityLevel::default(),
            time_ms,
            token_count: None,
            word_count,
            timestamp: Utc::now(),
        }
    }

    /// Set response type
    pub fn with_type(mut self, response_type: ResponseType) -> Self {
        self.response_type = response_type;
        self
    }

    /// Set complexity
    pub fn with_complexity(mut self, complexity: ComplexityLevel) -> Self {
        self.complexity = complexity;
        self
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: impl Into<String>) -> Self {
        self.ticket_id = Some(ticket_id.into());
        self
    }

    /// Set token count
    pub fn with_tokens(mut self, count: u32) -> Self {
        self.token_count = Some(count);
        self
    }

    /// Words per second
    pub fn words_per_second(&self) -> f64 {
        if self.time_ms == 0 {
            return 0.0;
        }
        (self.word_count as f64 / self.time_ms as f64) * 1000.0
    }
}
