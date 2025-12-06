//! Recipe to KnowledgeDoc conversion (v0.0.32R).
//!
//! Converts verified recipes into knowledge documents for RAG retrieval.
//! This allows successful solutions to be served without LLM calls.

use crate::recipe::{Recipe, RecipeAction, RecipeKind};
use super::sources::{KnowledgeDoc, KnowledgeSource, Provenance};

/// Convert a verified recipe into a KnowledgeDoc for the knowledge store.
///
/// Only call this when:
/// - Ticket status = Verified
/// - Reliability score >= 80
pub fn recipe_to_knowledge_doc(recipe: &Recipe) -> KnowledgeDoc {
    let title = build_title(recipe);
    let body = build_body(recipe);
    let tags = build_tags(recipe);

    KnowledgeDoc::new(
        KnowledgeSource::Recipe,
        title,
        body,
        tags,
        Provenance::computed(&format!("recipe:{}", recipe.id), recipe.reliability_score),
    )
    .with_ttl(30) // 30 day TTL for recipe-derived knowledge
}

/// Build a human-readable title from the recipe
fn build_title(recipe: &Recipe) -> String {
    let sig = &recipe.signature;

    // Try to create a "How to" title from the query pattern
    let pattern = &sig.query_pattern;

    if pattern.starts_with("how") {
        // Already a how-to question, capitalize first letter
        capitalize_first(pattern)
    } else if pattern.contains("alternative") || pattern.contains("instead") {
        format!("Finding alternatives: {}", pattern)
    } else {
        // Create a descriptive title from domain + intent
        let domain = &sig.domain;
        let intent = &sig.intent;

        if intent.is_empty() {
            format!("{} - {}", capitalize_first(domain), pattern)
        } else {
            format!("{}: {}", capitalize_first(domain), intent)
        }
    }
}

/// Build the document body with solution details
fn build_body(recipe: &Recipe) -> String {
    let mut body = String::new();

    // Add answer template if present
    if !recipe.answer_template.is_empty() {
        body.push_str("## Solution\n\n");
        body.push_str(&recipe.answer_template);
        body.push_str("\n\n");
    }

    // Add action details for config edit recipes
    if let Some(target) = &recipe.target {
        body.push_str("## Configuration\n\n");
        body.push_str(&format!("**Application:** {}\n", target.app_id));
        body.push_str(&format!("**Config file:** `{}`\n\n", target.config_path_template));
    }

    // Add the specific action
    match &recipe.action {
        RecipeAction::EnsureLine { line } => {
            body.push_str("**Action:** Ensure line exists\n");
            body.push_str(&format!("```\n{}\n```\n\n", line));
        }
        RecipeAction::AppendLine { line } => {
            body.push_str("**Action:** Append line\n");
            body.push_str(&format!("```\n{}\n```\n\n", line));
        }
        RecipeAction::None => {}
    }

    // Add rollback info if available
    if let Some(rollback) = &recipe.rollback {
        body.push_str("## Rollback\n\n");
        body.push_str(&format!("{}\n", rollback.description));
        body.push_str(&format!("Backup at: `{}`\n\n", rollback.backup_path.display()));
    }

    // Add risk level
    body.push_str(&format!("**Risk level:** {:?}\n", recipe.risk_level));

    // Add reliability info
    body.push_str(&format!("**Reliability:** {}% (verified {} times)\n",
        recipe.reliability_score, recipe.success_count));

    body
}

