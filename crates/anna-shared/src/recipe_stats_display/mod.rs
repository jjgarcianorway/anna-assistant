//! Recipe Statistics Display (v0.0.490).
//!
//! Provides statistics on learned and created recipes.
//! Shows categorization and usage patterns.

mod category;
mod origin;
mod stats;
mod tracker;
mod display;

// Re-export public types
pub use category::RecipeCategory;
pub use origin::RecipeOriginType;
pub use stats::RecipeStats;
pub use tracker::{RecipeStatsTracker, RecipeStatsSummary};
pub use display::{format_recipe_stats, format_recipe_stats_compact, recipe_stats_fun_fact};

/// Check if query is asking about recipe stats
pub fn is_recipe_stats_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "recipe stats",
        "recipes learned",
        "how many recipes",
        "recipe count",
        "most used recipe",
        "popular recipes",
        "recipe categories",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_recipe_stats_query() {
        assert!(is_recipe_stats_query("show recipe stats"));
        assert!(is_recipe_stats_query("how many recipes learned"));
        assert!(is_recipe_stats_query("most used recipe"));

        assert!(!is_recipe_stats_query("install vim"));
        assert!(!is_recipe_stats_query("status"));
    }
}
