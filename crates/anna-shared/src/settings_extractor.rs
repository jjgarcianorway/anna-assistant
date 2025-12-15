// v0.0.653: Settings Extractor (Phase 229)
// Extractor for pulling specific settings from configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Settings extractor registry
#[derive(Debug, Clone, Default)]
pub struct SettingsExtractorRegistry {
    /// Extractors by ID
    extractors: HashMap<String, SettingsExtractor>,
}

impl SettingsExtractorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register extractor
    pub fn register(&mut self, id: impl Into<String>, extractor: SettingsExtractor) {
        self.extractors.insert(id.into(), extractor);
    }

    /// Unregister extractor
    pub fn unregister(&mut self, id: &str) -> bool {
        self.extractors.remove(id).is_some()
    }

    /// Get extractor
    pub fn get(&self, id: &str) -> Option<&SettingsExtractor> {
        self.extractors.get(id)
    }

    /// Get extractor mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsExtractor> {
        self.extractors.get_mut(id)
    }

    /// Extractor count
    pub fn count(&self) -> usize {
        self.extractors.len()
    }
}

/// Format extractor registry
pub fn format_extractor_registry(registry: &SettingsExtractorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Extractor Registry:\n");
    output.push_str(&format!("  Extractors: {}\n", registry.count()));
    output
}

/// Check if query is about extractor
pub fn is_extractor_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("extractor") || lower.contains("extract settings") || lower.contains("pull settings")
}

/// Fun fact about extractor
pub fn extractor_fun_fact() -> &'static str {
    "Anna's settings extractors pull specific configs from any source!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_type_display() {
        assert_eq!(format!("{}", ExtractionType::Key), "key");
        assert_eq!(format!("{}", ExtractionType::Pattern), "pattern");
    }

    #[test]
    fn test_extraction_mode_display() {
        assert_eq!(format!("{}", ExtractionMode::First), "first");
        assert_eq!(format!("{}", ExtractionMode::All), "all");
    }

    #[test]
    fn test_config_new() {
        let c = ExtractorConfig::new(ExtractionType::Key);
        assert!(c.case_sensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = ExtractorConfig::new(ExtractionType::Pattern)
            .mode(ExtractionMode::First)
            .case_sensitive(false);
        assert_eq!(c.mode, ExtractionMode::First);
        assert!(!c.case_sensitive);
    }

    #[test]
    fn test_result_new() {
        let r = ExtractionResult::new(ExtractionType::Key, "test");
        assert!(!r.has_matches());
    }

    #[test]
    fn test_result_add() {
        let mut r = ExtractionResult::new(ExtractionType::Key, "test");
        r.add("key".to_string(), "value".to_string());
        assert!(r.has_matches());
        assert_eq!(r.match_count, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ExtractorStats::default();
        s.record(ExtractionType::Key, ExtractionMode::All, 5);
        s.record(ExtractionType::Pattern, ExtractionMode::First, 2);
        assert_eq!(s.total_extractions, 2);
        assert_eq!(s.total_matches, 7);
    }

    #[test]
    fn test_extractor_new() {
        let e = SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Key));
        assert_eq!(e.result_count(), 0);
    }

    #[test]
    fn test_extractor_extract_key() {
        let mut e = SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Key));
        let mut settings = HashMap::new();
        settings.insert("mykey".to_string(), "myvalue".to_string());

        let r = e.extract(&settings, "mykey");
        assert!(r.has_matches());
        assert_eq!(r.values.get("mykey"), Some(&"myvalue".to_string()));
    }

    #[test]
    fn test_extractor_extract_prefix() {
        let mut e = SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Prefix));
        let mut settings = HashMap::new();
        settings.insert("app_name".to_string(), "test".to_string());
        settings.insert("app_version".to_string(), "1.0".to_string());

        let r = e.extract(&settings, "app_");
        assert_eq!(r.match_count, 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsExtractorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsExtractorRegistry::new();
        r.register("ext1", SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Key)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_extractor_query() {
        assert!(is_extractor_query("settings extractor"));
        assert!(!is_extractor_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = extractor_fun_fact();
        assert!(fact.contains("extractor"));
    }
}
