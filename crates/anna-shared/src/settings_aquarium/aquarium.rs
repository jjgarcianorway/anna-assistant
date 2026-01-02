// v0.0.775: Settings Aquarium - Aquarium Module (Phase 351)
// Main settings aquarium implementation

use super::config::AquariumConfig;
use super::inhabitant::{AquariumInhabitant, AquariumAquarist};
use super::stats::AquariumStats;

/// Settings aquarium
#[derive(Debug, Clone, Default)]
pub struct SettingsAquarium {
    /// Config
    config: AquariumConfig,
    /// Inhabitants
    inhabitants: Vec<AquariumInhabitant>,
    /// Aquarists
    aquarists: Vec<AquariumAquarist>,
    /// Stats
    stats: AquariumStats,
}

impl SettingsAquarium {
    /// Create new aquarium system
    pub fn new(config: AquariumConfig) -> Self {
        Self {
            config,
            inhabitants: Vec::new(),
            aquarists: Vec::new(),
            stats: AquariumStats::default(),
        }
    }

    /// Add inhabitant
    pub fn add_inhabitant(&mut self, inhabitant: AquariumInhabitant) -> bool {
        if self.inhabitants.len() >= self.config.max_inhabitants {
            return false;
        }
        self.inhabitants.push(inhabitant);
        self.update_stats();
        true
    }

    /// Get inhabitant
    pub fn get_inhabitant(&self, id: &str) -> Option<&AquariumInhabitant> {
        self.inhabitants.iter().find(|i| i.id == id)
    }

    /// Get inhabitant mut
    pub fn get_inhabitant_mut(&mut self, id: &str) -> Option<&mut AquariumInhabitant> {
        self.inhabitants.iter_mut().find(|i| i.id == id)
    }

    /// Add aquarist
    pub fn add_aquarist(&mut self, aquarist: AquariumAquarist) {
        self.aquarists.push(aquarist);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.inhabitants, self.config.aquarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AquariumStats {
        &self.stats
    }

    /// Inhabitant count
    pub fn inhabitant_count(&self) -> usize {
        self.inhabitants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aquarium_new() {
        let a = SettingsAquarium::new(AquariumConfig::default());
        assert_eq!(a.inhabitant_count(), 0);
    }

    #[test]
    fn test_aquarium_add_inhabitant() {
        let mut a = SettingsAquarium::new(AquariumConfig::default());
        a.add_inhabitant(AquariumInhabitant::new("i1", "Title", "Content"));
        assert_eq!(a.inhabitant_count(), 1);
    }
}
