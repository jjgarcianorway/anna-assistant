// v0.0.600: Settings Aggregator Utilities (Phase 176)
// Utility functions for settings aggregation

use super::aggregator::SettingsAggregator;

/// Format aggregator
pub fn format_aggregator(agg: &SettingsAggregator) -> String {
    let mut output = String::new();
    output.push_str("Settings Aggregator:\n");
    output.push_str(&format!("  Aggregations: {}\n", agg.count()));
    output.push_str(&format!("  Cached results: {}\n", agg.result_count()));

    for id in agg.list_ids() {
        if let Some(def) = agg.get(id) {
            output.push_str(&format!("  - {} [{}] {}\n", id, def.agg_type, def.name));
        }
    }

    output
}

/// Check if query is about aggregator
pub fn is_aggregator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("aggregate")
        || lower.contains("summarize settings")
        || lower.contains("settings summary")
}

/// Fun fact about aggregator
pub fn aggregator_fun_fact() -> &'static str {
    "Anna can aggregate and summarize your settings across all categories!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_aggregator_query() {
        assert!(is_aggregator_query("aggregate settings"));
        assert!(!is_aggregator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = aggregator_fun_fact();
        assert!(fact.contains("aggregate"));
    }
}
