// v0.0.701: Settings Anthology (Phase 277)
// Statistics tracking for anthologies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::AnthologyWork;

/// Anthology stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnthologyStats {
    /// Total works
    pub total_works: usize,
    /// Featured works
    pub featured_works: usize,
    /// Total pieces
    pub total_pieces: usize,
    /// By author
    pub by_author: HashMap<String, usize>,
}

impl AnthologyStats {
    /// Update from anthology
    pub fn update(&mut self, works: &[AnthologyWork]) {
        self.total_works = works.len();
        self.featured_works = works.iter().filter(|w| w.featured).count();
        self.by_author.clear();
        for work in works {
            if !work.author.is_empty() {
                *self.by_author.entry(work.author.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Record piece
    pub fn record_piece(&mut self) {
        self.total_pieces += 1;
    }

    /// Featured rate
    pub fn featured_rate(&self) -> f64 {
        if self.total_works == 0 { 0.0 } else { self.featured_works as f64 / self.total_works as f64 * 100.0 }
    }
}
