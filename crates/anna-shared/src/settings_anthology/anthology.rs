// v0.0.701: Settings Anthology (Phase 277)
// Core anthology and registry implementations

use std::collections::HashMap;

use super::stats::AnthologyStats;
use super::types::{AnthologyConfig, AnthologyPiece, AnthologyStatus, AnthologyWork};

/// Settings anthology
#[derive(Debug, Clone, Default)]
pub struct SettingsAnthology {
    /// Config
    config: AnthologyConfig,
    /// Works
    works: Vec<AnthologyWork>,
    /// Pieces
    pieces: Vec<AnthologyPiece>,
    /// Status
    status: AnthologyStatus,
    /// Stats
    stats: AnthologyStats,
}

impl SettingsAnthology {
    /// Create new anthology
    pub fn new(config: AnthologyConfig) -> Self {
        Self {
            config,
            works: Vec::new(),
            pieces: Vec::new(),
            status: AnthologyStatus::Curating,
            stats: AnthologyStats::default(),
        }
    }

    /// Add work
    pub fn add_work(&mut self, work: AnthologyWork) -> bool {
        if self.works.len() >= self.config.max_works {
            return false;
        }
        self.works.push(work);
        self.update_stats();
        true
    }

    /// Get work
    pub fn get_work(&self, id: &str) -> Option<&AnthologyWork> {
        self.works.iter().find(|w| w.id == id)
    }

    /// Add piece
    pub fn add_piece(&mut self, piece: AnthologyPiece) {
        self.pieces.push(piece);
        self.stats.record_piece();
    }

    /// Get pieces for work
    pub fn get_pieces(&self, work_id: &str) -> Vec<&AnthologyPiece> {
        let mut result: Vec<_> = self.pieces.iter().filter(|p| p.work_id == work_id).collect();
        result.sort_by_key(|p| p.order);
        result
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.works);
    }

    /// Complete
    pub fn complete(&mut self) {
        self.status = AnthologyStatus::Complete;
    }

    /// Publish
    pub fn publish(&mut self) {
        self.status = AnthologyStatus::Published;
    }

    /// Archive
    pub fn archive(&mut self) {
        self.status = AnthologyStatus::Archived;
    }

    /// Get status
    pub fn status(&self) -> AnthologyStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &AnthologyStats {
        &self.stats
    }

    /// Work count
    pub fn work_count(&self) -> usize {
        self.works.len()
    }
}

/// Anthology registry
#[derive(Debug, Clone, Default)]
pub struct AnthologyRegistry {
    /// Anthologies by ID
    anthologies: HashMap<String, SettingsAnthology>,
}

impl AnthologyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register anthology
    pub fn register(&mut self, id: impl Into<String>, anthology: SettingsAnthology) {
        self.anthologies.insert(id.into(), anthology);
    }

    /// Unregister anthology
    pub fn unregister(&mut self, id: &str) -> bool {
        self.anthologies.remove(id).is_some()
    }

    /// Get anthology
    pub fn get(&self, id: &str) -> Option<&SettingsAnthology> {
        self.anthologies.get(id)
    }

    /// Get anthology mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAnthology> {
        self.anthologies.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.anthologies.len()
    }
}
