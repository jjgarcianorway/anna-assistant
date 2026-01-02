// v0.0.696: Settings Album (Phase 272)
// Main settings album implementation

use super::config::AlbumConfig;
use super::page::{AlbumPage, AlbumItem};
use super::stats::AlbumStats;
use super::types::AlbumStatus;

/// Settings album
#[derive(Debug, Clone, Default)]
pub struct SettingsAlbum {
    /// Config
    config: AlbumConfig,
    /// Pages
    pages: Vec<AlbumPage>,
    /// Status
    status: AlbumStatus,
    /// Stats
    stats: AlbumStats,
}

impl SettingsAlbum {
    /// Create new album
    pub fn new(config: AlbumConfig) -> Self {
        Self {
            config,
            pages: Vec::new(),
            status: AlbumStatus::Empty,
            stats: AlbumStats::default(),
        }
    }

    /// Add page
    pub fn add_page(&mut self, title: &str) -> bool {
        if self.pages.len() >= self.config.max_pages {
            return false;
        }
        let number = self.pages.len() + 1;
        self.pages.push(AlbumPage::new(number, title));
        self.update_status();
        self.update_stats();
        true
    }

    /// Get page
    pub fn get_page(&self, number: usize) -> Option<&AlbumPage> {
        self.pages.iter().find(|p| p.number == number)
    }

    /// Get page mut
    pub fn get_page_mut(&mut self, number: usize) -> Option<&mut AlbumPage> {
        self.pages.iter_mut().find(|p| p.number == number)
    }

    /// Add item to page
    pub fn add_item(&mut self, page_number: usize, item: AlbumItem) -> bool {
        if let Some(page) = self.get_page_mut(page_number) {
            page.add(item);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update status
    fn update_status(&mut self) {
        self.status = if self.pages.is_empty() {
            AlbumStatus::Empty
        } else if self.pages.len() < self.config.max_pages {
            AlbumStatus::Partial
        } else {
            AlbumStatus::Complete
        };
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.pages, self.config.album_type);
    }

    /// Seal album
    pub fn seal(&mut self) {
        self.status = AlbumStatus::Sealed;
    }

    /// Get status
    pub fn status(&self) -> AlbumStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &AlbumStats {
        &self.stats
    }

    /// Page count
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}
