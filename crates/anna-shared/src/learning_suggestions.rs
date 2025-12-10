//! Idle-time learning suggestions (v0.0.282).
//!
//! Suggests areas where Anna could improve her knowledge during idle time.
//! These suggestions help users understand what Anna is learning and
//! encourage exploration of new capabilities.
//!
//! v0.0.288: Removed hardcoded suggestions, now fully data-driven from recipes.

use crate::learning_progress::{compute_learning_progress, LearningProgress};
use crate::recipe_store::RecipeStore;
use crate::system_telemetry::TelemetryStore;
use serde::{Deserialize, Serialize};

/// A learning suggestion for idle display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSuggestion {
    /// Category of suggestion
    pub category: SuggestionCategory,
    /// Brief description
    pub title: String,
    /// Detailed explanation
    pub description: String,
    /// Example query the user could try
    pub example_query: Option<String>,
    /// Priority (1=high, 5=low)
    pub priority: u8,
}

/// Categories of learning suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionCategory {
    /// Explore a new domain
    NewDomain,
    /// Deep dive into existing knowledge
    DeepDive,
    /// Address a gap in knowledge
    KnowledgeGap,
    /// Improve weak areas
    Improvement,
    /// System health related
    SystemHealth,
}

impl std::fmt::Display for SuggestionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewDomain => write!(f, "explore"),
            Self::DeepDive => write!(f, "deep-dive"),
            Self::KnowledgeGap => write!(f, "gap"),
            Self::Improvement => write!(f, "improve"),
            Self::SystemHealth => write!(f, "health"),
        }
    }
}

/// Generate learning suggestions based on current state
/// v0.0.288: Now data-driven, no hardcoded suggestions
pub fn generate_suggestions(
    recipes: Option<&RecipeStore>,
    telemetry: Option<&TelemetryStore>,
) -> Vec<LearningSuggestion> {
    let mut suggestions = Vec::new();

    // v0.0.288: Use learning progress for data-driven suggestions
    let progress = compute_learning_progress();
    suggestions.extend(suggestions_from_progress(&progress));

    // Analyze recipe coverage
    if let Some(store) = recipes {
        suggestions.extend(analyze_recipe_gaps(store));
    }

    // Analyze telemetry for health-related learning
    if let Some(store) = telemetry {
        suggestions.extend(analyze_health_learning(store));
    }

    // Sort by priority
    suggestions.sort_by_key(|s| s.priority);

    // Limit to top 5
    suggestions.truncate(5);

    suggestions
}

/// v0.0.288: Generate suggestions from learning progress (data-driven)
fn suggestions_from_progress(progress: &LearningProgress) -> Vec<LearningSuggestion> {
    let mut suggestions = Vec::new();

    // If Anna is a beginner, suggest exploration
    if progress.is_beginner() {
        suggestions.push(LearningSuggestion {
            category: SuggestionCategory::NewDomain,
            title: "Help Anna learn".to_string(),
            description: format!(
                "Anna has {} recipes. Ask questions to help her grow!",
                progress.recipes_total
            ),
            example_query: None, // No hardcoded queries
            priority: 1,
        });
    }

    // Suggest improvement for growing areas (from actual data)
    for area in progress.growing_areas.iter().take(2) {
        suggestions.push(LearningSuggestion {
            category: SuggestionCategory::KnowledgeGap,
            title: format!("Explore {}", area),
            description: format!(
                "Anna is still learning about {}. Your questions help!",
                area
            ),
            example_query: None,
            priority: 2,
        });
    }

    // Acknowledge strong areas
    if !progress.strong_areas.is_empty() && progress.recipes_total >= 20 {
        suggestions.push(LearningSuggestion {
            category: SuggestionCategory::DeepDive,
            title: "Growing expertise".to_string(),
            description: format!(
                "Anna is confident with {}. Deep questions welcome!",
                progress.strong_areas.join(", ")
            ),
            example_query: None,
            priority: 4,
        });
    }

    suggestions
}

