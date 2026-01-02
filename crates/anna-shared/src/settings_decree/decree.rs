// v0.0.720: Settings Decree - Decree (Phase 296)
// Main decree system

use super::config::DecreeConfig;
use super::ruling::{DecreeRuling, DecreeClause};
use super::stats::DecreeStats;

/// Settings decree
#[derive(Debug, Clone, Default)]
pub struct SettingsDecree {
    /// Config
    config: DecreeConfig,
    /// Rulings
    rulings: Vec<DecreeRuling>,
    /// Clauses
    clauses: Vec<DecreeClause>,
    /// Stats
    stats: DecreeStats,
}

impl SettingsDecree {
    /// Create new decree system
    pub fn new(config: DecreeConfig) -> Self {
        Self {
            config,
            rulings: Vec::new(),
            clauses: Vec::new(),
            stats: DecreeStats::default(),
        }
    }

    /// Add ruling
    pub fn add_ruling(&mut self, ruling: DecreeRuling) -> bool {
        if self.rulings.len() >= self.config.max_decrees {
            return false;
        }
        self.rulings.push(ruling);
        self.update_stats();
        true
    }

    /// Get ruling
    pub fn get_ruling(&self, id: &str) -> Option<&DecreeRuling> {
        self.rulings.iter().find(|r| r.id == id)
    }

    /// Get ruling mut
    pub fn get_ruling_mut(&mut self, id: &str) -> Option<&mut DecreeRuling> {
        self.rulings.iter_mut().find(|r| r.id == id)
    }

    /// Add clause
    pub fn add_clause(&mut self, clause: DecreeClause) {
        self.clauses.push(clause);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.rulings, self.config.decree_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DecreeStats {
        &self.stats
    }

    /// Ruling count
    pub fn ruling_count(&self) -> usize {
        self.rulings.len()
    }
}
