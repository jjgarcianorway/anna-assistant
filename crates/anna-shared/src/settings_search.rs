// v0.0.566: Settings Search (Phase 142)
// Search and filter settings by keywords, values, or categories

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Search result match type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    /// Field name match
    FieldName,
    /// Field value match
    FieldValue,
    /// Category name match
    CategoryName,
    /// Description match
    Description,
}

impl std::fmt::Display for MatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldName => write!(f, "field"),
            Self::FieldValue => write!(f, "value"),
            Self::CategoryName => write!(f, "category"),
            Self::Description => write!(f, "description"),
        }
    }
}

/// A single search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Category of the match
    pub category: SettingsCategory,
    /// Field name
    pub field: String,
    /// Current value
    pub value: String,
    /// Type of match
    pub match_type: MatchType,
    /// Relevance score (0.0-1.0)
    pub score: f32,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        category: SettingsCategory,
        field: impl Into<String>,
        value: impl Into<String>,
        match_type: MatchType,
        score: f32,
    ) -> Self {
        Self {
            category,
            field: field.into(),
            value: value.into(),
            match_type,
            score,
        }
    }
}

/// Search results container
#[derive(Debug, Clone, Default)]
pub struct SearchResults {
    /// Found matches
    pub matches: Vec<SearchResult>,
    /// Search query
    pub query: String,
    /// Search took (ms)
    pub took_ms: u64,
}

impl SearchResults {
    /// Create new results
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            matches: Vec::new(),
            query: query.into(),
            took_ms: 0,
        }
    }

    /// Add a result
    pub fn add(&mut self, result: SearchResult) {
        self.matches.push(result);
    }

    /// Get count
    pub fn count(&self) -> usize {
        self.matches.len()
    }

    /// Is empty?
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    /// Sort by score
    pub fn sort_by_score(&mut self) {
        self.matches.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Filter by category
    pub fn by_category(&self, category: SettingsCategory) -> Vec<&SearchResult> {
        self.matches.iter().filter(|r| r.category == category).collect()
    }

    /// Filter by match type
    pub fn by_match_type(&self, match_type: MatchType) -> Vec<&SearchResult> {
        self.matches.iter().filter(|r| r.match_type == match_type).collect()
    }
}

/// Search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Case sensitive search
    pub case_sensitive: bool,
    /// Search in field names
    pub search_names: bool,
    /// Search in values
    pub search_values: bool,
    /// Search in descriptions
    pub search_descriptions: bool,
    /// Limit to categories
    pub categories: Option<Vec<SettingsCategory>>,
    /// Maximum results
    pub max_results: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            search_names: true,
            search_values: true,
            search_descriptions: true,
            categories: None,
            max_results: 50,
        }
    }
}

impl SearchOptions {
    /// Create new options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case sensitive
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Limit to specific categories
    pub fn in_categories(mut self, cats: Vec<SettingsCategory>) -> Self {
        self.categories = Some(cats);
        self
    }

    /// Set max results
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }
}

/// Settings searcher
#[derive(Debug, Clone, Default)]
pub struct SettingsSearcher {
    /// Search options
    pub options: SearchOptions,
}

impl SettingsSearcher {
    /// Create new searcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure options
    pub fn with_options(mut self, options: SearchOptions) -> Self {
        self.options = options;
        self
    }

    /// Search settings
    pub fn search(&self, settings: &UnifiedSettings, query: &str) -> SearchResults {
        let start = std::time::Instant::now();
        let mut results = SearchResults::new(query);

        let query_normalized = if self.options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        // Search personality settings
        if self.should_search_category(SettingsCategory::Personality) {
            self.search_personality(settings, &query_normalized, &mut results);
        }

        // Search risk settings
        if self.should_search_category(SettingsCategory::Risk) {
            self.search_risk(settings, &query_normalized, &mut results);
        }

        // Search learning settings
        if self.should_search_category(SettingsCategory::Learning) {
            self.search_learning(settings, &query_normalized, &mut results);
        }

        // Search verbosity settings
        if self.should_search_category(SettingsCategory::Verbosity) {
            self.search_verbosity(settings, &query_normalized, &mut results);
        }

        // Search timeout settings
        if self.should_search_category(SettingsCategory::Timeout) {
            self.search_timeout(settings, &query_normalized, &mut results);
        }

        // Search privacy settings
        if self.should_search_category(SettingsCategory::Privacy) {
            self.search_privacy(settings, &query_normalized, &mut results);
        }

        results.sort_by_score();
        results.took_ms = start.elapsed().as_millis() as u64;

        // Limit results
        if results.matches.len() > self.options.max_results {
            results.matches.truncate(self.options.max_results);
        }

        results
    }

