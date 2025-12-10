//! Learning stats section for status display (v0.0.280).
//!
//! Shows Anna's recipe learning progress and effectiveness.

use anna_shared::recipe_store::RecipeStore;
use anna_shared::ui::colors;

const KEY_WIDTH: usize = 22;

/// Print the learning section
pub fn print_learning_section() {
    let store = match RecipeStore::load(RecipeStore::default_path()) {
        Ok(s) => s,
        Err(_) => return, // No store yet
    };

    let recipes: Vec<_> = store.recipes.values().collect();

    // Only show if we have learned recipes
    if recipes.is_empty() {
        return;
    }

    println!();
    println!("{}[learning]{}", colors::HEADER, colors::RESET);

    kv("recipes_learned", &format!("{}", recipes.len()));

    // Calculate statistics
    let total_uses: u64 = recipes.iter().map(|r| r.usage_count).sum();
    let avg_reliability: f32 = if !recipes.is_empty() {
        recipes.iter().map(|r| r.learned_reliability as f32).sum::<f32>() / recipes.len() as f32
    } else {
        0.0
    };

    kv("total_recipe_uses", &format!("{}", total_uses));

    let reliability_color = if avg_reliability >= 85.0 {
        colors::OK
    } else if avg_reliability >= 70.0 {
        colors::WARN
    } else {
        colors::ERR
    };
    kv(
        "avg_reliability",
        &format!("{}{:.1}%{}", reliability_color, avg_reliability, colors::RESET),
    );

    // Group by category
    let mut category_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for recipe in &recipes {
        *category_counts.entry(recipe.category.clone()).or_insert(0) += 1;
    }

    if !category_counts.is_empty() {
        kv("by_category", "");
        let mut categories: Vec<_> = category_counts.iter().collect();
        categories.sort_by(|a, b| b.1.cmp(a.1));

        for (category, count) in categories.iter().take(5) {
            println!("    {:12}  {}", category, count);
        }
    }

    // Show top recipes by usage
    let mut sorted_recipes: Vec<_> = recipes.iter().collect();
    sorted_recipes.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));

    if !sorted_recipes.is_empty() {
        kv("top_recipes", "");
        for recipe in sorted_recipes.iter().take(3) {
            let title = truncate_pattern(&recipe.title, 30);
            println!(
                "    {:30}  uses: {:>3}  {}{}%{}",
                title,
                recipe.usage_count,
                if recipe.learned_reliability >= 85 {
                    colors::OK
                } else {
                    colors::WARN
                },
                recipe.learned_reliability,
                colors::RESET
            );
        }
    }

    // Learning rate (recipes learned in last 7 days)
    // For now, just show a placeholder - would need timestamp tracking
    kv("learning_status", &format!("{}ACTIVE{}", colors::OK, colors::RESET));
}

/// Truncate query pattern for display
fn truncate_pattern(pattern: &str, max_len: usize) -> String {
    let cleaned = pattern.trim();
    if cleaned.len() <= max_len {
        cleaned.to_string()
    } else {
        format!("{}...", &cleaned[..max_len - 3])
    }
}

fn kv(key: &str, value: &str) {
    println!("  {:width$}{}", key, value, width = KEY_WIDTH);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_pattern() {
        assert_eq!(truncate_pattern("short", 10), "short");
        assert_eq!(truncate_pattern("this is a very long pattern", 15), "this is a ve...");
    }
}