/// Build tags for indexing
fn build_tags(recipe: &Recipe) -> Vec<String> {
    let mut tags = Vec::new();

    // Add domain and intent
    let sig = &recipe.signature;
    if !sig.domain.is_empty() {
        tags.push(sig.domain.clone());
    }
    if !sig.intent.is_empty() {
        for word in sig.intent.split_whitespace() {
            if word.len() >= 3 {
                tags.push(word.to_lowercase());
            }
        }
    }

    // Add route class
    if !sig.route_class.is_empty() {
        tags.push(sig.route_class.clone());
    }

    // Add team
    tags.push(recipe.team.to_string().to_lowercase());

    // Add recipe kind
    match &recipe.kind {
        RecipeKind::Query => tags.push("query".to_string()),
        RecipeKind::ConfigEditLineAppend | RecipeKind::ConfigEnsureLine => {
            tags.push("config".to_string());
            tags.push("edit".to_string());
        }
        RecipeKind::ClarificationTemplate => tags.push("clarification".to_string()),
        RecipeKind::PackageInstall => {
            tags.push("package".to_string());
            tags.push("install".to_string());
        }
        RecipeKind::ServiceManage => {
            tags.push("service".to_string());
            tags.push("systemd".to_string());
        }
        RecipeKind::ShellConfig => {
            tags.push("shell".to_string());
            tags.push("config".to_string());
        }
        RecipeKind::GitConfig => {
            tags.push("git".to_string());
            tags.push("config".to_string());
        }
        RecipeKind::Unknown => {}
    }

    // Add target app if present
    if let Some(target) = &recipe.target {
        tags.push(target.app_id.clone());
    }

    // Extract keywords from query pattern
    for word in sig.query_pattern.split_whitespace() {
        let clean = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if clean.len() >= 3 && !tags.contains(&clean) {
            tags.push(clean);
        }
    }

    // Deduplicate and limit
    tags.sort();
    tags.dedup();
    tags.truncate(20);

    tags
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Check if a recipe should be converted to knowledge doc
pub fn should_convert_to_knowledge(recipe: &Recipe) -> bool {
    // Only convert verified recipes with good reliability
    recipe.reliability_score >= 80 && recipe.success_count >= 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::{RecipeSignature, RecipeTarget};
    use crate::teams::Team;
    use crate::ticket::RiskLevel;

    fn test_recipe() -> Recipe {
        Recipe::new(
            RecipeSignature::new("vim", "enable syntax", "ConfigChange", "how to enable syntax highlighting in vim"),
            Team::Desktop,
            RiskLevel::LowRiskChange,
            vec![],
            vec![],
            "Add `syntax on` to your ~/.vimrc file.".to_string(),
            95,
        )
    }

    #[test]
    fn test_recipe_to_doc_basic() {
        let recipe = test_recipe();
        let doc = recipe_to_knowledge_doc(&recipe);

        assert_eq!(doc.source, KnowledgeSource::Recipe);
        assert!(doc.title.contains("syntax") || doc.title.contains("vim"));
        assert!(doc.body.contains("syntax on"));
        assert!(doc.tags.contains(&"vim".to_string()));
    }

    #[test]
    fn test_recipe_to_doc_config_edit() {
        let recipe = Recipe::config_edit(
            RecipeSignature::new("vim", "syntax", "ConfigChange", "enable vim syntax"),
            Team::Desktop,
            RecipeTarget::new("vim", "$HOME/.vimrc"),
            RecipeAction::EnsureLine { line: "syntax on".to_string() },
            90,
        );

        let doc = recipe_to_knowledge_doc(&recipe);

        assert!(doc.body.contains(".vimrc"));
        assert!(doc.body.contains("syntax on"));
        assert!(doc.tags.contains(&"config".to_string()));
    }

    #[test]
    fn test_build_title_how_to() {
        let mut recipe = test_recipe();
        recipe.signature.query_pattern = "how to install neovim".to_string();

        let title = build_title(&recipe);
        assert!(title.starts_with("How"));
    }

    #[test]
    fn test_build_title_alternative() {
        let mut recipe = test_recipe();
        recipe.signature.query_pattern = "alternative to vim".to_string();

        let title = build_title(&recipe);
        assert!(title.contains("alternative"));
    }

    #[test]
    fn test_should_convert() {
        let recipe = test_recipe();
        assert!(should_convert_to_knowledge(&recipe));

        let mut low_score = test_recipe();
        low_score.reliability_score = 50;
        assert!(!should_convert_to_knowledge(&low_score));
    }

    #[test]
    fn test_tags_deduplication() {
        let recipe = test_recipe();
        let tags = build_tags(&recipe);

        // Check no duplicates
        let mut sorted = tags.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(tags.len(), sorted.len());
    }
}
