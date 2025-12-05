//! Per-team and per-person statistics for service desk analytics (v0.0.32).
//!
//! Tracks ticket resolution metrics by team and person for monitoring and optimization.
//! Person stats allow tracking individual specialist performance.

use crate::roster::{person_for, Tier};
use crate::teams::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

// === Per-Person Statistics (v0.0.32) ===

/// Statistics for a single person in the roster
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonStats {
    /// Person ID from roster
    pub person_id: String,
    /// Display name for convenience
    pub display_name: String,
    /// Team this person belongs to
    pub team: Team,
    /// Tier (junior/senior)
    pub tier: String,
    /// Total tickets closed by this person
    pub tickets_closed: u64,
    /// Tickets escalated to senior (only for juniors)
    pub escalations_sent: u64,
    /// Tickets received via escalation (only for seniors)
    pub escalations_received: u64,
    /// Average review loops before closure
    pub avg_loops: f32,
    /// Average reliability score of closed tickets
    pub avg_score: f32,
}

impl PersonStats {
    /// Create stats for a person
    pub fn new(person_id: &str, display_name: &str, team: Team, tier: &str) -> Self {
        Self {
            person_id: person_id.to_string(),
            display_name: display_name.to_string(),
            team,
            tier: tier.to_string(),
            ..Default::default()
        }
    }

    /// Create stats from roster entry
    pub fn from_roster(team: Team, tier: Tier) -> Self {
        let person = person_for(team, tier);
        Self::new(person.person_id, person.display_name, team, &tier.to_string())
    }

    /// Record a ticket closure
    pub fn record_closure(&mut self, loops: u8, score: u8) {
        self.tickets_closed += 1;
        let n = self.tickets_closed as f32;
        self.avg_loops = ((self.avg_loops * (n - 1.0)) + loops as f32) / n;
        self.avg_score = ((self.avg_score * (n - 1.0)) + score as f32) / n;
    }

    /// Record an escalation sent (junior only)
    pub fn record_escalation_sent(&mut self) {
        self.escalations_sent += 1;
    }

    /// Record an escalation received (senior only)
    pub fn record_escalation_received(&mut self) {
        self.escalations_received += 1;
    }
}

/// Per-person statistics tracker (v0.0.32)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonStatsTracker {
    /// Stats by person_id
    pub by_person: HashMap<String, PersonStats>,
}

impl PersonStatsTracker {
    /// Create new tracker with all roster entries
    pub fn new() -> Self {
        let mut by_person = HashMap::new();
        for team in [
            Team::Desktop, Team::Storage, Team::Network, Team::Performance,
            Team::Services, Team::Security, Team::Hardware, Team::General,
        ] {
            for tier in [Tier::Junior, Tier::Senior] {
                let stats = PersonStats::from_roster(team, tier);
                by_person.insert(stats.person_id.clone(), stats);
            }
        }
        Self { by_person }
    }

    /// Get or create stats for a person
    pub fn get_person_mut(&mut self, person_id: &str) -> Option<&mut PersonStats> {
        self.by_person.get_mut(person_id)
    }

    /// Get stats for a person
    pub fn get_person(&self, person_id: &str) -> Option<&PersonStats> {
        self.by_person.get(person_id)
    }

    /// Record a ticket closure by person
    pub fn record_closure(&mut self, person_id: &str, loops: u8, score: u8) {
        if let Some(stats) = self.by_person.get_mut(person_id) {
            stats.record_closure(loops, score);
        }
    }

    /// Record an escalation from junior to senior
    pub fn record_escalation(&mut self, junior_id: &str, senior_id: &str) {
        if let Some(jr) = self.by_person.get_mut(junior_id) {
            jr.record_escalation_sent();
        }
        if let Some(sr) = self.by_person.get_mut(senior_id) {
            sr.record_escalation_received();
        }
    }

    /// Get top performers by tickets closed
    pub fn top_closers(&self, limit: usize) -> Vec<&PersonStats> {
        let mut sorted: Vec<_> = self.by_person.values()
            .filter(|s| s.tickets_closed > 0)
            .collect();
        sorted.sort_by(|a, b| b.tickets_closed.cmp(&a.tickets_closed));
        sorted.truncate(limit);
        sorted
    }

    /// Get persons who escalated most (juniors)
    pub fn top_escalators(&self, limit: usize) -> Vec<&PersonStats> {
        let mut sorted: Vec<_> = self.by_person.values()
            .filter(|s| s.escalations_sent > 0)
            .collect();
        sorted.sort_by(|a, b| b.escalations_sent.cmp(&a.escalations_sent));
        sorted.truncate(limit);
        sorted
    }

    /// Get persons with best average score
    pub fn top_by_score(&self, limit: usize) -> Vec<&PersonStats> {
        let mut sorted: Vec<_> = self.by_person.values()
            .filter(|s| s.tickets_closed > 0)
            .collect();
        sorted.sort_by(|a, b| b.avg_score.partial_cmp(&a.avg_score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(limit);
        sorted
    }
}

// Tests moved to tests/stats_tests.rs
