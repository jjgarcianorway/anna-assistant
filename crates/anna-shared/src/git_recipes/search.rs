//! Git recipe search functions (v0.0.224).

use super::catalog::builtin_recipes;
use super::recipe::GitRecipe;
use super::types::GitFeature;

/// Find a recipe for a feature
pub fn find_recipe(feature: GitFeature) -> Vec<GitRecipe> {
    builtin_recipes()
        .into_iter()
        .filter(|r| r.feature == feature)
        .collect()
}

/// Find recipes matching keywords
pub fn find_recipes_by_keywords(keywords: &[&str]) -> Vec<GitRecipe> {
    builtin_recipes()
        .into_iter()
        .filter(|r| {
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
pub fn detect_feature(query: &str) -> Option<GitFeature> {
    let q = query.to_lowercase();

    if q.contains("name") || q.contains("email") || q.contains("identity") {
        return Some(GitFeature::UserIdentity);
    }
    if q.contains("default") && q.contains("branch") {
        return Some(GitFeature::DefaultBranch);
    }
    if q.contains("editor") {
        return Some(GitFeature::Editor);
    }
    if q.contains("merge") && q.contains("tool") {
        return Some(GitFeature::MergeTool);
    }
    if q.contains("diff") && q.contains("tool") {
        return Some(GitFeature::DiffTool);
    }
    if q.contains("color") {
        return Some(GitFeature::Colors);
    }
    if q.contains("alias") {
        return Some(GitFeature::Aliases);
    }
    if q.contains("push") {
        return Some(GitFeature::PushDefaults);
    }
    if q.contains("pull") || q.contains("rebase") {
        return Some(GitFeature::PullDefaults);
    }
    if q.contains("credential") || q.contains("password") || q.contains("cache") {
        return Some(GitFeature::CredentialHelper);
    }
    if q.contains("gpg") || q.contains("sign") {
        return Some(GitFeature::GpgSigning);
    }

    None
}
