// v0.0.778: Settings Aviary (Phase 354)
// Main aviary implementation

use super::config::AviaryConfig;
use super::bird::AviaryBird;
use super::keeper::AviaryKeeper;
use super::stats::AviaryStats;

/// Settings aviary
#[derive(Debug, Clone, Default)]
pub struct SettingsAviary {
    /// Config
    config: AviaryConfig,
    /// Birds
    birds: Vec<AviaryBird>,
    /// Keepers
    keepers: Vec<AviaryKeeper>,
    /// Stats
    stats: AviaryStats,
}

impl SettingsAviary {
    /// Create new aviary system
    pub fn new(config: AviaryConfig) -> Self {
        Self {
            config,
            birds: Vec::new(),
            keepers: Vec::new(),
            stats: AviaryStats::default(),
        }
    }

    /// Add bird
    pub fn add_bird(&mut self, bird: AviaryBird) -> bool {
        if self.birds.len() >= self.config.max_birds {
            return false;
        }
        self.birds.push(bird);
        self.update_stats();
        true
    }

    /// Get bird
    pub fn get_bird(&self, id: &str) -> Option<&AviaryBird> {
        self.birds.iter().find(|b| b.id == id)
    }

    /// Get bird mut
    pub fn get_bird_mut(&mut self, id: &str) -> Option<&mut AviaryBird> {
        self.birds.iter_mut().find(|b| b.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: AviaryKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.birds, self.config.aviary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AviaryStats {
        &self.stats
    }

    /// Bird count
    pub fn bird_count(&self) -> usize {
        self.birds.len()
    }
}
