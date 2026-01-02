// v0.0.566: Settings Searcher
// Main search implementation and orchestration

use crate::unified_settings::{SettingsCategory, UnifiedSettings};
use super::types::{SearchOptions, SearchResults};
use super::category_searches::CategorySearcher;

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
            CategorySearcher::search_personality(
                settings,
                &query_normalized,
                &mut results,
                self.options.case_sensitive,
                self.options.search_names,
                self.options.search_values,
            );
        }

        // Search risk settings
        if self.should_search_category(SettingsCategory::Risk) {
            CategorySearcher::search_risk(
                settings,
                &query_normalized,
                &mut results,
                self.options.case_sensitive,
                self.options.search_names,
                self.options.search_values,
            );
        }

        // Search learning settings
        if self.should_search_category(SettingsCategory::Learning) {
            CategorySearcher::search_learning(
                settings,
                &query_normalized,
                &mut results,
                self.options.case_sensitive,
                self.options.search_names,
                self.options.search_values,
            );
        }

        // Search verbosity settings
        if self.should_search_category(SettingsCategory::Verbosity) {
            CategorySearcher::search_verbosity(
                settings,
                &query_normalized,
                &mut results,
                self.options.case_sensitive,
                self.options.search_names,
                self.options.search_values,
            );
        }

        // Search timeout settings
        if self.should_search_category(SettingsCategory::Timeout) {
            CategorySearcher::search_timeout(
                settings,
                &query_normalized,
                &mut results,
                self.options.case_sensitive,
                self.options.search_names,
                self.options.search_values,
            );
        }

        // Search privacy settings
        if self.should_search_category(SettingsCategory::Privacy) {
            CategorySearcher::search_privacy(
                settings,
                &query_normalized,
                &mut results,
                self.options.case_sensitive,
                self.options.search_names,
                self.options.search_values,
            );
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_settings() -> UnifiedSettings {
        UnifiedSettings::default()
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
}
