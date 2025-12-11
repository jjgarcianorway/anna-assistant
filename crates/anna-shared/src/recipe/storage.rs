//! Recipe storage functions (v0.0.381).
//!
//! v0.0.381: Lowered recipe persistence threshold from 80 to 70 for faster learning.

use std::path::PathBuf;

/// Get the recipes directory path
pub fn recipe_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("recipes")
}

/// Get path for a specific recipe file
pub fn recipe_filename(recipe_id: &str) -> PathBuf {
    recipe_dir().join(format!("{}.json", recipe_id))
}

/// Check if a recipe should be persisted (v0.0.381: lowered threshold).
/// Only persist when: Verified status AND reliability >= 70.
///
/// This is the ONLY gate for recipe persistence - all callers MUST use this function.
/// Rationale: Never learn from unverified outcomes; only from proven successes.
///
/// v0.0.381: Lowered from 80 to 70 to improve learning rate.
/// - 80 was too strict: many legitimate answers scored 70-79 due to minor penalties
/// - Dynamic recipe maturity thresholds (v0.0.373) provide additional safety
/// - New recipes need higher match scores to be used, preventing low-quality answers
pub fn should_persist_recipe(verified: bool, score: u8) -> bool {
    verified && score >= RECIPE_PERSIST_THRESHOLD
}

/// Threshold for recipe persistence (v0.0.381: lowered from 80 to 70)
pub const RECIPE_PERSIST_THRESHOLD: u8 = 70;

/// Clear all recipes (for reset) (v0.0.28)
pub fn clear_all_recipes() -> std::io::Result<()> {
    let dir = recipe_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

/// Count recipes in store
pub fn recipe_count() -> usize {
    let dir = recipe_dir();
    if !dir.exists() {
        return 0;
    }
    std::fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}
