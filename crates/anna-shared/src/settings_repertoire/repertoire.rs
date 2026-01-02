// v0.0.703: Settings Repertoire Core (Phase 279)
// Main SettingsRepertoire implementation

use super::types::{RepertoireConfig, RepertoireItem, RepertoirePiece, RepertoireStats, RepertoireStatus};

/// Settings repertoire
#[derive(Debug, Clone, Default)]
pub struct SettingsRepertoire {
    /// Config
    config: RepertoireConfig,
    /// Pieces
    pieces: Vec<RepertoirePiece>,
    /// Items
    items: Vec<RepertoireItem>,
    /// Status
    status: RepertoireStatus,
    /// Stats
    stats: RepertoireStats,
}

impl SettingsRepertoire {
    /// Create new repertoire
    pub fn new(config: RepertoireConfig) -> Self {
        Self {
            config,
            pieces: Vec::new(),
            items: Vec::new(),
            status: RepertoireStatus::Rehearsing,
            stats: RepertoireStats::default(),
        }
    }

    /// Add piece
    pub fn add_piece(&mut self, piece: RepertoirePiece) -> bool {
        if self.pieces.len() >= self.config.max_pieces {
            return false;
        }
        self.pieces.push(piece);
        self.update_stats();
        true
    }

    /// Get piece
    pub fn get_piece(&self, id: &str) -> Option<&RepertoirePiece> {
        self.pieces.iter().find(|p| p.id == id)
    }

    /// Add item
    pub fn add_item(&mut self, item: RepertoireItem) {
        self.items.push(item);
        self.stats.record_item();
    }

    /// Get items for piece
    pub fn get_items(&self, piece_id: &str) -> Vec<&RepertoireItem> {
        self.items.iter().filter(|i| i.piece_id == piece_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.pieces);
    }

    /// Ready to perform
    pub fn ready(&mut self) {
        self.status = RepertoireStatus::Ready;
    }

    /// Start performing
    pub fn perform(&mut self) {
        self.status = RepertoireStatus::Performing;
    }

    /// Retire
    pub fn retire(&mut self) {
        self.status = RepertoireStatus::Retired;
    }

    /// Get status
    pub fn status(&self) -> RepertoireStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &RepertoireStats {
        &self.stats
    }

    /// Piece count
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }
}
