//! Recipe store management module.
//!
//! Handles global recipe store initialization, access, and garbage collection.

use anna_shared::recipe_store_v2::RecipeStoreV2;
use anna_shared::recipe_templates;
use std::sync::OnceLock;
use tracing::info;

/// Global recipe store (lazy loaded)
static RECIPE_STORE: OnceLock<std::sync::RwLock<RecipeStoreV2>> = OnceLock::new();

/// Get or initialize the recipe store
pub fn get_store() -> &'static std::sync::RwLock<RecipeStoreV2> {
    RECIPE_STORE.get_or_init(|| {
        let mut store = RecipeStoreV2::load();
        // Initialize with generic templates if empty
        if store.is_empty() {
            recipe_templates::initialize_store(&mut store);
            let _ = store.save();
            info!(
                "Initialized recipe store with {} generic templates",
                store.len()
            );
        }
        // v0.0.412: Run GC on startup
        store.gc();
        let _ = store.save();
        std::sync::RwLock::new(store)
    })
}

/// Trigger garbage collection manually
pub fn run_gc() {
    let store = get_store();
    if let Ok(mut s) = store.write() {
        s.gc();
        let _ = s.save();
        info!("Recipe store GC completed");
    }
}
