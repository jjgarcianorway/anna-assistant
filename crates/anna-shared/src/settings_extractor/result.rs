// v0.0.653: Extraction Result and Stats (Phase 229)
// Results and statistics for extraction operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{ExtractionMode, ExtractionType};

/// Extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Extracted values
    pub values: HashMap<String, String>,
    /// Extraction type used
    pub extraction_type: ExtractionType,
    /// Pattern/key used
    pub selector: String,
    /// Match count
    pub match_count: usize,
}

impl ExtractionResult {
    /// Create new result
    pub fn new(extraction_type: ExtractionType, selector: impl Into<String>) -> Self {
        Self {
            values: HashMap::new(),
            extraction_type,
            selector: selector.into(),
            match_count: 0,
        }
    }

    /// Add extracted value
    pub fn add(&mut self, key: String, value: String) {
        self.values.insert(key, value);
        self.match_count += 1;
    }

    /// Has matches
    pub fn has_matches(&self) -> bool {
        self.match_count > 0
    }

    /// Value count
    pub fn value_count(&self) -> usize {
        self.values.len()
    }
}

/// Extractor stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractorStats {
    /// Total extractions
    pub total_extractions: usize,
    /// Total matches
    pub total_matches: usize,
    /// By extraction type
    pub by_type: HashMap<String, usize>,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl ExtractorStats {
    /// Record extraction
    pub fn record(&mut self, extraction_type: ExtractionType, mode: ExtractionMode, matches: usize) {
        self.total_extractions += 1;
        self.total_matches += matches;
        *self.by_type.entry(extraction_type.to_string()).or_insert(0) += 1;
        *self.by_mode.entry(mode.to_string()).or_insert(0) += 1;
    }

    /// Average matches
    pub fn average_matches(&self) -> f64 {
        if self.total_extractions == 0 {
            0.0
        } else {
            self.total_matches as f64 / self.total_extractions as f64
        }
    }
}
