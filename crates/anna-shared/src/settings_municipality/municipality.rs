// v0.0.750: Settings Municipality Core (Phase 326)
// Core municipality structure

use super::code::MunicipalityCode;
use super::config::MunicipalityConfig;
use super::councilor::MunicipalityCouncilor;
use super::stats::MunicipalityStats;

/// Settings municipality
#[derive(Debug, Clone, Default)]
pub struct SettingsMunicipality {
    /// Config
    config: MunicipalityConfig,
    /// Codes
    codes: Vec<MunicipalityCode>,
    /// Councilors
    councilors: Vec<MunicipalityCouncilor>,
    /// Stats
    stats: MunicipalityStats,
}

impl SettingsMunicipality {
    /// Create new municipality system
    pub fn new(config: MunicipalityConfig) -> Self {
        Self {
            config,
            codes: Vec::new(),
            councilors: Vec::new(),
            stats: MunicipalityStats::default(),
        }
    }

    /// Add code
    pub fn add_code(&mut self, code: MunicipalityCode) -> bool {
        if self.codes.len() >= self.config.max_codes {
            return false;
        }
        self.codes.push(code);
        self.update_stats();
        true
    }

    /// Get code
    pub fn get_code(&self, id: &str) -> Option<&MunicipalityCode> {
        self.codes.iter().find(|c| c.id == id)
    }

    /// Get code mut
    pub fn get_code_mut(&mut self, id: &str) -> Option<&mut MunicipalityCode> {
        self.codes.iter_mut().find(|c| c.id == id)
    }

    /// Add councilor
    pub fn add_councilor(&mut self, councilor: MunicipalityCouncilor) {
        self.councilors.push(councilor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.codes, self.config.municipality_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MunicipalityStats {
        &self.stats
    }

    /// Code count
    pub fn code_count(&self) -> usize {
        self.codes.len()
    }
}
