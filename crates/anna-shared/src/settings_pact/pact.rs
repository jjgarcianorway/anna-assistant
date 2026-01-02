// v0.0.731: Settings Pact (Phase 307)
// Main SettingsPact implementation

use super::structs::{PactClause, PactConfig, PactParty, PactStats};

/// Settings pact
#[derive(Debug, Clone, Default)]
pub struct SettingsPact {
    /// Config
    config: PactConfig,
    /// Clauses
    clauses: Vec<PactClause>,
    /// Parties
    parties: Vec<PactParty>,
    /// Stats
    stats: PactStats,
}

impl SettingsPact {
    /// Create new pact system
    pub fn new(config: PactConfig) -> Self {
        Self {
            config,
            clauses: Vec::new(),
            parties: Vec::new(),
            stats: PactStats::default(),
        }
    }

    /// Add clause
    pub fn add_clause(&mut self, clause: PactClause) -> bool {
        if self.clauses.len() >= self.config.max_clauses {
            return false;
        }
        self.clauses.push(clause);
        self.update_stats();
        true
    }

    /// Get clause
    pub fn get_clause(&self, id: &str) -> Option<&PactClause> {
        self.clauses.iter().find(|c| c.id == id)
    }

    /// Get clause mut
    pub fn get_clause_mut(&mut self, id: &str) -> Option<&mut PactClause> {
        self.clauses.iter_mut().find(|c| c.id == id)
    }

    /// Add party
    pub fn add_party(&mut self, party: PactParty) {
        self.parties.push(party);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.clauses, self.config.pact_type);
    }

    /// Get stats
    pub fn stats(&self) -> &PactStats {
        &self.stats
    }

    /// Clause count
    pub fn clause_count(&self) -> usize {
        self.clauses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pact_new() {
        let p = SettingsPact::new(PactConfig::default());
        assert_eq!(p.clause_count(), 0);
    }

    #[test]
    fn test_pact_add_clause() {
        let mut p = SettingsPact::new(PactConfig::default());
        p.add_clause(PactClause::new("c1", "Title", "Content"));
        assert_eq!(p.clause_count(), 1);
    }
}
