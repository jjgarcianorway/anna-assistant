// v0.0.786: Settings Hideaway (Phase 362)
// Main hideaway system and registry

use std::collections::HashMap;
use super::models::{HideawayConfig, HideawayOccupant, HideawayGuardian, HideawayStats};

/// Settings hideaway
#[derive(Debug, Clone, Default)]
pub struct SettingsHideaway {
    /// Config
    config: HideawayConfig,
    /// Occupants
    occupants: Vec<HideawayOccupant>,
    /// Guardians
    guardians: Vec<HideawayGuardian>,
    /// Stats
    stats: HideawayStats,
}

impl SettingsHideaway {
    /// Create new hideaway system
    pub fn new(config: HideawayConfig) -> Self {
        Self {
            config,
            occupants: Vec::new(),
            guardians: Vec::new(),
            stats: HideawayStats::default(),
        }
    }

    /// Add occupant
    pub fn add_occupant(&mut self, occupant: HideawayOccupant) -> bool {
        if self.occupants.len() >= self.config.max_occupants {
            return false;
        }
        self.occupants.push(occupant);
        self.update_stats();
        true
    }

    /// Get occupant
    pub fn get_occupant(&self, id: &str) -> Option<&HideawayOccupant> {
        self.occupants.iter().find(|o| o.id == id)
    }

    /// Get occupant mut
    pub fn get_occupant_mut(&mut self, id: &str) -> Option<&mut HideawayOccupant> {
        self.occupants.iter_mut().find(|o| o.id == id)
    }

    /// Add guardian
    pub fn add_guardian(&mut self, guardian: HideawayGuardian) {
        self.guardians.push(guardian);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.occupants, self.config.hideaway_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HideawayStats {
        &self.stats
    }

    /// Occupant count
    pub fn occupant_count(&self) -> usize {
        self.occupants.len()
    }
}

/// Hideaway registry
#[derive(Debug, Clone, Default)]
pub struct HideawayRegistry {
    /// Hideaways by ID
    hideaways: HashMap<String, SettingsHideaway>,
}

impl HideawayRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register hideaway
    pub fn register(&mut self, id: impl Into<String>, hideaway: SettingsHideaway) {
        self.hideaways.insert(id.into(), hideaway);
    }

    /// Unregister hideaway
    pub fn unregister(&mut self, id: &str) -> bool {
        self.hideaways.remove(id).is_some()
    }

    /// Get hideaway
    pub fn get(&self, id: &str) -> Option<&SettingsHideaway> {
        self.hideaways.get(id)
    }

    /// Get hideaway mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHideaway> {
        self.hideaways.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.hideaways.len()
    }
}
