// v0.0.703: Settings Repertoire (Phase 279)
// Performance repertoire of available settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Repertoire type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RepertoireType {
    /// Standard repertoire
    #[default]
    Standard,
    /// Classic repertoire
    Classic,
    /// Modern repertoire
    Modern,
    /// Experimental repertoire
    Experimental,
}

impl std::fmt::Display for RepertoireType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Classic => write!(f, "classic"),
            Self::Modern => write!(f, "modern"),
            Self::Experimental => write!(f, "experimental"),
        }
    }
}

/// Repertoire status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RepertoireStatus {
    /// Rehearsing
    #[default]
    Rehearsing,
    /// Ready
    Ready,
    /// Performing
    Performing,
    /// Retired
    Retired,
}

impl std::fmt::Display for RepertoireStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rehearsing => write!(f, "rehearsing"),
            Self::Ready => write!(f, "ready"),
            Self::Performing => write!(f, "performing"),
            Self::Retired => write!(f, "retired"),
        }
    }
}

/// Repertoire config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepertoireConfig {
    /// Name
    pub name: String,
    /// Repertoire type
    pub repertoire_type: RepertoireType,
    /// Season
    pub season: String,
    /// Max pieces
    pub max_pieces: usize,
}

impl RepertoireConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            repertoire_type: RepertoireType::Standard,
            season: String::new(),
            max_pieces: 100,
        }
    }

    /// Set type
    pub fn repertoire_type(mut self, rt: RepertoireType) -> Self {
        self.repertoire_type = rt;
        self
    }

    /// Set season
    pub fn season(mut self, season: impl Into<String>) -> Self {
        self.season = season.into();
        self
    }

    /// Set max pieces
    pub fn max_pieces(mut self, max: usize) -> Self {
        self.max_pieces = max;
        self
    }
}

impl Default for RepertoireConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Repertoire piece
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepertoirePiece {
    /// Piece ID
    pub id: String,
    /// Title
    pub title: String,
    /// Composer
    pub composer: String,
    /// Difficulty
    pub difficulty: u8,
    /// Practiced
    pub practiced: bool,
}

impl RepertoirePiece {
    /// Create new piece
    pub fn new(id: impl Into<String>, title: impl Into<String>, composer: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            composer: composer.into(),
            difficulty: 1,
            practiced: false,
        }
    }

    /// Set difficulty
    pub fn difficulty(mut self, diff: u8) -> Self {
        self.difficulty = diff.min(10);
        self
    }

    /// Mark practiced
    pub fn practiced(mut self, p: bool) -> Self {
        self.practiced = p;
        self
    }
}

/// Repertoire item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepertoireItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Piece ID
    pub piece_id: String,
    /// Performance notes
    pub notes: Option<String>,
}

impl RepertoireItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, piece_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            piece_id: piece_id.into(),
            notes: None,
        }
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Repertoire stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepertoireStats {
    /// Total pieces
    pub total_pieces: usize,
    /// Practiced pieces
    pub practiced_pieces: usize,
    /// Total items
    pub total_items: usize,
    /// Avg difficulty
    pub avg_difficulty: f64,
}

impl RepertoireStats {
    /// Update from repertoire
    pub fn update(&mut self, pieces: &[RepertoirePiece]) {
        self.total_pieces = pieces.len();
        self.practiced_pieces = pieces.iter().filter(|p| p.practiced).count();
        if self.total_pieces > 0 {
            let sum: u32 = pieces.iter().map(|p| p.difficulty as u32).sum();
            self.avg_difficulty = sum as f64 / self.total_pieces as f64;
        }
    }

    /// Record item
    pub fn record_item(&mut self) {
        self.total_items += 1;
    }

    /// Practice rate
    pub fn practice_rate(&self) -> f64 {
        if self.total_pieces == 0 { 0.0 } else { self.practiced_pieces as f64 / self.total_pieces as f64 * 100.0 }
    }
}

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

/// Repertoire registry
#[derive(Debug, Clone, Default)]
pub struct RepertoireRegistry {
    /// Repertoires by ID
    repertoires: HashMap<String, SettingsRepertoire>,
}

