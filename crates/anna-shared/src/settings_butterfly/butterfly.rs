// v0.0.780: Settings Butterfly (Phase 356)
// Main butterfly system

use super::config::ButterflyConfig;
use super::specimen::{ButterflySpecimen, ButterflyCurator};
use super::stats::ButterflyStats;

/// Settings butterfly
#[derive(Debug, Clone, Default)]
pub struct SettingsButterfly {
    /// Config
    config: ButterflyConfig,
    /// Specimens
    specimens: Vec<ButterflySpecimen>,
    /// Curators
    curators: Vec<ButterflyCurator>,
    /// Stats
    stats: ButterflyStats,
}

impl SettingsButterfly {
    /// Create new butterfly system
    pub fn new(config: ButterflyConfig) -> Self {
        Self {
            config,
            specimens: Vec::new(),
            curators: Vec::new(),
            stats: ButterflyStats::default(),
        }
    }

    /// Add specimen
    pub fn add_specimen(&mut self, specimen: ButterflySpecimen) -> bool {
        if self.specimens.len() >= self.config.max_specimens {
            return false;
        }
        self.specimens.push(specimen);
        self.update_stats();
        true
    }

    /// Get specimen
    pub fn get_specimen(&self, id: &str) -> Option<&ButterflySpecimen> {
        self.specimens.iter().find(|s| s.id == id)
    }

    /// Get specimen mut
    pub fn get_specimen_mut(&mut self, id: &str) -> Option<&mut ButterflySpecimen> {
        self.specimens.iter_mut().find(|s| s.id == id)
    }

    /// Add curator
    pub fn add_curator(&mut self, curator: ButterflyCurator) {
        self.curators.push(curator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.specimens, self.config.butterfly_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ButterflyStats {
        &self.stats
    }

    /// Specimen count
    pub fn specimen_count(&self) -> usize {
        self.specimens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_butterfly_new() {
        let b = SettingsButterfly::new(ButterflyConfig::default());
        assert_eq!(b.specimen_count(), 0);
    }

    #[test]
    fn test_butterfly_add_specimen() {
        let mut b = SettingsButterfly::new(ButterflyConfig::default());
        b.add_specimen(ButterflySpecimen::new("s1", "Title", "Content"));
        assert_eq!(b.specimen_count(), 1);
    }
}
