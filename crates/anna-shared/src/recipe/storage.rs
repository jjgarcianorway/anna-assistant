//! Recipe storage functions (v0.0.177).

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

/// Check if a recipe should be persisted (v0.45.x stabilization gate).
/// Only persist when: Verified status AND reliability >= 80.
///
/// This is the ONLY gate for recipe persistence - all callers MUST use this function.
/// Rationale: Never learn from unverified outcomes; only from proven successes.
pub fn should_persist_recipe(verified: bool, score: u8) -> bool {
    verified && score >= 80
}

/// Threshold for recipe persistence
pub const RECIPE_PERSIST_THRESHOLD: u8 = 80;

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
