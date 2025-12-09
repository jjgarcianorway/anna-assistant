//! Shell recipe search functions (v0.0.231).

use super::catalog::builtin_recipes;
use super::types::{Shell, ShellFeature, ShellRecipe};

/// Find a recipe for a shell and feature
pub fn find_recipe(shell: Shell, feature: ShellFeature) -> Option<ShellRecipe> {
    builtin_recipes()
        .into_iter()
        .find(|r| r.shell == shell && r.feature == feature)
}

/// Find recipes matching keywords
pub fn find_recipes_by_keywords(keywords: &[&str], shell: Option<Shell>) -> Vec<ShellRecipe> {
    builtin_recipes()
        .into_iter()
        .filter(|r| {
            // Filter by shell if specified
            if let Some(s) = shell {
                if r.shell != s {
                    return false;
                }
            }

            // Check if any keyword matches feature keywords
            let feature_keywords = r.feature.keywords();
            keywords.iter().any(|kw| {
                let kw_lower = kw.to_lowercase();
                feature_keywords
                    .iter()
                    .any(|fk| fk.contains(&kw_lower) || kw_lower.contains(fk))
            })
        })
        .collect()
}

/// Detect feature from query
pub fn detect_feature(query: &str) -> Option<ShellFeature> {
    let q = query.to_lowercase();

    if q.contains("git") && q.contains("prompt") {
        return Some(ShellFeature::GitPrompt);
    }
    if q.contains("color") && (q.contains("prompt") || q.contains("ps1")) {
        return Some(ShellFeature::ColoredPrompt);
    }
    if q.contains("syntax") || q.contains("highlight") {
        return Some(ShellFeature::SyntaxHighlighting);
    }
    if q.contains("suggest") || q.contains("auto") && q.contains("complete") {
        return Some(ShellFeature::AutoSuggestions);
    }
    if q.contains("color") && q.contains("ls") {
        return Some(ShellFeature::ColoredLs);
    }
    if q.contains("history") {
        return Some(ShellFeature::HistorySettings);
    }
    if q.contains("alias") {
        return Some(ShellFeature::Aliases);
    }
    if q.contains("path") {
        return Some(ShellFeature::PathAdditions);
    }

    None
}
