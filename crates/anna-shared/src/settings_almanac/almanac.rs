// v0.0.705: Settings Almanac (Phase 281)
// Main almanac structure

use crate::settings_almanac::chapter::{AlmanacChapter, AlmanacEntry};
use crate::settings_almanac::config::AlmanacConfig;
use crate::settings_almanac::stats::AlmanacStats;
use crate::settings_almanac::types::AlmanacEdition;

/// Settings almanac
#[derive(Debug, Clone, Default)]
pub struct SettingsAlmanac {
    /// Config
    config: AlmanacConfig,
    /// Chapters
    chapters: Vec<AlmanacChapter>,
    /// Edition
    edition: AlmanacEdition,
    /// Stats
    stats: AlmanacStats,
}

impl SettingsAlmanac {
    /// Create new almanac
    pub fn new(config: AlmanacConfig) -> Self {
        Self {
            config,
            chapters: Vec::new(),
            edition: AlmanacEdition::Current,
            stats: AlmanacStats::default(),
        }
    }

    /// Add chapter
    pub fn add_chapter(&mut self, chapter: AlmanacChapter) -> bool {
        if self.chapters.len() >= self.config.max_chapters {
            return false;
        }
        self.chapters.push(chapter);
        self.update_stats();
        true
    }

    /// Get chapter
    pub fn get_chapter(&self, number: usize) -> Option<&AlmanacChapter> {
        self.chapters.iter().find(|c| c.number == number)
    }

    /// Get chapter mut
    pub fn get_chapter_mut(&mut self, number: usize) -> Option<&mut AlmanacChapter> {
        self.chapters.iter_mut().find(|c| c.number == number)
    }

    /// Add entry to chapter
    pub fn add_entry(&mut self, chapter_number: usize, entry: AlmanacEntry) -> bool {
        if let Some(chapter) = self.get_chapter_mut(chapter_number) {
            chapter.add(entry);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.chapters);
    }

    /// Set edition
    pub fn set_edition(&mut self, edition: AlmanacEdition) {
        self.edition = edition;
    }

    /// Get edition
    pub fn edition(&self) -> AlmanacEdition {
        self.edition
    }

    /// Get stats
    pub fn stats(&self) -> &AlmanacStats {
        &self.stats
    }

    /// Chapter count
    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }
}
