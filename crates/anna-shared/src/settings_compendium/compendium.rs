// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Main compendium structure

use super::config::CompendiumConfig;
use super::volume::CompendiumVolume;
use super::entry::CompendiumEntry;
use super::stats::CompendiumStats;

/// Settings compendium
#[derive(Debug, Clone, Default)]
pub struct SettingsCompendium {
    /// Config
    config: CompendiumConfig,
    /// Volumes
    volumes: Vec<CompendiumVolume>,
    /// Entries
    entries: Vec<CompendiumEntry>,
    /// Stats
    stats: CompendiumStats,
}

impl SettingsCompendium {
    /// Create new compendium
    pub fn new(config: CompendiumConfig) -> Self {
        Self {
            config,
            volumes: Vec::new(),
            entries: Vec::new(),
            stats: CompendiumStats::default(),
        }
    }

    /// Add volume
    pub fn add_volume(&mut self, volume: CompendiumVolume) -> bool {
        if self.volumes.len() >= self.config.max_volumes {
            return false;
        }
        self.volumes.push(volume);
        self.update_stats();
        true
    }

    /// Get volume
    pub fn get_volume(&self, number: usize) -> Option<&CompendiumVolume> {
        self.volumes.iter().find(|v| v.number == number)
    }

    /// Get volume mut
    pub fn get_volume_mut(&mut self, number: usize) -> Option<&mut CompendiumVolume> {
        self.volumes.iter_mut().find(|v| v.number == number)
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: CompendiumEntry) {
        self.entries.push(entry);
        self.stats.record_entry();
    }

    /// Get entries for article
    pub fn get_entries(&self, article_id: &str) -> Vec<&CompendiumEntry> {
        self.entries.iter().filter(|e| e.article_id == article_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.volumes);
    }

    /// Get stats
    pub fn stats(&self) -> &CompendiumStats {
        &self.stats
    }

    /// Volume count
    pub fn volume_count(&self) -> usize {
        self.volumes.len()
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}
