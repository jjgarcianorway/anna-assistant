//! Recipe book caching.

use anna_shared::recipe::RecipeBook;
use std::time::Instant;
use tracing::debug;

use super::types::{CachedRecipeBook, RECIPE_BOOK_CACHE, RECIPE_BOOK_TTL_SECS};

/// Get cached recipe book or load it.
pub fn get_cached_recipe_book() -> Option<RecipeBook> {
    if let Ok(guard) = RECIPE_BOOK_CACHE.read() {
        if let Some(ref cached) = *guard {
            if cached.loaded_at.elapsed().as_secs() < RECIPE_BOOK_TTL_SECS {
                debug!("Recipe book cache hit");
                return Some(cached.book.clone());
            }
        }
    }
    match RecipeBook::load() {
        Ok(book) => {
            if let Ok(mut guard) = RECIPE_BOOK_CACHE.write() {
                *guard = Some(CachedRecipeBook {
                    book: book.clone(),
                    loaded_at: Instant::now(),
                });
            }
            Some(book)
        }
        Err(e) => {
            debug!("Failed to load recipe book: {}", e);
            None
        }
    }
}