    /// Check if should search category
    fn should_search_category(&self, category: SettingsCategory) -> bool {
        match &self.options.categories {
            Some(cats) => cats.contains(&category),
            None => true,
        }
    }

    /// Check if text matches query
    fn matches(&self, text: &str, query: &str) -> Option<f32> {
        let text_normalized = if self.options.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        if text_normalized == query {
            return Some(1.0);
        }
        if text_normalized.contains(query) {
            return Some(0.7);
        }
        if text_normalized.starts_with(query) {
            return Some(0.8);
        }

        None
    }

    fn search_personality(&self, settings: &UnifiedSettings, query: &str, results: &mut SearchResults) {
        let p = &settings.personality;

        if self.options.search_names {
            if let Some(score) = self.matches("formality", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "formality",
                    format!("{:?}", p.formality), MatchType::FieldName, score
                ));
            }
            if let Some(score) = self.matches("friendliness", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "friendliness",
                    format!("{:?}", p.friendliness), MatchType::FieldName, score
                ));
            }
            if let Some(score) = self.matches("humor", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "humor",
                    format!("{:?}", p.humor), MatchType::FieldName, score
                ));
            }
        }

        if self.options.search_values {
            let formality_str = format!("{:?}", p.formality).to_lowercase();
            if let Some(score) = self.matches(&formality_str, query) {
                results.add(SearchResult::new(
                    SettingsCategory::Personality, "formality", formality_str, MatchType::FieldValue, score
                ));
            }
        }
    }

    fn search_risk(&self, settings: &UnifiedSettings, query: &str, results: &mut SearchResults) {
        let r = &settings.risk;

        if self.options.search_names {
            if let Some(score) = self.matches("risk", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Risk, "auto_approve_up_to",
                    format!("{:?}", r.auto_approve_up_to), MatchType::FieldName, score
                ));
            }
            if let Some(score) = self.matches("confirmation", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Risk, "confirmation_mode",
                    format!("{:?}", r.confirmation_mode), MatchType::FieldName, score
                ));
            }
        }
    }

    fn search_learning(&self, settings: &UnifiedSettings, query: &str, results: &mut SearchResults) {
        let l = &settings.learning;

        if self.options.search_names {
            if let Some(score) = self.matches("learning", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Learning, "level",
                    format!("{:?}", l.level), MatchType::FieldName, score
                ));
            }
            if let Some(score) = self.matches("explain", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Learning, "explain_commands",
                    l.explain_commands.to_string(), MatchType::FieldName, score
                ));
            }
        }
    }

    fn search_verbosity(&self, settings: &UnifiedSettings, query: &str, results: &mut SearchResults) {
        let v = &settings.verbosity;

        if self.options.search_names {
            if let Some(score) = self.matches("verbosity", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Verbosity, "level",
                    format!("{:?}", v.level), MatchType::FieldName, score
                ));
            }
            if let Some(score) = self.matches("detail", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Verbosity, "answer_detail",
                    format!("{:?}", v.answer_detail), MatchType::FieldName, score
                ));
            }
        }
    }

    fn search_timeout(&self, settings: &UnifiedSettings, query: &str, results: &mut SearchResults) {
        let t = &settings.timeout;

        if self.options.search_names {
            if let Some(score) = self.matches("timeout", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Timeout, "command_timeout_ms",
                    t.command_timeout_ms.to_string(), MatchType::FieldName, score
                ));
            }
        }
    }

    fn search_privacy(&self, settings: &UnifiedSettings, query: &str, results: &mut SearchResults) {
        let p = &settings.privacy;

        if self.options.search_names {
            if let Some(score) = self.matches("privacy", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Privacy, "data_collection",
                    format!("{:?}", p.data_collection), MatchType::FieldName, score
                ));
            }
            if let Some(score) = self.matches("log", query) {
                results.add(SearchResult::new(
                    SettingsCategory::Privacy, "log_retention",
                    format!("{:?}", p.log_retention), MatchType::FieldName, score
                ));
            }
        }
    }
}

