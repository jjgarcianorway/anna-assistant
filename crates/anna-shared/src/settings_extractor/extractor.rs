// v0.0.653: Settings Extractor Core (Phase 229)
// Main extraction logic

use std::collections::HashMap;

use super::result::{ExtractionResult, ExtractorStats};
use super::types::{ExtractionMode, ExtractionType, ExtractorConfig};

/// Settings extractor
#[derive(Debug, Clone, Default)]
pub struct SettingsExtractor {
    /// Config
    config: ExtractorConfig,
    /// Results
    results: Vec<ExtractionResult>,
    /// Stats
    stats: ExtractorStats,
}

impl SettingsExtractor {
    /// Create new extractor
    pub fn new(config: ExtractorConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: ExtractorStats::default(),
        }
    }

    /// Extract by selector
    pub fn extract(&mut self, settings: &HashMap<String, String>, selector: &str) -> ExtractionResult {
        let mut result = ExtractionResult::new(self.config.extraction_type, selector);

        for (key, value) in settings {
            let matches = self.matches_selector(key, selector);
            if matches {
                result.add(key.clone(), value.clone());

                // Handle extraction modes
                match self.config.mode {
                    ExtractionMode::First => break,
                    ExtractionMode::All | ExtractionMode::Unique => {}
                    ExtractionMode::Last => {
                        result.values.clear();
                        result.values.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        self.stats.record(
            self.config.extraction_type,
            self.config.mode,
            result.match_count,
        );
        self.results.push(result.clone());
        result
    }

    /// Check if key matches selector
    fn matches_selector(&self, key: &str, selector: &str) -> bool {
        let (key, selector) = if self.config.case_sensitive {
            (key.to_string(), selector.to_string())
        } else {
            (key.to_lowercase(), selector.to_lowercase())
        };

        match self.config.extraction_type {
            ExtractionType::Key => key == selector,
            ExtractionType::Pattern => key.contains(&selector),
            ExtractionType::Category => key.starts_with(&selector),
            ExtractionType::Prefix => key.starts_with(&selector),
            ExtractionType::Suffix => key.ends_with(&selector),
        }
    }

    /// Get results
    pub fn results(&self) -> &[ExtractionResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &ExtractorStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}