/// Analyze recipe store for knowledge gaps
fn analyze_recipe_gaps(store: &RecipeStore) -> Vec<LearningSuggestion> {
    let mut suggestions = Vec::new();

    // Count recipes by category
    let mut category_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for recipe in store.recipes.values() {
        *category_counts
            .entry(recipe.category.clone())
            .or_default() += 1;
    }

    // Find categories with few recipes
    let expected_categories = [
        "storage",
        "memory",
        "network",
        "services",
        "desktop",
        "security",
        "performance",
    ];

    for category in expected_categories {
        let count = category_counts.get(category).copied().unwrap_or(0);
        if count < 3 {
            suggestions.push(LearningSuggestion {
                category: SuggestionCategory::KnowledgeGap,
                title: format!("Learn about {}", category),
                description: format!(
                    "Only {} recipes for {}. Ask me questions to help me learn!",
                    count, category
                ),
                example_query: example_query_for_domain(category),
                priority: if count == 0 { 1 } else { 2 },
            });
        }
    }

    // Find recipes with low reliability
    let weak_recipes: Vec<_> = store
        .recipes
        .values()
        .filter(|r| r.learned_reliability < 70 && r.usage_count > 0)
        .collect();

    if !weak_recipes.is_empty() {
        suggestions.push(LearningSuggestion {
            category: SuggestionCategory::Improvement,
            title: "Improve weak areas".to_string(),
            description: format!(
                "{} recipes have reliability under 70%. Asking similar questions helps me improve!",
                weak_recipes.len()
            ),
            example_query: weak_recipes.first().map(|r| r.title.clone()),
            priority: 2,
        });
    }

    suggestions
}

/// Example query for a domain - returns None to avoid hardcoded examples
/// v0.0.288: Removed hardcoded queries, let users ask naturally
fn example_query_for_domain(_domain: &str) -> Option<String> {
    // No hardcoded examples - Anna learns from user queries
    None
}

/// Analyze telemetry for health-related learning opportunities
fn analyze_health_learning(store: &TelemetryStore) -> Vec<LearningSuggestion> {
    let mut suggestions = Vec::new();

    // Check health score
    let score = store.health_score();
    if score < 80 {
        suggestions.push(LearningSuggestion {
            category: SuggestionCategory::SystemHealth,
            title: "System health check".to_string(),
            description: format!(
                "Health score is {}%. Ask me to diagnose system issues.",
                score
            ),
            example_query: Some("What's wrong with my system?".to_string()),
            priority: 1,
        });
    }

    // Check for anomalies
    let anomalies = store.recent_anomalies();
    if !anomalies.is_empty() {
        let categories: Vec<_> = anomalies
            .iter()
            .map(|a| a.category.to_string())
            .collect();
        let unique: std::collections::HashSet<_> = categories.into_iter().collect();

        suggestions.push(LearningSuggestion {
            category: SuggestionCategory::SystemHealth,
            title: "Investigate anomalies".to_string(),
            description: format!(
                "Detected {} anomalies in: {}",
                anomalies.len(),
                unique.into_iter().collect::<Vec<_>>().join(", ")
            ),
            example_query: Some("Show me system health details".to_string()),
            priority: 2,
        });
    }

    suggestions
}

// v0.0.288: Removed general_exploration_suggestions()
// All suggestions now come from actual recipe/learning data

/// Format suggestions for display
pub fn format_suggestions_for_display(suggestions: &[LearningSuggestion]) -> String {
    if suggestions.is_empty() {
        return String::new();
    }

    let mut output = String::from("Learning opportunities:\n");

    for (i, suggestion) in suggestions.iter().take(3).enumerate() {
        output.push_str(&format!(
            "  {}. [{}] {}\n",
            i + 1,
            suggestion.category,
            suggestion.title
        ));
        if let Some(example) = &suggestion.example_query {
            output.push_str(&format!("     Try: \"{}\"\n", example));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_suggestions_empty() {
        let suggestions = generate_suggestions(None, None);
        // Should have general suggestions even without stores
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_format_suggestions() {
        let suggestions = vec![LearningSuggestion {
            category: SuggestionCategory::NewDomain,
            title: "Test".to_string(),
            description: "Description".to_string(),
            example_query: Some("try this".to_string()),
            priority: 1,
        }];

        let formatted = format_suggestions_for_display(&suggestions);
        assert!(formatted.contains("Test"));
        assert!(formatted.contains("try this"));
    }

    #[test]
    fn test_example_queries() {
        // v0.0.288: Function now returns None (no hardcoded examples)
        assert!(example_query_for_domain("storage").is_none());
        assert!(example_query_for_domain("memory").is_none());
    }
}
