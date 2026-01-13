//! Specialist Statistics - Per-specialist performance tracking.

use super::output::SpecialistOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-specialist statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistMetrics {
    /// Specialist ID.
    pub specialist_id: String,
    /// Total tickets handled.
    pub tickets_handled: u64,
    /// Successfully resolved tickets.
    pub tickets_resolved: u64,
    /// Tickets escalated to senior.
    pub tickets_escalated: u64,
    /// Total resolution time in milliseconds.
    pub total_resolution_ms: u64,
    /// Recipes created from high-confidence resolutions.
    pub recipes_created: u32,
    /// Sum of confidence scores for averaging.
    pub total_confidence: f64,
}

impl SpecialistMetrics {
    /// Create new metrics for a specialist.
    pub fn new(specialist_id: &str) -> Self {
        Self {
            specialist_id: specialist_id.to_string(),
            ..Default::default()
        }
    }

    /// Record a ticket handled.
    pub fn record_ticket(&mut self, output: &SpecialistOutput, duration_ms: u64) {
        self.tickets_handled += 1;
        self.total_resolution_ms += duration_ms;

        match output {
            SpecialistOutput::Completed {
                confidence,
                should_learn,
                ..
            } => {
                self.tickets_resolved += 1;
                self.total_confidence += *confidence as f64;
                if *should_learn {
                    self.recipes_created += 1;
                }
            }
            SpecialistOutput::NeedsEscalation { .. } => {
                self.tickets_escalated += 1;
            }
            SpecialistOutput::Failed { can_escalate, .. } => {
                if *can_escalate {
                    self.tickets_escalated += 1;
                }
            }
            SpecialistOutput::NeedsHelpers { .. } => {
                // Not counted as resolved or escalated
            }
        }
    }

    /// Get average resolution time in milliseconds.
    pub fn avg_resolution_ms(&self) -> u64 {
        if self.tickets_resolved > 0 {
            self.total_resolution_ms / self.tickets_resolved
        } else {
            0
        }
    }

    /// Get average confidence score.
    pub fn avg_confidence(&self) -> f32 {
        if self.tickets_resolved > 0 {
            (self.total_confidence / self.tickets_resolved as f64) as f32
        } else {
            0.0
        }
    }

    /// Get resolution rate (0.0 - 1.0).
    pub fn resolution_rate(&self) -> f32 {
        if self.tickets_handled > 0 {
            self.tickets_resolved as f32 / self.tickets_handled as f32
        } else {
            0.0
        }
    }
}

/// Aggregate statistics for all specialists.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistStatsStore {
    /// Per-specialist metrics.
    pub specialists: HashMap<String, SpecialistMetrics>,
    /// Total tickets across all specialists.
    pub total_tickets: u64,
    /// Total escalations.
    pub total_escalations: u64,
    /// Total recipes created.
    pub total_recipes_created: u32,
}

impl SpecialistStatsStore {
    /// Record stats for a specialist.
    pub fn record(&mut self, specialist_id: &str, output: &SpecialistOutput, duration_ms: u64) {
        let metrics = self
            .specialists
            .entry(specialist_id.to_string())
            .or_insert_with(|| SpecialistMetrics::new(specialist_id));

        metrics.record_ticket(output, duration_ms);

        self.total_tickets += 1;

        if matches!(output, SpecialistOutput::NeedsEscalation { .. }) {
            self.total_escalations += 1;
        }

        if output.should_learn_recipe() {
            self.total_recipes_created += 1;
        }
    }

    /// Get metrics for a specialist.
    pub fn get(&self, specialist_id: &str) -> Option<&SpecialistMetrics> {
        self.specialists.get(specialist_id)
    }

    /// Get all specialist metrics sorted by tickets handled.
    pub fn all_sorted(&self) -> Vec<&SpecialistMetrics> {
        let mut metrics: Vec<_> = self.specialists.values().collect();
        metrics.sort_by(|a, b| b.tickets_handled.cmp(&a.tickets_handled));
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_metrics_new() {
        let metrics = SpecialistMetrics::new("sys-jr");
        assert_eq!(metrics.specialist_id, "sys-jr");
        assert_eq!(metrics.tickets_handled, 0);
    }

    #[test]
    fn test_record_completed_ticket() {
        let mut metrics = SpecialistMetrics::new("sys-jr");
        let output = SpecialistOutput::Completed {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            commands_executed: vec!["df -h".to_string()],
            outputs: vec!["output".to_string()],
            confidence: 0.9,
            recipe_used: None,
            should_learn: true,
        };

        metrics.record_ticket(&output, 100);

        assert_eq!(metrics.tickets_handled, 1);
        assert_eq!(metrics.tickets_resolved, 1);
        assert_eq!(metrics.recipes_created, 1);
        assert_eq!(metrics.avg_resolution_ms(), 100);
        assert!((metrics.avg_confidence() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_record_escalated_ticket() {
        let mut metrics = SpecialistMetrics::new("net-jr");
        let output = SpecialistOutput::NeedsEscalation {
            specialist_id: "net-jr".to_string(),
            reason: "Complex routing".to_string(),
        };

        metrics.record_ticket(&output, 50);

        assert_eq!(metrics.tickets_handled, 1);
        assert_eq!(metrics.tickets_resolved, 0);
        assert_eq!(metrics.tickets_escalated, 1);
    }

    #[test]
    fn test_resolution_rate() {
        let mut metrics = SpecialistMetrics::new("sys-jr");

        // 2 resolved, 1 escalated = 66% resolution rate
        let resolved = SpecialistOutput::Completed {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            commands_executed: vec![],
            outputs: vec![],
            confidence: 0.8,
            recipe_used: None,
            should_learn: false,
        };
        let escalated = SpecialistOutput::NeedsEscalation {
            specialist_id: "sys-jr".to_string(),
            reason: "Too complex".to_string(),
        };

        metrics.record_ticket(&resolved, 100);
        metrics.record_ticket(&resolved, 100);
        metrics.record_ticket(&escalated, 50);

        assert!((metrics.resolution_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_stats_store() {
        let mut store = SpecialistStatsStore::default();

        let output1 = SpecialistOutput::Completed {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            commands_executed: vec![],
            outputs: vec![],
            confidence: 0.9,
            recipe_used: None,
            should_learn: true,
        };
        let output2 = SpecialistOutput::Completed {
            specialist_id: "net-jr".to_string(),
            specialist_name: "Michael".to_string(),
            commands_executed: vec![],
            outputs: vec![],
            confidence: 0.8,
            recipe_used: None,
            should_learn: false,
        };

        store.record("sys-jr", &output1, 100);
        store.record("net-jr", &output2, 150);

        assert_eq!(store.total_tickets, 2);
        assert_eq!(store.total_recipes_created, 1);
        assert!(store.get("sys-jr").is_some());
        assert!(store.get("net-jr").is_some());
    }
}
