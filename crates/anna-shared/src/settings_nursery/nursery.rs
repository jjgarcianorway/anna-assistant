// v0.0.769: Settings Nursery - Main Nursery (Phase 345)

use super::config::NurseryConfig;
use super::seedling::NurserySeedling;
use super::propagator::NurseryPropagator;
use super::stats::NurseryStats;

/// Settings nursery
#[derive(Debug, Clone, Default)]
pub struct SettingsNursery {
    /// Config
    config: NurseryConfig,
    /// Seedlings
    seedlings: Vec<NurserySeedling>,
    /// Propagators
    propagators: Vec<NurseryPropagator>,
    /// Stats
    stats: NurseryStats,
}

impl SettingsNursery {
    /// Create new nursery system
    pub fn new(config: NurseryConfig) -> Self {
        Self {
            config,
            seedlings: Vec::new(),
            propagators: Vec::new(),
            stats: NurseryStats::default(),
        }
    }

    /// Add seedling
    pub fn add_seedling(&mut self, seedling: NurserySeedling) -> bool {
        if self.seedlings.len() >= self.config.max_seedlings {
            return false;
        }
        self.seedlings.push(seedling);
        self.update_stats();
        true
    }

    /// Get seedling
    pub fn get_seedling(&self, id: &str) -> Option<&NurserySeedling> {
        self.seedlings.iter().find(|s| s.id == id)
    }

    /// Get seedling mut
    pub fn get_seedling_mut(&mut self, id: &str) -> Option<&mut NurserySeedling> {
        self.seedlings.iter_mut().find(|s| s.id == id)
    }

    /// Add propagator
    pub fn add_propagator(&mut self, propagator: NurseryPropagator) {
        self.propagators.push(propagator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.seedlings, self.config.nursery_type);
    }

    /// Get stats
    pub fn stats(&self) -> &NurseryStats {
        &self.stats
    }

    /// Seedling count
    pub fn seedling_count(&self) -> usize {
        self.seedlings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nursery_new() {
        let n = SettingsNursery::new(NurseryConfig::default());
        assert_eq!(n.seedling_count(), 0);
    }

    #[test]
    fn test_nursery_add_seedling() {
        let mut n = SettingsNursery::new(NurseryConfig::default());
        n.add_seedling(NurserySeedling::new("s1", "Title", "Content"));
        assert_eq!(n.seedling_count(), 1);
    }
}
