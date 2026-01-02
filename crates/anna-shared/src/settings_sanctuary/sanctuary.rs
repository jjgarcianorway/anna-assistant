// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Sanctuary

use super::config::SanctuaryConfig;
use super::resident::SanctuaryResident;
use super::warden::SanctuaryWarden;
use super::stats::SanctuaryStats;

/// Settings sanctuary
#[derive(Debug, Clone, Default)]
pub struct SettingsSanctuary {
    /// Config
    config: SanctuaryConfig,
    /// Residents
    residents: Vec<SanctuaryResident>,
    /// Wardens
    wardens: Vec<SanctuaryWarden>,
    /// Stats
    stats: SanctuaryStats,
}

impl SettingsSanctuary {
    /// Create new sanctuary system
    pub fn new(config: SanctuaryConfig) -> Self {
        Self {
            config,
            residents: Vec::new(),
            wardens: Vec::new(),
            stats: SanctuaryStats::default(),
        }
    }

    /// Add resident
    pub fn add_resident(&mut self, resident: SanctuaryResident) -> bool {
        if self.residents.len() >= self.config.max_residents {
            return false;
        }
        self.residents.push(resident);
        self.update_stats();
        true
    }

    /// Get resident
    pub fn get_resident(&self, id: &str) -> Option<&SanctuaryResident> {
        self.residents.iter().find(|r| r.id == id)
    }

    /// Get resident mut
    pub fn get_resident_mut(&mut self, id: &str) -> Option<&mut SanctuaryResident> {
        self.residents.iter_mut().find(|r| r.id == id)
    }

    /// Add warden
    pub fn add_warden(&mut self, warden: SanctuaryWarden) {
        self.wardens.push(warden);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.residents, self.config.sanctuary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &SanctuaryStats {
        &self.stats
    }

    /// Resident count
    pub fn resident_count(&self) -> usize {
        self.residents.len()
    }
}
