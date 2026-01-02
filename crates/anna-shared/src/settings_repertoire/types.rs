// v0.0.703: Settings Repertoire Types (Phase 279)
// Type definitions for settings repertoire

use serde::{Deserialize, Serialize};

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
