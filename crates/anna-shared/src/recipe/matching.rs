//! Recipe matching - context and score calculation.

use super::types::{Recipe, RecipeContext};
use crate::profile::SystemInfo;

/// Check if recipe context matches system context
pub fn recipe_context_matches(recipe_ctx: &RecipeContext, system: &SystemInfo) -> bool {
    // OS check
    if let Some(ref required_os) = recipe_ctx.os {
        if let Some(ref system_os) = system.os_name {
            if !system_os.to_lowercase().contains(&required_os.to_lowercase()) {
                return false;
            }
        }
    }

    // Editor check
    if let Some(ref required_editor) = recipe_ctx.editor {
        if let Some(ref system_editor) = system.editor {
            if system_editor != required_editor {
                return false;
            }
        } else {
            return false; // No editor detected, but recipe requires one
        }
    }

    // Shell check
    if let Some(ref required_shell) = recipe_ctx.shell {
        if let Some(ref system_shell) = system.shell {
            if system_shell != required_shell {
                return false;
            }
        }
    }

    // Bootloader check
    if let Some(ref required_boot) = recipe_ctx.bootloader {
        if let Some(ref system_boot) = system.bootloader {
            if system_boot != required_boot {
                return false;
            }
        }
    }

    // Desktop check
    if let Some(ref required_de) = recipe_ctx.desktop {
        if let Some(ref system_de) = system.desktop {
            if !system_de.to_lowercase().contains(&required_de.to_lowercase()) {
                return false;
            }
        }
    }

    // Filesystem check
    if let Some(ref required_fs) = recipe_ctx.filesystem {
        if let Some(ref system_fs) = system.root_filesystem {
            if system_fs != required_fs {
                return false;
            }
        }
    }

    true
}

/// Calculate match score for a recipe
pub fn calculate_match_score(recipe: &Recipe, question: &str, words: &[&str]) -> f32 {
    let mut score = 0.0;

    // Check pattern matches
    for pattern in &recipe.patterns {
        if question.contains(pattern) {
            score += 0.5;
        }
    }

    // Check keyword matches
    let keyword_matches = recipe
        .keywords
        .iter()
        .filter(|k| words.iter().any(|w| w.contains(k.as_str())))
        .count();

    if !recipe.keywords.is_empty() {
        score += (keyword_matches as f32) / (recipe.keywords.len() as f32) * 0.3;
    }

    // v0.0.892: Logarithmic scaling for success count
    // Provides diminishing returns but continues to reward proven recipes
    // 1 success = ~0.07, 10 successes = ~0.16, 50 successes = ~0.25, 100 = ~0.30
    let success_boost = if recipe.success_count > 0 {
        ((recipe.success_count as f32).ln_1p() / 15.0).min(0.35)
    } else {
        0.0
    };
    score += success_boost;

    score
}