impl RepertoireRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register repertoire
    pub fn register(&mut self, id: impl Into<String>, repertoire: SettingsRepertoire) {
        self.repertoires.insert(id.into(), repertoire);
    }

    /// Unregister repertoire
    pub fn unregister(&mut self, id: &str) -> bool {
        self.repertoires.remove(id).is_some()
    }

    /// Get repertoire
    pub fn get(&self, id: &str) -> Option<&SettingsRepertoire> {
        self.repertoires.get(id)
    }

    /// Get repertoire mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRepertoire> {
        self.repertoires.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.repertoires.len()
    }
}

/// Format repertoire registry
pub fn format_repertoire_registry(registry: &RepertoireRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Repertoire Registry:\n");
    output.push_str(&format!("  Repertoires: {}\n", registry.count()));
    output
}

/// Check if query is about repertoire
pub fn is_repertoire_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings repertoire") || lower.contains("repertoire settings") || lower.contains("available settings")
}

/// Fun fact about repertoire
pub fn repertoire_fun_fact() -> &'static str {
    "Anna's settings repertoire performs your configurations with virtuoso precision!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repertoire_type_display() {
        assert_eq!(format!("{}", RepertoireType::Standard), "standard");
        assert_eq!(format!("{}", RepertoireType::Classic), "classic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RepertoireStatus::Rehearsing), "rehearsing");
        assert_eq!(format!("{}", RepertoireStatus::Performing), "performing");
    }

    #[test]
    fn test_config_new() {
        let c = RepertoireConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RepertoireConfig::new("test")
            .repertoire_type(RepertoireType::Modern)
            .season("2025");
        assert_eq!(c.repertoire_type, RepertoireType::Modern);
        assert_eq!(c.season, "2025");
    }

    #[test]
    fn test_piece_new() {
        let p = RepertoirePiece::new("p1", "Piece 1", "Composer");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_piece_builder() {
        let p = RepertoirePiece::new("p1", "Piece 1", "Composer")
            .difficulty(5)
            .practiced(true);
        assert_eq!(p.difficulty, 5);
        assert!(p.practiced);
    }

    #[test]
    fn test_item_new() {
        let i = RepertoireItem::new("key", "value", "p1");
        assert_eq!(i.piece_id, "p1");
    }

    #[test]
    fn test_item_notes() {
        let i = RepertoireItem::new("key", "value", "p1").notes("Performance note");
        assert!(i.notes.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = RepertoireStats::default();
        let pieces = vec![RepertoirePiece::new("p1", "Piece", "Composer").practiced(true)];
        s.update(&pieces);
        assert_eq!(s.total_pieces, 1);
        assert_eq!(s.practiced_pieces, 1);
    }

    #[test]
    fn test_repertoire_new() {
        let r = SettingsRepertoire::new(RepertoireConfig::default());
        assert_eq!(r.piece_count(), 0);
    }

    #[test]
    fn test_repertoire_add_piece() {
        let mut r = SettingsRepertoire::new(RepertoireConfig::default());
        r.add_piece(RepertoirePiece::new("p1", "Piece 1", "Composer"));
        assert_eq!(r.piece_count(), 1);
    }

    #[test]
    fn test_repertoire_ready() {
        let mut r = SettingsRepertoire::new(RepertoireConfig::default());
        r.ready();
        assert_eq!(r.status(), RepertoireStatus::Ready);
    }

    #[test]
    fn test_repertoire_perform() {
        let mut r = SettingsRepertoire::new(RepertoireConfig::default());
        r.perform();
        assert_eq!(r.status(), RepertoireStatus::Performing);
    }

    #[test]
    fn test_registry_new() {
        let r = RepertoireRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RepertoireRegistry::new();
        r.register("r1", SettingsRepertoire::new(RepertoireConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_repertoire_query() {
        assert!(is_repertoire_query("settings repertoire"));
        assert!(!is_repertoire_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = repertoire_fun_fact();
        assert!(fact.contains("repertoire"));
    }
}
