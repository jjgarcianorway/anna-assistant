// v0.0.726: Settings Covenant (Phase 302)
// Binding agreement for settings governance - Covenant Implementation

use super::types::{CovenantConfig, CovenantObligation, CovenantStats, CovenantTerm};

/// Settings covenant
#[derive(Debug, Clone, Default)]
pub struct SettingsCovenant {
    /// Config
    config: CovenantConfig,
    /// Terms
    terms: Vec<CovenantTerm>,
    /// Obligations
    obligations: Vec<CovenantObligation>,
    /// Stats
    stats: CovenantStats,
}

impl SettingsCovenant {
    /// Create new covenant system
    pub fn new(config: CovenantConfig) -> Self {
        Self {
            config,
            terms: Vec::new(),
            obligations: Vec::new(),
            stats: CovenantStats::default(),
        }
    }

    /// Add term
    pub fn add_term(&mut self, term: CovenantTerm) -> bool {
        if self.terms.len() >= self.config.max_terms {
            return false;
        }
        self.terms.push(term);
        self.update_stats();
        true
    }

    /// Get term
    pub fn get_term(&self, id: &str) -> Option<&CovenantTerm> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// Get term mut
    pub fn get_term_mut(&mut self, id: &str) -> Option<&mut CovenantTerm> {
        self.terms.iter_mut().find(|t| t.id == id)
    }

    /// Add obligation
    pub fn add_obligation(&mut self, obligation: CovenantObligation) {
        self.obligations.push(obligation);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.terms, self.config.covenant_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CovenantStats {
        &self.stats
    }

    /// Term count
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covenant_new() {
        let c = SettingsCovenant::new(CovenantConfig::default());
        assert_eq!(c.term_count(), 0);
    }

    #[test]
    fn test_covenant_add_term() {
        let mut c = SettingsCovenant::new(CovenantConfig::default());
        c.add_term(CovenantTerm::new("t1", "Title", "Content"));
        assert_eq!(c.term_count(), 1);
    }
}
