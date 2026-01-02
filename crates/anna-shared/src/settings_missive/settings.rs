// v0.0.716: Settings Missive (Phase 292)
// Main settings missive implementation

use super::config::MissiveConfig;
use super::letter::MissiveLetter;
use super::enclosure::MissiveEnclosure;
use super::stats::MissiveStats;

/// Settings missive
#[derive(Debug, Clone, Default)]
pub struct SettingsMissive {
    /// Config
    config: MissiveConfig,
    /// Letters
    letters: Vec<MissiveLetter>,
    /// Enclosures
    enclosures: Vec<MissiveEnclosure>,
    /// Stats
    stats: MissiveStats,
}

impl SettingsMissive {
    /// Create new missive system
    pub fn new(config: MissiveConfig) -> Self {
        Self {
            config,
            letters: Vec::new(),
            enclosures: Vec::new(),
            stats: MissiveStats::default(),
        }
    }

    /// Add letter
    pub fn add_letter(&mut self, letter: MissiveLetter) -> bool {
        if self.letters.len() >= self.config.max_missives {
            return false;
        }
        self.letters.push(letter);
        self.update_stats();
        true
    }

    /// Get letter
    pub fn get_letter(&self, id: &str) -> Option<&MissiveLetter> {
        self.letters.iter().find(|l| l.id == id)
    }

    /// Get letter mut
    pub fn get_letter_mut(&mut self, id: &str) -> Option<&mut MissiveLetter> {
        self.letters.iter_mut().find(|l| l.id == id)
    }

    /// Add enclosure
    pub fn add_enclosure(&mut self, enclosure: MissiveEnclosure) {
        self.enclosures.push(enclosure);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.letters, self.config.missive_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MissiveStats {
        &self.stats
    }

    /// Letter count
    pub fn letter_count(&self) -> usize {
        self.letters.len()
    }
}
