//! Recipe statistics display formatting.

use super::category::RecipeCategory;
use super::tracker::RecipeStatsTracker;

/// Format recipe stats for display
pub fn format_recipe_stats(tracker: &RecipeStatsTracker) -> String {
    let mut output = String::new();

    output.push_str("Recipe Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if tracker.total_recipes() == 0 {
        output.push_str("No recipes registered yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Recipes: {} ({} seed, {} learned)\n",
        tracker.total_recipes(),
        tracker.seed_count(),
        tracker.learned_count()
    ));
    output.push_str(&format!(
        "Total Uses:    {} ({:.1}% success)\n\n",
        tracker.total_uses,
        tracker.success_rate()
    ));

    output.push_str("By Category:\n");
    for category in RecipeCategory::all() {
        let count = tracker.by_category.get(&category).copied().unwrap_or(0);
        if count > 0 {
            output.push_str(&format!("  {}: {}\n", category.display_name(), count));
        }
    }

    let most_used = tracker.most_used(3);
    if !most_used.is_empty() {
        output.push_str("\nMost Used:\n");
        for recipe in most_used {
            output.push_str(&format!(
                "  {} - {} uses ({:.0}% success)\n",
                recipe.name,
                recipe.use_count,
                recipe.success_rate()
            ));
        }
    }

    output
}

/// Format compact recipe stats
pub fn format_recipe_stats_compact(tracker: &RecipeStatsTracker) -> String {
    if tracker.total_recipes() == 0 {
        return "No recipes yet".to_string();
    }

    format!(
        "{} recipes ({} learned), {} uses, {:.0}% success",
        tracker.total_recipes(),
        tracker.learned_count(),
        tracker.total_uses,
        tracker.success_rate()
    )
}

/// Generate fun fact about recipes
pub fn recipe_stats_fun_fact(tracker: &RecipeStatsTracker) -> Option<String> {
    if tracker.total_recipes() < 3 {
        return None;
    }

    let facts = vec![
        format!(
            "Anna has learned {} recipes beyond the basics - knowledge grows!",
            tracker.learned_count()
        ),
        format!(
            "Recipes have been used {} times with {:.0}% success!",
            tracker.total_uses,
            tracker.success_rate()
        ),
        tracker
            .top_category()
            .map(|(cat, count)| {
                format!(
                    "{} is the most popular category with {} recipes!",
                    cat.display_name(),
                    count
                )
            })
            .unwrap_or_else(|| "Diverse recipe collection!".to_string()),
        tracker
            .most_used(1)
            .first()
            .map(|r| format!("\"{}\" is the most popular recipe with {} uses!", r.name, r.use_count))
            .unwrap_or_else(|| "All recipes contribute to success!".to_string()),
    ];

    let index = (tracker.total_uses as usize) % facts.len();
    Some(facts[index].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_stats_display::{RecipeCategory, RecipeOriginType, RecipeStats};

    #[test]
    fn test_format_compact() {
        let mut tracker = RecipeStatsTracker::new();

        tracker.register(RecipeStats::new("test", "Test", RecipeCategory::Package, RecipeOriginType::Seed, 1000));
        tracker.record_use("test", true, 0.9, 2000);

        let output = format_recipe_stats_compact(&tracker);
        assert!(output.contains("1 recipes"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = RecipeStatsTracker::new();

        for i in 0..5 {
            tracker.register(RecipeStats::new(
                &format!("r{}", i),
                &format!("Recipe {}", i),
                RecipeCategory::Package,
                RecipeOriginType::Learned,
                1000 + i as u64,
            ));
        }

        let fact = recipe_stats_fun_fact(&tracker);
        assert!(fact.is_some());
    }
}
