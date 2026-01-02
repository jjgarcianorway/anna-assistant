// v0.0.746: Settings Province - Province (Phase 322)
// Main province structure

use super::config::ProvinceConfig;
use super::edict::ProvinceEdict;
use super::governor::ProvinceGovernor;
use super::stats::ProvinceStats;

/// Settings province
#[derive(Debug, Clone, Default)]
pub struct SettingsProvince {
    /// Config
    config: ProvinceConfig,
    /// Edicts
    edicts: Vec<ProvinceEdict>,
    /// Governors
    governors: Vec<ProvinceGovernor>,
    /// Stats
    stats: ProvinceStats,
}

impl SettingsProvince {
    /// Create new province system
    pub fn new(config: ProvinceConfig) -> Self {
        Self {
            config,
            edicts: Vec::new(),
            governors: Vec::new(),
            stats: ProvinceStats::default(),
        }
    }

    /// Add edict
    pub fn add_edict(&mut self, edict: ProvinceEdict) -> bool {
        if self.edicts.len() >= self.config.max_edicts {
            return false;
        }
        self.edicts.push(edict);
        self.update_stats();
        true
    }

    /// Get edict
    pub fn get_edict(&self, id: &str) -> Option<&ProvinceEdict> {
        self.edicts.iter().find(|e| e.id == id)
    }

    /// Get edict mut
    pub fn get_edict_mut(&mut self, id: &str) -> Option<&mut ProvinceEdict> {
        self.edicts.iter_mut().find(|e| e.id == id)
    }

    /// Add governor
    pub fn add_governor(&mut self, governor: ProvinceGovernor) {
        self.governors.push(governor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.edicts, self.config.province_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ProvinceStats {
        &self.stats
    }

    /// Edict count
    pub fn edict_count(&self) -> usize {
        self.edicts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_province_new() {
        let p = SettingsProvince::new(ProvinceConfig::default());
        assert_eq!(p.edict_count(), 0);
    }

    #[test]
    fn test_province_add_edict() {
        let mut p = SettingsProvince::new(ProvinceConfig::default());
        p.add_edict(ProvinceEdict::new("e1", "Title", "Content"));
        assert_eq!(p.edict_count(), 1);
    }
}
