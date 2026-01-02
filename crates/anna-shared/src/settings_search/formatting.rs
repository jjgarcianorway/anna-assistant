// v0.0.566: Settings Search Formatting and Utilities
// Formatting functions and helper utilities for search results

use super::types::SearchResults;

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
    use crate::unified_settings::UnifiedSettings;
    use crate::settings_search::SettingsSearcher;

    #[test]
    fn test_format_search_results() {
        let settings = UnifiedSettings::default();
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
