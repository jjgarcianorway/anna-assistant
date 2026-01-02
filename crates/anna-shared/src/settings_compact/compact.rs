// v0.0.729: Settings Compact (Phase 305)
// Settings compact main structure

use super::config::CompactConfig;
use super::term::CompactTerm;
use super::member::CompactMember;
use super::stats::CompactStats;

/// Settings compact
#[derive(Debug, Clone, Default)]
pub struct SettingsCompact {
    /// Config
    config: CompactConfig,
    /// Terms
    terms: Vec<CompactTerm>,
    /// Members
    members: Vec<CompactMember>,
    /// Stats
    stats: CompactStats,
}

impl SettingsCompact {
    /// Create new compact system
    pub fn new(config: CompactConfig) -> Self {
        Self {
            config,
            terms: Vec::new(),
            members: Vec::new(),
            stats: CompactStats::default(),
        }
    }

    /// Add term
    pub fn add_term(&mut self, term: CompactTerm) -> bool {
        if self.terms.len() >= self.config.max_terms {
            return false;
        }
        self.terms.push(term);
        self.update_stats();
        true
    }

    /// Get term
    pub fn get_term(&self, id: &str) -> Option<&CompactTerm> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// Get term mut
    pub fn get_term_mut(&mut self, id: &str) -> Option<&mut CompactTerm> {
        self.terms.iter_mut().find(|t| t.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: CompactMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.terms, self.config.compact_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CompactStats {
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
    fn test_compact_new() {
        let c = SettingsCompact::new(CompactConfig::default());
        assert_eq!(c.term_count(), 0);
    }

    #[test]
    fn test_compact_add_term() {
        let mut c = SettingsCompact::new(CompactConfig::default());
        c.add_term(CompactTerm::new("t1", "Title", "Content"));
        assert_eq!(c.term_count(), 1);
    }
}