/// Format search results for display
pub fn format_search_results(results: &SearchResults) -> String {
    let mut output = String::new();

    output.push_str(&format!("=== Search: \"{}\" ===\n\n", results.query));

    if results.is_empty() {
        output.push_str("No matches found.\n");
        return output;
    }

    output.push_str(&format!("Found {} match(es) in {}ms\n\n", results.count(), results.took_ms));

    for result in &results.matches {
        output.push_str(&format!(
            "• [{}] {}.{} = {}\n  Match: {} (score: {:.1})\n\n",
            result.category,
            format!("{:?}", result.category).to_lowercase(),
            result.field,
            result.value,
            result.match_type,
            result.score
        ));
    }

    output
}

/// Check if query is a search request
pub fn is_search_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("search settings")
        || lower.contains("find setting")
        || lower.contains("where is")
        || lower.contains("which setting")
        || lower.contains("look for")
}

/// Fun fact about settings search
pub fn settings_search_fun_fact() -> &'static str {
    "Anna can search through all settings to find what you're looking for instantly!"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_settings() -> UnifiedSettings {
        UnifiedSettings::default()
    }

    #[test]
    fn test_match_type_display() {
        assert_eq!(format!("{}", MatchType::FieldName), "field");
        assert_eq!(format!("{}", MatchType::FieldValue), "value");
    }

    #[test]
    fn test_search_result_new() {
        let result = SearchResult::new(
            SettingsCategory::Personality,
            "tone",
            "friendly",
            MatchType::FieldName,
            0.8,
        );
        assert_eq!(result.field, "tone");
        assert_eq!(result.score, 0.8);
    }

    #[test]
    fn test_search_results_new() {
        let results = SearchResults::new("test");
        assert!(results.is_empty());
        assert_eq!(results.query, "test");
    }

    #[test]
    fn test_search_results_add() {
        let mut results = SearchResults::new("test");
        results.add(SearchResult::new(
            SettingsCategory::Privacy, "field", "value", MatchType::FieldName, 0.5
        ));
        assert_eq!(results.count(), 1);
    }

    #[test]
    fn test_search_options_default() {
        let opts = SearchOptions::default();
        assert!(!opts.case_sensitive);
        assert!(opts.search_names);
        assert!(opts.search_values);
    }

    #[test]
    fn test_search_options_builder() {
        let opts = SearchOptions::new()
            .case_sensitive(true)
            .max_results(10);
        assert!(opts.case_sensitive);
        assert_eq!(opts.max_results, 10);
    }

    #[test]
    fn test_settings_searcher_new() {
        let searcher = SettingsSearcher::new();
        assert!(!searcher.options.case_sensitive);
    }

    #[test]
    fn test_search_personality() {
        let settings = sample_settings();
        let searcher = SettingsSearcher::new();
        let results = searcher.search(&settings, "formality");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_risk() {
        let settings = sample_settings();
        let searcher = SettingsSearcher::new();
        let results = searcher.search(&settings, "risk");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_no_results() {
        let settings = sample_settings();
        let searcher = SettingsSearcher::new();
        let results = searcher.search(&settings, "xyznonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_format_search_results() {
        let settings = sample_settings();
        let searcher = SettingsSearcher::new();
        let results = searcher.search(&settings, "privacy");
        let output = format_search_results(&results);
        assert!(output.contains("Search"));
    }

    #[test]
    fn test_is_search_query() {
        assert!(is_search_query("search settings for timeout"));
        assert!(is_search_query("find setting"));
        assert!(!is_search_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_search_fun_fact();
        assert!(fact.contains("search"));
    }
}
