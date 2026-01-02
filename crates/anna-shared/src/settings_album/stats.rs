// v0.0.696: Settings Album (Phase 272)
// Album statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::page::AlbumPage;
use super::types::AlbumType;

/// Album stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlbumStats {
    /// Total pages
    pub total_pages: usize,
    /// Total items
    pub total_items: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AlbumStats {
    /// Update from album
    pub fn update(&mut self, pages: &[AlbumPage], album_type: AlbumType) {
        self.total_pages = pages.len();
        self.total_items = pages.iter().map(|p| p.count()).sum();
        *self.by_type.entry(album_type.to_string()).or_insert(0) += 1;
    }

    /// Avg items per page
    pub fn avg_per_page(&self) -> f64 {
        if self.total_pages == 0 { 0.0 } else { self.total_items as f64 / self.total_pages as f64 }
    }
}
