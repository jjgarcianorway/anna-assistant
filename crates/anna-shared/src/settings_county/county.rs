// v0.0.749: Settings County Core (Phase 325)
// Main county management system

use super::config::CountyConfig;
use super::ordinance::CountyOrdinance;
use super::commissioner::CountyCommissioner;
use super::stats::CountyStats;

/// Settings county
#[derive(Debug, Clone, Default)]
pub struct SettingsCounty {
    /// Config
    config: CountyConfig,
    /// Ordinances
    ordinances: Vec<CountyOrdinance>,
    /// Commissioners
    commissioners: Vec<CountyCommissioner>,
    /// Stats
    stats: CountyStats,
}

impl SettingsCounty {
    /// Create new county system
    pub fn new(config: CountyConfig) -> Self {
        Self {
            config,
            ordinances: Vec::new(),
            commissioners: Vec::new(),
            stats: CountyStats::default(),
        }
    }

    /// Add ordinance
    pub fn add_ordinance(&mut self, ordinance: CountyOrdinance) -> bool {
        if self.ordinances.len() >= self.config.max_ordinances {
            return false;
        }
        self.ordinances.push(ordinance);
        self.update_stats();
        true
    }

    /// Get ordinance
    pub fn get_ordinance(&self, id: &str) -> Option<&CountyOrdinance> {
        self.ordinances.iter().find(|o| o.id == id)
    }

    /// Get ordinance mut
    pub fn get_ordinance_mut(&mut self, id: &str) -> Option<&mut CountyOrdinance> {
        self.ordinances.iter_mut().find(|o| o.id == id)
    }

    /// Add commissioner
    pub fn add_commissioner(&mut self, commissioner: CountyCommissioner) {
        self.commissioners.push(commissioner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.ordinances, self.config.county_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CountyStats {
        &self.stats
    }

    /// Ordinance count
    pub fn ordinance_count(&self) -> usize {
        self.ordinances.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_county_new() {
        let c = SettingsCounty::new(CountyConfig::default());
        assert_eq!(c.ordinance_count(), 0);
    }

    #[test]
    fn test_county_add_ordinance() {
        let mut c = SettingsCounty::new(CountyConfig::default());
        c.add_ordinance(CountyOrdinance::new("o1", "Title", "Content"));
        assert_eq!(c.ordinance_count(), 1);
    }
}
