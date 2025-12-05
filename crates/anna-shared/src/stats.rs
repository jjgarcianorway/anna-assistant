//! Per-team statistics for service desk analytics.
//!
//! Tracks ticket resolution metrics by team for monitoring and optimization.

use crate::teams::Team;
use serde::{Deserialize, Serialize};

/// Statistics for a single team
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamStats {
    /// Team this stats belongs to
    pub team: Team,
    /// Total tickets processed
    pub tickets_total: u64,
    /// Tickets verified successfully
    pub tickets_verified: u64,
    /// Tickets that failed verification
    pub tickets_failed: u64,
    /// Average number of review rounds per ticket
    pub avg_rounds: f32,
    /// Rate of escalation to senior (0.0-1.0)
    pub escalation_rate: f32,
    /// Average reliability score
    pub avg_reliability_score: f32,
}

impl TeamStats {
    /// Create new stats for a team
    pub fn new(team: Team) -> Self {
        Self {
            team,
            ..Default::default()
        }
    }

    /// Record a verified ticket
    pub fn record_verified(&mut self, rounds: u8, score: u8, escalated: bool) {
        self.tickets_total += 1;
        self.tickets_verified += 1;
        self.update_averages(rounds, score, escalated);
    }

    /// Record a failed ticket
    pub fn record_failed(&mut self, rounds: u8, score: u8, escalated: bool) {
        self.tickets_total += 1;
        self.tickets_failed += 1;
        self.update_averages(rounds, score, escalated);
    }

    fn update_averages(&mut self, rounds: u8, score: u8, escalated: bool) {
        let n = self.tickets_total as f32;
        // Update running averages
        self.avg_rounds = ((self.avg_rounds * (n - 1.0)) + rounds as f32) / n;
        self.avg_reliability_score = ((self.avg_reliability_score * (n - 1.0)) + score as f32) / n;
        if escalated {
            self.escalation_rate = ((self.escalation_rate * (n - 1.0)) + 1.0) / n;
        } else {
            self.escalation_rate = (self.escalation_rate * (n - 1.0)) / n;
        }
    }

    /// Get success rate (0.0-1.0)
    pub fn success_rate(&self) -> f32 {
        if self.tickets_total == 0 {
            0.0
        } else {
            self.tickets_verified as f32 / self.tickets_total as f32
        }
    }
}

/// Global statistics across all teams
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalStats {
    /// Total requests processed
    pub total_requests: u64,
    /// Per-team statistics
    pub by_team: Vec<TeamStats>,
    /// Most consulted team (highest ticket count)
    pub most_consulted_team: Option<Team>,
}

impl GlobalStats {
    /// Create new global stats with all teams
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            by_team: vec![
                TeamStats::new(Team::Desktop),
                TeamStats::new(Team::Storage),
                TeamStats::new(Team::Network),
                TeamStats::new(Team::Performance),
                TeamStats::new(Team::Services),
                TeamStats::new(Team::Security),
                TeamStats::new(Team::Hardware),
                TeamStats::new(Team::General),
            ],
            most_consulted_team: None,
        }
    }

    /// Get mutable stats for a team
    pub fn get_team_mut(&mut self, team: Team) -> &mut TeamStats {
        self.by_team.iter_mut().find(|s| s.team == team).unwrap()
    }

    /// Get stats for a team
    pub fn get_team(&self, team: Team) -> Option<&TeamStats> {
        self.by_team.iter().find(|s| s.team == team)
    }

    /// Record a ticket result
    pub fn record_ticket(&mut self, team: Team, verified: bool, rounds: u8, score: u8, escalated: bool) {
        self.total_requests += 1;
        let team_stats = self.get_team_mut(team);
        if verified {
            team_stats.record_verified(rounds, score, escalated);
        } else {
            team_stats.record_failed(rounds, score, escalated);
        }
        self.update_most_consulted();
    }

    fn update_most_consulted(&mut self) {
        self.most_consulted_team = self
            .by_team
            .iter()
            .max_by_key(|s| s.tickets_total)
            .filter(|s| s.tickets_total > 0)
            .map(|s| s.team);
    }

    /// Get overall success rate
    pub fn overall_success_rate(&self) -> f32 {
        let total: u64 = self.by_team.iter().map(|s| s.tickets_total).sum();
        let verified: u64 = self.by_team.iter().map(|s| s.tickets_verified).sum();
        if total == 0 { 0.0 } else { verified as f32 / total as f32 }
    }

    /// Get overall average reliability score
    pub fn overall_avg_score(&self) -> f32 {
        let active: Vec<_> = self.by_team.iter().filter(|s| s.tickets_total > 0).collect();
        if active.is_empty() {
            0.0
        } else {
            active.iter().map(|s| s.avg_reliability_score * s.tickets_total as f32).sum::<f32>()
                / active.iter().map(|s| s.tickets_total as f32).sum::<f32>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_stats_new() {
        let stats = TeamStats::new(Team::Storage);
        assert_eq!(stats.team, Team::Storage);
        assert_eq!(stats.tickets_total, 0);
    }

    #[test]
    fn test_team_stats_record_verified() {
        let mut stats = TeamStats::new(Team::Network);
        stats.record_verified(2, 85, false);

        assert_eq!(stats.tickets_total, 1);
        assert_eq!(stats.tickets_verified, 1);
        assert_eq!(stats.avg_rounds, 2.0);
        assert_eq!(stats.avg_reliability_score, 85.0);
        assert_eq!(stats.escalation_rate, 0.0);
    }

    #[test]
    fn test_team_stats_success_rate() {
        let mut stats = TeamStats::new(Team::Performance);
        stats.record_verified(1, 90, false);
        stats.record_verified(2, 85, false);
        stats.record_failed(3, 50, true);

        assert_eq!(stats.tickets_total, 3);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_global_stats_new() {
        let stats = GlobalStats::new();
        assert_eq!(stats.by_team.len(), 8);
        assert!(stats.most_consulted_team.is_none());
    }

    #[test]
    fn test_global_stats_record_ticket() {
        let mut stats = GlobalStats::new();
        stats.record_ticket(Team::Storage, true, 1, 90, false);
        stats.record_ticket(Team::Storage, true, 2, 85, false);
        stats.record_ticket(Team::Network, false, 3, 50, true);

        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.get_team(Team::Storage).unwrap().tickets_total, 2);
        assert_eq!(stats.get_team(Team::Network).unwrap().tickets_total, 1);
        assert_eq!(stats.most_consulted_team, Some(Team::Storage));
    }

    #[test]
    fn test_global_stats_overall_success_rate() {
        let mut stats = GlobalStats::new();
        stats.record_ticket(Team::Storage, true, 1, 90, false);
        stats.record_ticket(Team::Network, true, 1, 85, false);
        stats.record_ticket(Team::Hardware, false, 3, 50, true);

        assert!((stats.overall_success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_serialization() {
        let mut stats = GlobalStats::new();
        stats.record_ticket(Team::Security, true, 1, 88, false);

        let json = serde_json::to_string(&stats).unwrap();
        let parsed: GlobalStats = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_requests, 1);
        assert_eq!(parsed.get_team(Team::Security).unwrap().tickets_verified, 1);
    }
}
