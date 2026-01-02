//! Specialist statistics tracking.

use serde::{Deserialize, Serialize};
use super::types::InteractionType;

/// Statistics for a specific specialist
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistStats {
    /// Total interactions
    pub total_interactions: u64,
    /// Dispatches received
    pub dispatches: u64,
    /// Responses sent
    pub responses: u64,
    /// Escalations made
    pub escalations: u64,
    /// Clarifications requested
    pub clarifications: u64,
    /// Average response time (ms)
    pub avg_response_ms: f64,
    /// Total response time (ms) for averaging
    total_response_ms: u64,
    response_count: u64,
}

impl SpecialistStats {
    /// Record an interaction
    pub fn record(&mut self, interaction_type: InteractionType, duration_ms: Option<u64>) {
        self.total_interactions += 1;

        match interaction_type {
            InteractionType::Dispatch => self.dispatches += 1,
            InteractionType::Response => {
                self.responses += 1;
                if let Some(ms) = duration_ms {
                    self.total_response_ms += ms;
                    self.response_count += 1;
                    self.avg_response_ms =
                        self.total_response_ms as f64 / self.response_count as f64;
                }
            }
            InteractionType::Escalation => self.escalations += 1,
            InteractionType::Clarification => self.clarifications += 1,
            _ => {}
        }
    }

    /// Get escalation rate
    pub fn escalation_rate(&self) -> f64 {
        if self.dispatches == 0 {
            0.0
        } else {
            self.escalations as f64 / self.dispatches as f64 * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_stats_record() {
        let mut stats = SpecialistStats::default();

        stats.record(InteractionType::Dispatch, None);
        stats.record(InteractionType::Response, Some(500));
        stats.record(InteractionType::Escalation, None);

        assert_eq!(stats.dispatches, 1);
        assert_eq!(stats.responses, 1);
        assert_eq!(stats.escalations, 1);
        assert_eq!(stats.avg_response_ms, 500.0);
    }

    #[test]
    fn test_escalation_rate() {
        let mut stats = SpecialistStats::default();

        stats.record(InteractionType::Dispatch, None);
        stats.record(InteractionType::Dispatch, None);
        stats.record(InteractionType::Escalation, None);

        assert_eq!(stats.escalation_rate(), 50.0);
    }
}
