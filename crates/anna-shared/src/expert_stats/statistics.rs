//! Individual expert statistics.

use serde::{Deserialize, Serialize};

/// Statistics for a single expert
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpertStatistics {
    /// Tickets closed
    pub tickets_closed: u64,
    /// Tickets escalated (for juniors)
    pub tickets_escalated: u64,
    /// Average confidence on resolutions
    pub avg_confidence: f64,
    /// Total confidence (for averaging)
    total_confidence: f64,
    /// High confidence resolutions (>90%)
    pub high_confidence_count: u64,
    /// Resolution count (for averaging)
    resolution_count: u64,
    /// Average response time (ms)
    pub avg_response_ms: f64,
    /// Total response time (for averaging)
    total_response_ms: u64,
    response_count: u64,
}

impl ExpertStatistics {
    /// Record a closed ticket
    pub fn record_closed(&mut self, confidence: f64, response_ms: Option<u64>) {
        self.tickets_closed += 1;
        self.resolution_count += 1;
        self.total_confidence += confidence;
        self.avg_confidence = self.total_confidence / self.resolution_count as f64;

        if confidence >= 0.9 {
            self.high_confidence_count += 1;
        }

        if let Some(ms) = response_ms {
            self.total_response_ms += ms;
            self.response_count += 1;
            self.avg_response_ms = self.total_response_ms as f64 / self.response_count as f64;
        }
    }

    /// Record an escalation
    pub fn record_escalation(&mut self) {
        self.tickets_escalated += 1;
    }

    /// Get escalation rate
    pub fn escalation_rate(&self) -> f64 {
        let total = self.tickets_closed + self.tickets_escalated;
        if total == 0 {
            0.0
        } else {
            self.tickets_escalated as f64 / total as f64 * 100.0
        }
    }

    /// Get high confidence rate
    pub fn high_confidence_rate(&self) -> f64 {
        if self.tickets_closed == 0 {
            0.0
        } else {
            self.high_confidence_count as f64 / self.tickets_closed as f64 * 100.0
        }
    }

    /// Get response count
    pub fn response_count(&self) -> u64 {
        self.response_count
    }
}
