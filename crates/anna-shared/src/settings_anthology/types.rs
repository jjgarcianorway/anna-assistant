// v0.0.701: Settings Anthology (Phase 277)
// Core types for anthology management

use serde::{Deserialize, Serialize};

/// Anthology type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AnthologyType {
    /// Best of anthology
    #[default]
    BestOf,
    /// Complete anthology
    Complete,
    /// Themed anthology
    Themed,
    /// Historical anthology
    Historical,
}

impl std::fmt::Display for AnthologyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BestOf => write!(f, "best_of"),
            Self::Complete => write!(f, "complete"),
            Self::Themed => write!(f, "themed"),
            Self::Historical => write!(f, "historical"),
        }
    }
}

/// Anthology status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AnthologyStatus {
    /// Curating
    #[default]
    Curating,
    /// Complete
    Complete,
    /// Published
    Published,
    /// Archived
    Archived,
}

impl std::fmt::Display for AnthologyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Curating => write!(f, "curating"),
            Self::Complete => write!(f, "complete"),
            Self::Published => write!(f, "published"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

/// Anthology config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthologyConfig {
    /// Name
    pub name: String,
    /// Anthology type
    pub anthology_type: AnthologyType,
    /// Theme
    pub theme: String,
    /// Max works
    pub max_works: usize,
}

impl AnthologyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            anthology_type: AnthologyType::BestOf,
            theme: String::new(),
            max_works: 100,
        }
    }

    /// Set type
    pub fn anthology_type(mut self, at: AnthologyType) -> Self {
        self.anthology_type = at;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    /// Set max works
    pub fn max_works(mut self, max: usize) -> Self {
        self.max_works = max;
        self
    }
}

impl Default for AnthologyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Anthology work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthologyWork {
    /// Work ID
    pub id: String,
    /// Title
    pub title: String,
    /// Author
    pub author: String,
    /// Source
    pub source: String,
    /// Featured
    pub featured: bool,
}

impl AnthologyWork {
    /// Create new work
    pub fn new(id: impl Into<String>, title: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            author: author.into(),
            source: String::new(),
            featured: false,
        }
    }

    /// Set source
    pub fn source(mut self, src: impl Into<String>) -> Self {
        self.source = src.into();
        self
    }

    /// Set featured
    pub fn featured(mut self, feat: bool) -> Self {
        self.featured = feat;
        self
    }
}

/// Anthology piece
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthologyPiece {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Work ID
    pub work_id: String,
    /// Order
    pub order: usize,
}

impl AnthologyPiece {
    /// Create new piece
    pub fn new(key: impl Into<String>, value: impl Into<String>, work_id: impl Into<String>, order: usize) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            work_id: work_id.into(),
            order,
        }
    }
}
