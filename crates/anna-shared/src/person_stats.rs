//! Per-person statistics for service desk analytics (v0.0.32).
//!
//! Tracks individual specialist performance.
//! Extracted from stats.rs to keep modules under 400 lines.

use crate::roster::{person_for, Tier};
use crate::teams::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
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
        let mut sorted: Vec<_> = self.by_person.values().filter(|s| s.tickets_closed > 0).collect();
        sorted.sort_by(|a, b| b.tickets_closed.cmp(&a.tickets_closed));
        sorted.truncate(limit);
        sorted
    }

    /// Get persons who escalated most (juniors)
    pub fn top_escalators(&self, limit: usize) -> Vec<&PersonStats> {
        let mut sorted: Vec<_> = self.by_person.values().filter(|s| s.escalations_sent > 0).collect();
        sorted.sort_by(|a, b| b.escalations_sent.cmp(&a.escalations_sent));
        sorted.truncate(limit);
        sorted
    }

    /// Get persons with best average score
    pub fn top_by_score(&self, limit: usize) -> Vec<&PersonStats> {
        let mut sorted: Vec<_> = self.by_person.values().filter(|s| s.tickets_closed > 0).collect();
        sorted.sort_by(|a, b| {
            b.avg_score
                .partial_cmp(&a.avg_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(limit);
        sorted
    }
}
