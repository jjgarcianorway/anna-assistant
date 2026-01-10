//! Recipe System - Learned solutions for common tasks.
//!
//! Recipes allow Anna to:
//! - Store successful solutions for reuse
//! - Apply known solutions without LLM overhead
//! - Learn from wiki, LLM, and user corrections
//!
//! Recipe matching considers:
//! - Question patterns (fuzzy matching)
//! - System context (profile fields)
//! - Success history

mod builtin;
mod matching;
mod types;

pub use types::*;

use anyhow::Result;
use std::path::PathBuf;

use crate::config::anna_data_dir;
use matching::{calculate_match_score, recipe_context_matches};

impl RecipeBook {
    /// Load recipe book from disk
    pub fn load() -> Result<Self> {
        let path = recipes_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let book: RecipeBook = serde_json::from_str(&content)?;
            Ok(book)
        } else {
            // Create with built-in recipes
            let mut book = RecipeBook::default();
            builtin::add_builtin_recipes(&mut book);
            book.save()?;
            Ok(book)
        }
    }

    /// Save recipe book to disk
    pub fn save(&self) -> Result<()> {
        let path = recipes_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Find matching recipes for a question
    pub fn find_matches(
        &self,
        question: &str,
        context: &crate::profile::SystemInfo,
    ) -> Vec<&Recipe> {
        let question_lower = question.to_lowercase();
        let words: Vec<&str> = question_lower.split_whitespace().collect();

        let mut matches: Vec<(&Recipe, f32)> = self
            .recipes
            .iter()
            .filter(|r| r.enabled)
            .filter_map(|recipe| {
                // Check context requirements
                if !recipe_context_matches(&recipe.context, context) {
                    return None;
                }

                // Calculate match score
                let score = calculate_match_score(recipe, &question_lower, &words);
                if score > 0.3 {
                    Some((recipe, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        matches.into_iter().map(|(r, _)| r).collect()
    }

    /// Add a new recipe
    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
        self.last_updated = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mark a recipe as used successfully
    pub fn mark_success(&mut self, recipe_id: &str) {
        if let Some(recipe) = self.recipes.iter_mut().find(|r| r.id == recipe_id) {
            recipe.success_count += 1;
            recipe.last_used = Some(chrono::Utc::now().to_rfc3339());
        }
    }
}

/// Get recipes storage path
pub fn recipes_path() -> PathBuf {
    anna_data_dir().join("recipes.json")
}
