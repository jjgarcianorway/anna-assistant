//! Per-team statistics for service desk analytics (v0.0.32).
//!
//! Tracks ticket resolution metrics by team for monitoring and optimization.
//! v0.0.40: Added clarity counters for clarification verification tracking.
//!
//! Per-person stats moved to person_stats.rs to keep under 400 lines.

use crate::teams::Team;
use serde::{Deserialize, Serialize};

// Re-export PersonStats from person_stats module
pub use crate::person_stats::{PersonStats, PersonStatsTracker};

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
    // === v0.0.39 Performance Stats ===
    /// Requests answered via fast path (no LLM)
    #[serde(default)]
    pub fast_path_hits: u64,
    /// Requests where snapshot was fresh (cache hit)
    #[serde(default)]
    pub snapshot_cache_hits: u64,
    /// Requests where snapshot was stale (required fresh probes)
    #[serde(default)]
    pub snapshot_cache_misses: u64,
    /// Requests using knowledge pack answers
    #[serde(default)]
    pub knowledge_pack_hits: u64,
    /// Requests using learned recipe answers
    #[serde(default)]
    pub recipe_hits: u64,
    /// Translator timeouts (LLM took too long)
    #[serde(default)]
    pub translator_timeouts: u64,
    /// Specialist timeouts (LLM took too long)
    #[serde(default)]
    pub specialist_timeouts: u64,
    // === v0.0.40 Clarity Counters ===
    /// Total clarification questions asked
    #[serde(default)]
    pub clarifications_asked: u64,
    /// Clarification answers that passed verification
    #[serde(default)]
    pub clarifications_verified: u64,
    /// Clarification answers that failed verification
    #[serde(default)]
    pub clarifications_failed: u64,
    /// Facts learned from verified clarifications
    #[serde(default)]
    pub facts_learned: u64,
    /// Clarifications cancelled by user
    #[serde(default)]
    pub clarifications_cancelled: u64,
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
            // v0.0.39 performance stats
            fast_path_hits: 0,
            snapshot_cache_hits: 0,
            snapshot_cache_misses: 0,
            knowledge_pack_hits: 0,
            recipe_hits: 0,
            translator_timeouts: 0,
            specialist_timeouts: 0,
            // v0.0.40 clarity counters
            clarifications_asked: 0,
            clarifications_verified: 0,
            clarifications_failed: 0,
            facts_learned: 0,
            clarifications_cancelled: 0,
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

    // === v0.0.39 Performance Recording ===

    /// Record a fast path hit
    pub fn record_fast_path_hit(&mut self) {
        self.total_requests += 1;
        self.fast_path_hits += 1;
    }

    /// Record snapshot cache status
    pub fn record_snapshot_cache(&mut self, hit: bool) {
        if hit {
            self.snapshot_cache_hits += 1;
        } else {
            self.snapshot_cache_misses += 1;
        }
    }

    /// Record knowledge pack hit
    pub fn record_knowledge_pack_hit(&mut self) {
        self.knowledge_pack_hits += 1;
    }

    /// Record recipe hit
    pub fn record_recipe_hit(&mut self) {
        self.recipe_hits += 1;
    }

    /// Record translator timeout
    pub fn record_translator_timeout(&mut self) {
        self.translator_timeouts += 1;
    }

    /// Record specialist timeout
    pub fn record_specialist_timeout(&mut self) {
        self.specialist_timeouts += 1;
    }

    /// Get fast path percentage (0.0-100.0)
    pub fn fast_path_percentage(&self) -> f32 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.fast_path_hits as f32 / self.total_requests as f32) * 100.0
        }
    }

    /// Get snapshot cache hit rate (0.0-100.0)
    pub fn snapshot_cache_hit_rate(&self) -> f32 {
        let total = self.snapshot_cache_hits + self.snapshot_cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.snapshot_cache_hits as f32 / total as f32) * 100.0
        }
    }

    /// Get timeout rate (0.0-100.0)
    pub fn timeout_rate(&self) -> f32 {
        if self.total_requests == 0 {
            0.0
        } else {
            let timeouts = self.translator_timeouts + self.specialist_timeouts;
            (timeouts as f32 / self.total_requests as f32) * 100.0
        }
    }

    // === v0.0.40 Clarity Recording ===

    /// Record a clarification question asked
    pub fn record_clarification_asked(&mut self) {
        self.clarifications_asked += 1;
    }

    /// Record a verified clarification answer
    pub fn record_clarification_verified(&mut self) {
        self.clarifications_verified += 1;
    }

    /// Record a failed clarification verification
    pub fn record_clarification_failed(&mut self) {
        self.clarifications_failed += 1;
    }

    /// Record a fact learned from verified clarification
    pub fn record_fact_learned(&mut self) {
        self.facts_learned += 1;
    }

    /// Record a clarification cancelled by user
    pub fn record_clarification_cancelled(&mut self) {
        self.clarifications_cancelled += 1;
    }

    /// Get clarification verification rate (0.0-100.0)
    pub fn clarification_verify_rate(&self) -> f32 {
        let total = self.clarifications_verified + self.clarifications_failed;
        if total == 0 {
            0.0
        } else {
            (self.clarifications_verified as f32 / total as f32) * 100.0
        }
    }
}

// Tests moved to tests/stats_tests.rs
// PersonStats and PersonStatsTracker moved to person_stats.rs
