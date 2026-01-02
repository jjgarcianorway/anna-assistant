// v0.0.566: Settings Search Module
// Search and filter settings by keywords, values, or categories

mod types;
mod category_searches;
mod searcher;
mod formatting;

// Re-export all public types and functions to preserve API
pub use types::{MatchType, SearchResult, SearchResults, SearchOptions};
pub use searcher::SettingsSearcher;
pub use formatting::{format_search_results, is_search_query, settings_search_fun_fact};
