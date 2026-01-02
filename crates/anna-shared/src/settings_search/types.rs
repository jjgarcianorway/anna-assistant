// v0.0.566: Settings Search Types
// Type definitions for search results, options, and match types

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
