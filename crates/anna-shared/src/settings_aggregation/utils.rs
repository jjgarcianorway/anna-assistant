// v0.0.671: Settings Aggregation - Utility Functions
// Utility functions for settings aggregation

/// Check if query is about aggregator
pub fn is_aggregator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("aggregate") || lower.contains("group by") || lower.contains("sum settings")
}

/// Fun fact about aggregator
pub fn aggregator_fun_fact() -> &'static str {
    "Anna's settings aggregator can group and summarize settings with various functions!"
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
        assert!(fact.contains("aggregator"));
    }
}
