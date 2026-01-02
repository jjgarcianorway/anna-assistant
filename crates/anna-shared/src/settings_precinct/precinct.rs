// v0.0.753: Settings Precinct Core (Phase 329)
// Main precinct implementation

use super::config::PrecinctConfig;
use super::ballot::PrecinctBallot;
use super::captain::PrecinctCaptain;
use super::stats::PrecinctStats;

/// Settings precinct
#[derive(Debug, Clone, Default)]
pub struct SettingsPrecinct {
    /// Config
    config: PrecinctConfig,
    /// Ballots
    ballots: Vec<PrecinctBallot>,
    /// Captains
    captains: Vec<PrecinctCaptain>,
    /// Stats
    stats: PrecinctStats,
}

impl SettingsPrecinct {
    /// Create new precinct system
    pub fn new(config: PrecinctConfig) -> Self {
        Self {
            config,
            ballots: Vec::new(),
            captains: Vec::new(),
            stats: PrecinctStats::default(),
        }
    }

    /// Add ballot
    pub fn add_ballot(&mut self, ballot: PrecinctBallot) -> bool {
        if self.ballots.len() >= self.config.max_ballots {
            return false;
        }
        self.ballots.push(ballot);
        self.update_stats();
        true
    }

    /// Get ballot
    pub fn get_ballot(&self, id: &str) -> Option<&PrecinctBallot> {
        self.ballots.iter().find(|b| b.id == id)
    }

    /// Get ballot mut
    pub fn get_ballot_mut(&mut self, id: &str) -> Option<&mut PrecinctBallot> {
        self.ballots.iter_mut().find(|b| b.id == id)
    }

    /// Add captain
    pub fn add_captain(&mut self, captain: PrecinctCaptain) {
        self.captains.push(captain);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.ballots, self.config.precinct_type);
    }

    /// Get stats
    pub fn stats(&self) -> &PrecinctStats {
        &self.stats
    }

    /// Ballot count
    pub fn ballot_count(&self) -> usize {
        self.ballots.len()
    }
}
