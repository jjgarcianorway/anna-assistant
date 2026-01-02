// v0.0.695: Settings Folio (Phase 271)
// Settings folio implementation

use super::config::FolioConfig;
use super::section::FolioSection;
use super::stats::FolioStats;
use super::types::FolioStatus;

/// Settings folio
#[derive(Debug, Clone, Default)]
pub struct SettingsFolio {
    /// Config
    config: FolioConfig,
    /// Sections
    sections: Vec<FolioSection>,
    /// Status
    status: FolioStatus,
    /// Stats
    stats: FolioStats,
}

impl SettingsFolio {
    /// Create new folio
    pub fn new(config: FolioConfig) -> Self {
        Self {
            config,
            sections: Vec::new(),
            status: FolioStatus::Open,
            stats: FolioStats::default(),
        }
    }

    /// Add section
    pub fn add_section(&mut self, id: &str, name: &str) -> bool {
        if self.sections.len() >= self.config.max_sections {
            return false;
        }
        let order = self.sections.len();
        self.sections.push(FolioSection::new(id, name, order));
        self.update_stats();
        true
    }

    /// Get section
    pub fn get_section(&self, id: &str) -> Option<&FolioSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// Get section mut
    pub fn get_section_mut(&mut self, id: &str) -> Option<&mut FolioSection> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    /// Add setting to section
    pub fn add_setting(&mut self, section_id: &str, key: &str, value: &str) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.add(key, value);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.sections, self.config.folio_type);
    }

    /// Lock folio
    pub fn lock(&mut self) {
        self.status = FolioStatus::Locked;
    }

    /// Close folio
    pub fn close(&mut self) {
        self.status = FolioStatus::Closed;
    }

    /// Get status
    pub fn status(&self) -> FolioStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &FolioStats {
        &self.stats
    }

    /// Section count
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folio_new() {
        let f = SettingsFolio::new(FolioConfig::default());
        assert_eq!(f.section_count(), 0);
    }

    #[test]
    fn test_folio_add_section() {
        let mut f = SettingsFolio::new(FolioConfig::default());
        f.add_section("s1", "Section 1");
        assert_eq!(f.section_count(), 1);
    }

    #[test]
    fn test_folio_add_setting() {
        let mut f = SettingsFolio::new(FolioConfig::default());
        f.add_section("s1", "Section 1");
        let added = f.add_setting("s1", "key", "value");
        assert!(added);
    }

    #[test]
    fn test_folio_lock() {
        let mut f = SettingsFolio::new(FolioConfig::default());
        f.lock();
        assert_eq!(f.status(), FolioStatus::Locked);
    }
}
