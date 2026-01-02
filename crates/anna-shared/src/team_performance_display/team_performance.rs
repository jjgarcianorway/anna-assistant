//! Team performance tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::team_metrics::TeamMetrics;

/// Team performance tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamPerformance {
    /// Metrics by team
    pub teams: HashMap<String, TeamMetrics>,
    /// Total tickets across all teams
    pub total_tickets: u64,
    /// Total resolved across all teams
    pub total_resolved: u64,
}

impl TeamPerformance {
    /// Create a new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create metrics for a team
    pub fn get_or_create(&mut self, team_id: &str) -> &mut TeamMetrics {
        self.teams
            .entry(team_id.to_string())
            .or_insert_with(|| TeamMetrics::new(team_id))
    }

    /// Record a ticket for a team
    pub fn record_ticket(
        &mut self,
        team_id: &str,
        resolution_time_ms: u64,
        resolved: bool,
        is_senior: bool,
    ) {
        self.total_tickets += 1;
        if resolved {
            self.total_resolved += 1;
        }

        let metrics = self.get_or_create(team_id);
        metrics.record_ticket(resolution_time_ms, resolved, is_senior);
    }

    /// Get metrics for a team
    pub fn get_team(&self, team_id: &str) -> Option<&TeamMetrics> {
        self.teams.get(team_id)
    }

    /// Get teams sorted by ticket count (most active first)
    pub fn by_activity(&self) -> Vec<(&String, &TeamMetrics)> {
        let mut teams: Vec<_> = self.teams.iter().collect();
        teams.sort_by(|a, b| b.1.tickets_handled.cmp(&a.1.tickets_handled));
        teams
    }

    /// Get teams sorted by success rate (best first)
    pub fn by_success_rate(&self) -> Vec<(&String, &TeamMetrics)> {
        let mut teams: Vec<_> = self.teams.iter().collect();
        teams.sort_by(|a, b| {
            b.1.success_rate()
                .partial_cmp(&a.1.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        teams
    }

    /// Get teams sorted by average resolution time (fastest first)
    pub fn by_speed(&self) -> Vec<(&String, &TeamMetrics)> {
        let mut teams: Vec<_> = self.teams.iter().filter(|(_, m)| m.avg_resolution_ms() > 0).collect();
        teams.sort_by(|a, b| a.1.avg_resolution_ms().cmp(&b.1.avg_resolution_ms()));
        teams
    }

    /// Get the most active team
    pub fn most_active(&self) -> Option<(&String, &TeamMetrics)> {
        self.by_activity().first().copied()
    }

    /// Get the best performing team (by success rate)
    pub fn best_performing(&self) -> Option<(&String, &TeamMetrics)> {
        self.by_success_rate()
            .into_iter()
            .find(|(_, m)| m.tickets_handled >= 3)
    }

    /// Get the fastest team (by average resolution)
    pub fn fastest(&self) -> Option<(&String, &TeamMetrics)> {
        self.by_speed().first().copied()
    }

    /// Overall success rate
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_tickets == 0 {
            return 0.0;
        }
        (self.total_resolved as f64 / self.total_tickets as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_performance_record() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 5000, true, false);

        assert_eq!(perf.total_tickets, 1);
        assert_eq!(perf.total_resolved, 1);
        assert!(perf.teams.contains_key("Desktop"));
    }

    #[test]
    fn test_team_performance_by_activity() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Network", 1000, true, false);

        let by_activity = perf.by_activity();
        assert_eq!(by_activity[0].0, "Desktop");
    }

    #[test]
    fn test_team_performance_most_active() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Network", 1000, true, false);

        let (name, _) = perf.most_active().unwrap();
        assert_eq!(name, "Desktop");
    }
}
