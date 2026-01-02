// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Compendium statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::volume::CompendiumVolume;

/// Compendium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompendiumStats {
    /// Total volumes
    pub total_volumes: usize,
    /// Total articles
    pub total_articles: usize,
    /// Total entries
    pub total_entries: usize,
    /// By subject
    pub by_subject: HashMap<String, usize>,
}

impl CompendiumStats {
    /// Update from compendium
    pub fn update(&mut self, volumes: &[CompendiumVolume]) {
        self.total_volumes = volumes.len();
        self.total_articles = volumes.iter().map(|v| v.article_count()).sum();
        self.by_subject.clear();
        for vol in volumes {
            if !vol.subject.is_empty() {
                *self.by_subject.entry(vol.subject.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Record entry
    pub fn record_entry(&mut self) {
        self.total_entries += 1;
    }

    /// Avg articles per volume
    pub fn avg_per_volume(&self) -> f64 {
        if self.total_volumes == 0 { 0.0 } else { self.total_articles as f64 / self.total_volumes as f64 }
    }
}
