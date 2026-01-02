//! Resolution criteria for ticket state transitions.

use super::TicketState;

/// Minimum confidence threshold for RESOLVED.
pub const MIN_CONFIDENCE_FOR_RESOLVED: f64 = 0.5;

/// Resolution criteria for transitioning to RESOLVED.
#[derive(Debug, Clone)]
pub struct ResolutionCriteria {
    /// All required evidence collected.
    pub evidence_complete: bool,
    /// Valid answer produced (from specialist or fallback).
    pub valid_answer: bool,
    /// Confidence level.
    pub confidence: f64,
    /// Confidence threshold (default 0.5).
    pub threshold: f64,
}

impl ResolutionCriteria {
    /// Create new criteria.
    pub fn new(evidence_complete: bool, valid_answer: bool, confidence: f64) -> Self {
        Self {
            evidence_complete,
            valid_answer,
            confidence,
            threshold: MIN_CONFIDENCE_FOR_RESOLVED,
        }
    }

    /// Set custom threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Check if criteria met for RESOLVED.
    pub fn is_resolved(&self) -> bool {
        self.evidence_complete && self.valid_answer && self.confidence >= self.threshold
    }

    /// Get reason if not resolved.
    pub fn failure_reason(&self) -> Option<&'static str> {
        if !self.evidence_complete {
            Some("Required evidence not collected")
        } else if !self.valid_answer {
            Some("No valid answer produced")
        } else if self.confidence < self.threshold {
            Some("Confidence below threshold")
        } else {
            None
        }
    }
}

/// State transition event.
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Previous state.
    pub from: TicketState,
    /// New state.
    pub to: TicketState,
    /// Reason for transition.
    pub reason: String,
    /// Timestamp (milliseconds since epoch).
    pub timestamp_ms: u64,
}

impl StateTransition {
    /// Create a new transition.
    pub fn new(from: TicketState, to: TicketState, reason: &str) -> Self {
        Self {
            from,
            to,
            reason: reason.to_string(),
            timestamp_ms: current_timestamp_ms(),
        }
    }
}

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_criteria() {
        let criteria = ResolutionCriteria::new(true, true, 0.8);
        assert!(criteria.is_resolved());

        let low_confidence = ResolutionCriteria::new(true, true, 0.3);
        assert!(!low_confidence.is_resolved());
        assert_eq!(
            low_confidence.failure_reason(),
            Some("Confidence below threshold")
        );

        let no_answer = ResolutionCriteria::new(true, false, 0.9);
        assert!(!no_answer.is_resolved());
        assert_eq!(no_answer.failure_reason(), Some("No valid answer produced"));
    }
}
