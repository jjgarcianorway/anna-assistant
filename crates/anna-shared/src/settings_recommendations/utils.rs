// v0.0.578: Settings Recommendations - Utilities (Phase 154)
// Helper functions for recommendations

use super::engine::RecommendationEngine;
use super::types::RecommendationPriority;

/// Format recommendations for display
pub fn format_recommendations(engine: &RecommendationEngine) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Recommendations ===\n\n");
    output.push_str(&format!("Active: {}\n", engine.active_count()));
    output.push_str(&format!(
        "Critical: {}\n\n",
        engine.count_by_priority(RecommendationPriority::Critical)
    ));

    let active = engine.active();
    if active.is_empty() {
        output.push_str("No active recommendations. Your settings look good!\n");
        return output;
    }

    for rec in active {
        output.push_str(&format!(
            "• [{}] {} - {} ({})\n",
            rec.priority, rec.rec_type, rec.setting, rec.category
        ));
        output.push_str(&format!("  {}\n", rec.reason));
    }

    output
}

/// Check if query is about recommendations
pub fn is_recommendations_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("recommend")
        || lower.contains("suggestion")
        || lower.contains("improve settings")
        || lower.contains("optimize settings")
}

/// Fun fact about recommendations
pub fn settings_recommendations_fun_fact() -> &'static str {
    "Anna analyzes your settings to provide personalized recommendations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_recommendations() {
        let engine = RecommendationEngine::new();
        let output = format_recommendations(&engine);
        assert!(output.contains("Recommendations"));
    }

    #[test]
    fn test_is_recommendations_query() {
        assert!(is_recommendations_query("show recommendations"));
        assert!(is_recommendations_query("suggestions for settings"));
        assert!(is_recommendations_query("improve settings"));
        assert!(!is_recommendations_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_recommendations_fun_fact();
        assert!(fact.contains("recommend"));
    }
}
