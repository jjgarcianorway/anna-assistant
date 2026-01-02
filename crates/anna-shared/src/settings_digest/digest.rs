// v0.0.709: Settings Digest Core (Phase 285)
// Main digest structure and operations

use super::config::DigestConfig;
use super::section::{DigestSection, DigestItem};
use super::stats::DigestStats;

/// Settings digest
#[derive(Debug, Clone, Default)]
pub struct SettingsDigest {
    /// Config
    config: DigestConfig,
    /// Sections
    sections: Vec<DigestSection>,
    /// Stats
    stats: DigestStats,
}

impl SettingsDigest {
    /// Create new digest
    pub fn new(config: DigestConfig) -> Self {
        Self {
            config,
            sections: Vec::new(),
            stats: DigestStats::default(),
        }
    }

    /// Add section
    pub fn add_section(&mut self, section: DigestSection) -> bool {
        if self.sections.len() >= self.config.max_sections {
            return false;
        }
        self.sections.push(section);
        self.update_stats();
        true
    }

    /// Get section
    pub fn get_section(&self, id: &str) -> Option<&DigestSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// Get section mut
    pub fn get_section_mut(&mut self, id: &str) -> Option<&mut DigestSection> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    /// Add item to section
    pub fn add_item(&mut self, section_id: &str, item: DigestItem) -> bool {
        if let Some(section) = self.get_section_mut(section_id) {
            section.add(item);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.sections, self.config.format);
    }

    /// Get stats
    pub fn stats(&self) -> &DigestStats {
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
    fn test_digest_new() {
        let d = SettingsDigest::new(DigestConfig::default());
        assert_eq!(d.section_count(), 0);
    }

    #[test]
    fn test_digest_add_section() {
        let mut d = SettingsDigest::new(DigestConfig::default());
        d.add_section(DigestSection::new("s1", "Section 1", 1));
        assert_eq!(d.section_count(), 1);
    }

    #[test]
    fn test_digest_add_item() {
        let mut d = SettingsDigest::new(DigestConfig::default());
        d.add_section(DigestSection::new("s1", "Section 1", 1));
        let added = d.add_item("s1", DigestItem::new("key", "value"));
        assert!(added);
    }
}
