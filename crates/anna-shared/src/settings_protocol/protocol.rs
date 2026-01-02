// v0.0.728: Settings Protocol - Core Protocol Implementation

use super::config::ProtocolConfig;
use super::data::{ProtocolClause, ProtocolParty, ProtocolStats};

/// Settings protocol
#[derive(Debug, Clone, Default)]
pub struct SettingsProtocol {
    /// Config
    config: ProtocolConfig,
    /// Clauses
    clauses: Vec<ProtocolClause>,
    /// Parties
    parties: Vec<ProtocolParty>,
    /// Stats
    stats: ProtocolStats,
}

impl SettingsProtocol {
    /// Create new protocol system
    pub fn new(config: ProtocolConfig) -> Self {
        Self {
            config,
            clauses: Vec::new(),
            parties: Vec::new(),
            stats: ProtocolStats::default(),
        }
    }

    /// Add clause
    pub fn add_clause(&mut self, clause: ProtocolClause) -> bool {
        if self.clauses.len() >= self.config.max_clauses {
            return false;
        }
        self.clauses.push(clause);
        self.update_stats();
        true
    }

    /// Get clause
    pub fn get_clause(&self, id: &str) -> Option<&ProtocolClause> {
        self.clauses.iter().find(|c| c.id == id)
    }

    /// Get clause mut
    pub fn get_clause_mut(&mut self, id: &str) -> Option<&mut ProtocolClause> {
        self.clauses.iter_mut().find(|c| c.id == id)
    }

    /// Add party
    pub fn add_party(&mut self, party: ProtocolParty) {
        self.parties.push(party);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.clauses, self.config.protocol_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ProtocolStats {
        &self.stats
    }

    /// Clause count
    pub fn clause_count(&self) -> usize {
        self.clauses.len()
    }
}
