//! Learning stats command (v0.0.412).
//!
//! Shows what Anna has learned from experience:
//! - Probe effectiveness per category
//! - Negative patterns (mistakes to avoid)
//! - Keyword associations (v0.0.325)
//! - Query recommendations test (v0.0.328)
//! - Health status and confidence (v0.0.334)
//! - v0.0.339: Use centralized UI helpers for consistency.
//! - v0.0.344: Use print_title() and print_footer() for consistency.
//! - v0.0.354: Use print_step() for arrow-prefixed lines.
//! - v0.0.406: Add suggest-recipes command for recipe candidate analysis.
//! - v0.0.412: Show learned recipes from RecipeStoreV2.

mod query_recommendations;
mod recipe_analysis;
mod stats_display;
mod utils;

use anyhow::Result;

pub use query_recommendations::show_query_recommendations;
pub use recipe_analysis::handle_suggest_recipes;
pub use stats_display::handle_learning;

/// Handle learning command - show what Anna has learned
/// v0.0.328: Optional query parameter to test recommendations
pub fn handle_learning_with_query(query: Option<&str>) -> Result<()> {
    // If query provided, show recommendations for it
    if let Some(q) = query {
        return show_query_recommendations(q);
    }
    handle_learning()
}
