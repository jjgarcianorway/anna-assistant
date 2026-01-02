use super::config::VivariumConfig;
use super::creature::VivariumCreature;
use super::keeper::VivariumKeeper;
use super::stats::VivariumStats;

/// Settings vivarium
#[derive(Debug, Clone, Default)]
pub struct SettingsVivarium {
    /// Config
    config: VivariumConfig,
    /// Creatures
    creatures: Vec<VivariumCreature>,
    /// Keepers
    keepers: Vec<VivariumKeeper>,
    /// Stats
    stats: VivariumStats,
}

impl SettingsVivarium {
    /// Create new vivarium system
    pub fn new(config: VivariumConfig) -> Self {
        Self {
            config,
            creatures: Vec::new(),
            keepers: Vec::new(),
            stats: VivariumStats::default(),
        }
    }

    /// Add creature
    pub fn add_creature(&mut self, creature: VivariumCreature) -> bool {
        if self.creatures.len() >= self.config.max_creatures {
            return false;
        }
        self.creatures.push(creature);
        self.update_stats();
        true
    }

    /// Get creature
    pub fn get_creature(&self, id: &str) -> Option<&VivariumCreature> {
        self.creatures.iter().find(|c| c.id == id)
    }

    /// Get creature mut
    pub fn get_creature_mut(&mut self, id: &str) -> Option<&mut VivariumCreature> {
        self.creatures.iter_mut().find(|c| c.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: VivariumKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.creatures, self.config.vivarium_type);
    }

    /// Get stats
    pub fn stats(&self) -> &VivariumStats {
        &self.stats
    }

    /// Creature count
    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }
}
