//! Main interaction counter and summary.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{InteractionRecord, InteractionType};
use super::stats::SpecialistStats;

/// Interaction counter tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InteractionCounter {
    /// Total interactions
    pub total_interactions: u64,
    /// Interactions per ticket (for averaging)
    pub interactions_per_ticket: HashMap<String, u64>,
    /// Stats per specialist
    pub by_specialist: HashMap<String, SpecialistStats>,
    /// Stats per interaction type
    pub by_type: HashMap<String, u64>,
    /// Anna solo resolutions (no specialist)
    pub anna_solo_count: u64,
    /// Recent interactions (last 50)
    pub recent: Vec<InteractionRecord>,
}

impl InteractionCounter {
    /// Create new counter
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an interaction
    pub fn record(&mut self, record: InteractionRecord) {
        self.total_interactions += 1;

        // Track by ticket
        if let Some(ticket_id) = &record.ticket_id {
            *self.interactions_per_ticket.entry(ticket_id.clone()).or_default() += 1;
        }

        // Track by specialist (the 'to' field for dispatches, 'from' for responses)
        let specialist = match record.interaction_type {
            InteractionType::Dispatch => &record.to,
            _ => &record.from,
        };

        if specialist != "Anna" && specialist != "User" {
            let stats = self.by_specialist.entry(specialist.clone()).or_default();
            stats.record(record.interaction_type, record.duration_ms);
        }

        // Track by type
        let type_name = record.interaction_type.display_name().to_string();
        *self.by_type.entry(type_name).or_default() += 1;

        // Add to recent
        self.recent.push(record);
        if self.recent.len() > 50 {
            self.recent.remove(0);
        }
    }

    /// Record Anna solving without specialist
    pub fn record_anna_solo(&mut self) {
        self.anna_solo_count += 1;
    }

    /// Get average interactions per ticket
    pub fn average_per_ticket(&self) -> f64 {
        if self.interactions_per_ticket.is_empty() {
            0.0
        } else {
            let total: u64 = self.interactions_per_ticket.values().sum();
            total as f64 / self.interactions_per_ticket.len() as f64
        }
    }

    /// Get total tickets tracked
    pub fn total_tickets(&self) -> usize {
        self.interactions_per_ticket.len()
    }

    /// Get most consulted specialist
    pub fn most_consulted(&self) -> Option<(&str, u64)> {
        self.by_specialist
            .iter()
            .max_by_key(|(_, s)| s.total_interactions)
            .map(|(name, stats)| (name.as_str(), stats.total_interactions))
    }

    /// Get least consulted specialist
    pub fn least_consulted(&self) -> Option<(&str, u64)> {
        self.by_specialist
            .iter()
            .filter(|(_, s)| s.total_interactions > 0)
            .min_by_key(|(_, s)| s.total_interactions)
            .map(|(name, stats)| (name.as_str(), stats.total_interactions))
    }

    /// Get fastest responding specialist
    pub fn fastest_responder(&self) -> Option<(&str, f64)> {
        self.by_specialist
            .iter()
            .filter(|(_, s)| s.avg_response_ms > 0.0)
            .min_by(|a, b| {
                a.1.avg_response_ms
                    .partial_cmp(&b.1.avg_response_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, stats)| (name.as_str(), stats.avg_response_ms))
    }

    /// Get Anna's solo resolution rate
    pub fn anna_solo_rate(&self) -> f64 {
        let total_resolutions = self.anna_solo_count + self.total_tickets() as u64;
        if total_resolutions == 0 {
            0.0
        } else {
            self.anna_solo_count as f64 / total_resolutions as f64 * 100.0
        }
    }

    /// Generate summary
    pub fn summary(&self) -> InteractionSummary {
        InteractionSummary {
            total: self.total_interactions,
            tickets: self.total_tickets(),
            avg_per_ticket: self.average_per_ticket(),
            anna_solo: self.anna_solo_count,
            anna_solo_rate: self.anna_solo_rate(),
            most_consulted: self.most_consulted().map(|(n, _)| n.to_string()),
            specialist_count: self.by_specialist.len(),
        }
    }
}

/// Interaction counter summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSummary {
    /// Total interactions
    pub total: u64,
    /// Total tickets
    pub tickets: usize,
    /// Average per ticket
    pub avg_per_ticket: f64,
    /// Anna solo count
    pub anna_solo: u64,
    /// Anna solo rate
    pub anna_solo_rate: f64,
    /// Most consulted specialist
    pub most_consulted: Option<String>,
    /// Specialist count
    pub specialist_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_record() {
        let mut counter = InteractionCounter::new();

        let record = InteractionRecord::new("Anna", "Desktop Admin", InteractionType::Dispatch, 1000)
            .with_ticket("TKT-001");

        counter.record(record);

        assert_eq!(counter.total_interactions, 1);
        assert_eq!(counter.total_tickets(), 1);
    }

    #[test]
    fn test_average_per_ticket() {
        let mut counter = InteractionCounter::new();

        // Ticket 1: 2 interactions
        counter.record(
            InteractionRecord::new("Anna", "Admin", InteractionType::Dispatch, 1000)
                .with_ticket("TKT-001"),
        );
        counter.record(
            InteractionRecord::new("Admin", "Anna", InteractionType::Response, 2000)
                .with_ticket("TKT-001"),
        );

        // Ticket 2: 4 interactions
        for i in 0..4 {
            counter.record(
                InteractionRecord::new("Anna", "Network", InteractionType::Dispatch, 3000 + i * 100)
                    .with_ticket("TKT-002"),
            );
        }

        // Average should be 3 (6 total / 2 tickets)
        assert_eq!(counter.average_per_ticket(), 3.0);
    }

    #[test]
    fn test_anna_solo() {
        let mut counter = InteractionCounter::new();

        counter.record_anna_solo();
        counter.record_anna_solo();

        // Add one ticket with specialist
        counter.record(
            InteractionRecord::new("Anna", "Admin", InteractionType::Dispatch, 1000)
                .with_ticket("TKT-001"),
        );

        // 2 solo / (2 solo + 1 ticket) = 66.67%
        assert!(counter.anna_solo_rate() > 60.0);
    }

    #[test]
    fn test_most_consulted() {
        let mut counter = InteractionCounter::new();

        // Desktop Admin: 3 interactions
        for i in 0..3 {
            counter.record(InteractionRecord::new(
                "Anna",
                "Desktop Admin",
                InteractionType::Dispatch,
                i * 1000,
            ));
        }

        // Network: 1 interaction
        counter.record(InteractionRecord::new(
            "Anna",
            "Network",
            InteractionType::Dispatch,
            5000,
        ));

        let (name, count) = counter.most_consulted().unwrap();
        assert_eq!(name, "Desktop Admin");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_recent_limit() {
        let mut counter = InteractionCounter::new();

        for i in 0..60 {
            counter.record(InteractionRecord::new(
                "Anna",
                "Admin",
                InteractionType::Dispatch,
                i * 1000,
            ));
        }

        assert_eq!(counter.recent.len(), 50);
    }

    #[test]
    fn test_summary() {
        let mut counter = InteractionCounter::new();

        counter.record(InteractionRecord::new(
            "Anna",
            "Admin",
            InteractionType::Dispatch,
            1000,
        ));
        counter.record_anna_solo();

        let summary = counter.summary();
        assert_eq!(summary.total, 1);
        assert_eq!(summary.anna_solo, 1);
    }
}
