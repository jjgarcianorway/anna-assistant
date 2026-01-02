// v0.0.653: Settings Extractor Types (Phase 229)
// Core types for settings extraction

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Extraction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ExtractionType {
    /// Key-based extraction
    #[default]
    Key,
    /// Pattern-based extraction
    Pattern,
    /// Category-based extraction
    Category,
    /// Prefix-based extraction
    Prefix,
    /// Suffix-based extraction
    Suffix,
}

impl std::fmt::Display for ExtractionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key => write!(f, "key"),
            Self::Pattern => write!(f, "pattern"),
            Self::Category => write!(f, "category"),
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
        }
    }
}

/// Extraction mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExtractionMode {
    /// Extract first match
    #[default]
    First,
    /// Extract all matches
    All,
    /// Extract last match
    Last,
    /// Extract unique matches
    Unique,
}

impl std::fmt::Display for ExtractionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::All => write!(f, "all"),
            Self::Last => write!(f, "last"),
            Self::Unique => write!(f, "unique"),
        }
    }
}

/// Extractor config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    /// Extraction type
    pub extraction_type: ExtractionType,
    /// Extraction mode
    pub mode: ExtractionMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Case sensitive
    pub case_sensitive: bool,
    /// Include defaults
    pub include_defaults: bool,
}

impl ExtractorConfig {
    /// Create new config
    pub fn new(extraction_type: ExtractionType) -> Self {
        Self {
            extraction_type,
            mode: ExtractionMode::All,
            category: None,
            case_sensitive: true,
            include_defaults: false,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: ExtractionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set case sensitive
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Set include defaults
    pub fn include_defaults(mut self, include: bool) -> Self {
        self.include_defaults = include;
        self
    }
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self::new(ExtractionType::Key)
    }
}
