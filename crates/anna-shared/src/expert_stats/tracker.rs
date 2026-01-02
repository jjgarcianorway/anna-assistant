//! Expert statistics tracker and summary.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::expert::Expert;
use super::level::ExpertLevel;
use super::statistics::ExpertStatistics;

/// Expert statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpertStatsTracker {
    /// Registered experts
    pub experts: HashMap<String, Expert>,
    /// Statistics per expert
    pub stats: HashMap<String, ExpertStatistics>,
    /// Total junior tickets
    pub junior_total: u64,
    /// Total senior tickets
    pub senior_total: u64,
    /// Anna solo tickets
    pub anna_solo: u64,
}

impl ExpertStatsTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an expert
    pub fn register_expert(&mut self, expert: Expert) {
        self.experts.insert(expert.id.clone(), expert);
    }

    /// Record a closed ticket
    pub fn record_closed(
        &mut self,
        expert_id: &str,
        confidence: f64,
        response_ms: Option<u64>,
    ) {
        let stats = self.stats.entry(expert_id.to_string()).or_default();
        stats.record_closed(confidence, response_ms);

        // Update level totals
        if let Some(expert) = self.experts.get(expert_id) {
            match expert.level {
                ExpertLevel::Junior => self.junior_total += 1,
                ExpertLevel::Senior => self.senior_total += 1,
            }
        }
    }

    /// Record an escalation from junior to senior
    pub fn record_escalation(&mut self, junior_id: &str) {
        let stats = self.stats.entry(junior_id.to_string()).or_default();
        stats.record_escalation();
    }

    /// Record Anna solo resolution
    pub fn record_anna_solo(&mut self) {
        self.anna_solo += 1;
    }

    /// Get total tickets handled
    pub fn total_tickets(&self) -> u64 {
        self.junior_total + self.senior_total + self.anna_solo
    }

    /// Get Anna's share
    pub fn anna_share(&self) -> f64 {
        let total = self.total_tickets();
        if total == 0 {
            0.0
        } else {
            self.anna_solo as f64 / total as f64 * 100.0
        }
    }

    /// Get junior share
    pub fn junior_share(&self) -> f64 {
        let total = self.total_tickets();
        if total == 0 {
            0.0
        } else {
            self.junior_total as f64 / total as f64 * 100.0
        }
    }

    /// Get senior share
    pub fn senior_share(&self) -> f64 {
        let total = self.total_tickets();
        if total == 0 {
            0.0
        } else {
            self.senior_total as f64 / total as f64 * 100.0
        }
    }

    /// Get top performers by tickets closed
    pub fn top_performers(&self, limit: usize) -> Vec<(&str, &ExpertStatistics)> {
        let mut sorted: Vec<_> = self.stats.iter().collect();
        sorted.sort_by(|a, b| b.1.tickets_closed.cmp(&a.1.tickets_closed));
        sorted
            .into_iter()
            .take(limit)
            .map(|(id, stats)| (id.as_str(), stats))
            .collect()
    }

    /// Get experts by department
    pub fn by_department(&self, department: &str) -> Vec<(&Expert, &ExpertStatistics)> {
        self.experts
            .values()
            .filter(|e| e.department == department)
            .filter_map(|e| self.stats.get(&e.id).map(|s| (e, s)))
            .collect()
    }

    /// Get experts by level
    pub fn by_level(&self, level: ExpertLevel) -> Vec<(&Expert, &ExpertStatistics)> {
        self.experts
            .values()
            .filter(|e| e.level == level)
            .filter_map(|e| self.stats.get(&e.id).map(|s| (e, s)))
            .collect()
    }

    /// Get most reliable expert (highest confidence)
    pub fn most_reliable(&self) -> Option<(&str, f64)> {
        self.stats
            .iter()
            .filter(|(_, s)| s.tickets_closed >= 5)
            .max_by(|a, b| {
                a.1.avg_confidence
                    .partial_cmp(&b.1.avg_confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, s)| (id.as_str(), s.avg_confidence))
    }

    /// Get fastest responder
    pub fn fastest_responder(&self) -> Option<(&str, f64)> {
        self.stats
            .iter()
            .filter(|(_, s)| s.response_count() > 0)
            .min_by(|a, b| {
                a.1.avg_response_ms
                    .partial_cmp(&b.1.avg_response_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, s)| (id.as_str(), s.avg_response_ms))
    }

    /// Generate summary
    pub fn summary(&self) -> ExpertStatsSummary {
        let top = self.top_performers(1).first().map(|(id, _)| {
            self.experts
                .get(*id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| id.to_string())
        });

        ExpertStatsSummary {
            total_tickets: self.total_tickets(),
            anna_solo: self.anna_solo,
            junior_total: self.junior_total,
            senior_total: self.senior_total,
            expert_count: self.experts.len(),
            top_performer: top,
        }
    }
}

/// Expert stats summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertStatsSummary {
    /// Total tickets
    pub total_tickets: u64,
    /// Anna solo count
    pub anna_solo: u64,
    /// Junior total
    pub junior_total: u64,
    /// Senior total
    pub senior_total: u64,
    /// Expert count
    pub expert_count: usize,
    /// Top performer name
    pub top_performer: Option<String>,
}
