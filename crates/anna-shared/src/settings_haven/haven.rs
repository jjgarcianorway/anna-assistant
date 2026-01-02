// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Haven module

use super::config::HavenConfig;
use super::guest::HavenGuest;
use super::keeper::HavenKeeper;
use super::stats::HavenStats;

/// Settings haven
#[derive(Debug, Clone, Default)]
pub struct SettingsHaven {
    /// Config
    config: HavenConfig,
    /// Guests
    guests: Vec<HavenGuest>,
    /// Keepers
    keepers: Vec<HavenKeeper>,
    /// Stats
    stats: HavenStats,
}

impl SettingsHaven {
    /// Create new haven system
    pub fn new(config: HavenConfig) -> Self {
        Self {
            config,
            guests: Vec::new(),
            keepers: Vec::new(),
            stats: HavenStats::default(),
        }
    }

    /// Add guest
    pub fn add_guest(&mut self, guest: HavenGuest) -> bool {
        if self.guests.len() >= self.config.max_guests {
            return false;
        }
        self.guests.push(guest);
        self.update_stats();
        true
    }

    /// Get guest
    pub fn get_guest(&self, id: &str) -> Option<&HavenGuest> {
        self.guests.iter().find(|g| g.id == id)
    }

    /// Get guest mut
    pub fn get_guest_mut(&mut self, id: &str) -> Option<&mut HavenGuest> {
        self.guests.iter_mut().find(|g| g.id == id)
    }

    /// Add keeper
    pub fn add_keeper(&mut self, keeper: HavenKeeper) {
        self.keepers.push(keeper);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.guests, self.config.haven_type);
    }

    /// Get stats
    pub fn stats(&self) -> &HavenStats {
        &self.stats
    }

    /// Guest count
    pub fn guest_count(&self) -> usize {
        self.guests.len()
    }
}
