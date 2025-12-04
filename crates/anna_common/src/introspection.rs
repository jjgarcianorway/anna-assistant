//! Introspection Module v0.0.13
//!
//! Handles user requests about Anna's learning:
//! - "What have you learned recently?"
//! - "List recipes"
//! - "Show recipe for X"
//! - "Forget what you learned about X"
//! - "Delete that recipe"
//! - "Why did you choose this plan?"
//!
//! These are normal requests routed through the pipeline, with
//! evidence IDs referencing memory/recipe entries.

use crate::memory::{MemoryManager, SessionRecord, MEMORY_EVIDENCE_PREFIX};
use crate::recipes::{Recipe, RecipeManager, RECIPE_EVIDENCE_PREFIX};

/// Introspection intent types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrospectionIntent {
    /// "What have you learned recently?"
    ListRecentLearning,
    /// "List recipes" / "Show all recipes"
    ListRecipes,
    /// "Show recipe for X" / "How do you handle X?"
    ShowRecipe { query: String },
    /// "Forget what you learned about X"
    ForgetLearning { query: String },
    /// "Delete recipe X"
    DeleteRecipe { recipe_id: String },
    /// "Why did you choose this plan?"
    ExplainPlan,
    /// "Show my recent questions"
    ListRecentSessions,
    /// "Search memory for X"
    SearchMemory { query: String },
    /// Not an introspection request
    NotIntrospection,
}

/// Confirmation phrase for forget/delete operations
pub const FORGET_CONFIRMATION: &str = "I CONFIRM (forget)";

/// Detect if a request is an introspection intent
pub fn detect_introspection_intent(request: &str) -> IntrospectionIntent {
    let request_lower = request.to_lowercase();

    // Learning queries
    if request_lower.contains("what have you learned")
        || request_lower.contains("what did you learn")
        || request_lower.contains("show what you learned")
        || request_lower.contains("what you've learned")
    {
        return IntrospectionIntent::ListRecentLearning;
    }

    // Recipe listing
    if request_lower.contains("list recipes")
        || request_lower.contains("show recipes")
        || request_lower.contains("show all recipes")
        || request_lower.contains("what recipes")
    {
        return IntrospectionIntent::ListRecipes;
    }

    // Recipe search
    if request_lower.starts_with("show recipe for")
        || request_lower.starts_with("recipe for")
        || request_lower.contains("how do you handle")
    {
        let query = extract_query_part(
            &request_lower,
            &["show recipe for", "recipe for", "how do you handle"],
        );
        return IntrospectionIntent::ShowRecipe { query };
    }

    // Forget/delete
    if request_lower.contains("forget what you learned")
        || request_lower.contains("forget about")
        || request_lower.contains("delete what you know")
    {
        let query = extract_query_part(
            &request_lower,
            &[
                "forget what you learned about",
                "forget about",
                "delete what you know about",
            ],
        );
        return IntrospectionIntent::ForgetLearning { query };
    }

    // Delete recipe
    if request_lower.starts_with("delete recipe") || request_lower.starts_with("remove recipe") {
        let recipe_id = extract_query_part(&request_lower, &["delete recipe", "remove recipe"]);
        return IntrospectionIntent::DeleteRecipe { recipe_id };
    }

    // Explain plan
    if request_lower.contains("why did you choose")
        || request_lower.contains("why this plan")
        || request_lower.contains("explain your plan")
        || request_lower.contains("how did you decide")
    {
        return IntrospectionIntent::ExplainPlan;
    }

    // Recent sessions
    if request_lower.contains("recent questions")
        || request_lower.contains("my history")
        || request_lower.contains("what have I asked")
        || request_lower.contains("recent sessions")
    {
        return IntrospectionIntent::ListRecentSessions;
    }

    // Search memory
    if request_lower.starts_with("search memory for")
        || request_lower.starts_with("search your memory for")
        || request_lower.contains("do you remember")
    {
        let query = extract_query_part(
            &request_lower,
            &[
                "search memory for",
                "search your memory for",
                "do you remember",
            ],
        );
        return IntrospectionIntent::SearchMemory { query };
    }

    IntrospectionIntent::NotIntrospection
}

/// Extract the query part after a prefix
fn extract_query_part(text: &str, prefixes: &[&str]) -> String {
    for prefix in prefixes {
        if let Some(rest) = text.strip_prefix(prefix) {
            return rest
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
        }
        if let Some(idx) = text.find(prefix) {
            let rest = &text[idx + prefix.len()..];
            return rest
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
        }
    }
    text.to_string()
}

/// Result of an introspection query
#[derive(Debug, Clone)]
pub struct IntrospectionResult {
    /// Whether successful
    pub success: bool,
    /// Human-readable answer
    pub answer: String,
    /// Evidence IDs referenced
    pub evidence_ids: Vec<String>,
    /// Whether confirmation is required
    pub requires_confirmation: bool,
    /// Items found (for listing operations)
    pub items: Vec<IntrospectionItem>,
}

/// An item returned from introspection
#[derive(Debug, Clone)]
pub struct IntrospectionItem {
    pub evidence_id: String,
    pub title: String,
    pub summary: String,
    pub item_type: IntrospectionItemType,
}

#[derive(Debug, Clone, Copy)]
pub enum IntrospectionItemType {
    Session,
    Recipe,
}

impl IntrospectionResult {
    pub fn success(answer: &str) -> Self {
        Self {
            success: true,
            answer: answer.to_string(),
            evidence_ids: Vec::new(),
            requires_confirmation: false,
            items: Vec::new(),
        }
    }

    pub fn failure(answer: &str) -> Self {
        Self {
            success: false,
            answer: answer.to_string(),
            evidence_ids: Vec::new(),
            requires_confirmation: false,
            items: Vec::new(),
        }
    }

    pub fn needs_confirmation(answer: &str) -> Self {
        Self {
            success: true,
            answer: answer.to_string(),
            evidence_ids: Vec::new(),
            requires_confirmation: true,
            items: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, id: &str) -> Self {
        self.evidence_ids.push(id.to_string());
        self
    }

    pub fn with_item(mut self, item: IntrospectionItem) -> Self {
        self.evidence_ids.push(item.evidence_id.clone());
        self.items.push(item);
        self
    }
}

/// Execute an introspection intent
pub fn execute_introspection(intent: &IntrospectionIntent) -> IntrospectionResult {
    match intent {
        IntrospectionIntent::ListRecentLearning => list_recent_learning(),
        IntrospectionIntent::ListRecipes => list_recipes(),
        IntrospectionIntent::ShowRecipe { query } => show_recipe(query),
        IntrospectionIntent::ForgetLearning { query } => forget_learning_request(query),
        IntrospectionIntent::DeleteRecipe { recipe_id } => delete_recipe_request(recipe_id),
        IntrospectionIntent::ExplainPlan => explain_plan(),
        IntrospectionIntent::ListRecentSessions => list_recent_sessions(),
        IntrospectionIntent::SearchMemory { query } => search_memory(query),
        IntrospectionIntent::NotIntrospection => {
            IntrospectionResult::failure("Not an introspection request")
        }
    }
}

/// List recent learning (sessions that created/updated recipes)
fn list_recent_learning() -> IntrospectionResult {
    let manager = MemoryManager::default();
    let learning_sessions = manager.get_learning_sessions(10);

    if learning_sessions.is_empty() {
        return IntrospectionResult::success(
            "I haven't learned anything yet. As I help you with more requests, \
             I'll create recipes for patterns I recognize.",
        );
    }

    let mut result = IntrospectionResult::success(&format!(
        "Here's what I've learned recently ({} sessions with learning):",
        learning_sessions.len()
    ));

    for session in learning_sessions {
        let action_desc = match &session.recipe_action {
            Some(crate::memory::RecipeAction::Created { recipe_id }) => {
                format!("Created recipe {}", recipe_id)
            }
            Some(crate::memory::RecipeAction::Updated { recipe_id }) => {
                format!("Updated recipe {}", recipe_id)
            }
            _ => "Learning recorded".to_string(),
        };

        result = result.with_item(IntrospectionItem {
            evidence_id: session.evidence_id.clone(),
            title: truncate(&session.request_text, 50),
            summary: action_desc,
            item_type: IntrospectionItemType::Session,
        });
    }

    result
}

/// List all recipes
fn list_recipes() -> IntrospectionResult {
    let recipes = RecipeManager::get_all();

    if recipes.is_empty() {
        return IntrospectionResult::success(
            "I don't have any recipes yet. Recipes are created when I successfully \
             handle requests with high reliability (80%+).",
        );
    }

    let stats = RecipeManager::get_stats();
    let mut result = IntrospectionResult::success(&format!(
        "I have {} recipes ({} total uses). Here are the top ones:",
        stats.total_recipes, stats.total_uses
    ));

    // Sort by success count
    let mut sorted_recipes = recipes;
    sorted_recipes.sort_by(|a, b| b.success_count.cmp(&a.success_count));

    for recipe in sorted_recipes.iter().take(10) {
        result = result.with_item(IntrospectionItem {
            evidence_id: recipe.id.clone(),
            title: recipe.name.clone(),
            summary: format!(
                "{} - {:.0}% confidence, {} uses",
                truncate(&recipe.description, 40),
                recipe.confidence * 100.0,
                recipe.success_count
            ),
            item_type: IntrospectionItemType::Recipe,
        });
    }

    if sorted_recipes.len() > 10 {
        result.answer.push_str(&format!(
            "\n\n...and {} more recipes.",
            sorted_recipes.len() - 10
        ));
    }

    result
}

/// Show a specific recipe by query
fn show_recipe(query: &str) -> IntrospectionResult {
    // First try exact ID match
    if query.starts_with(RECIPE_EVIDENCE_PREFIX) {
        if let Some(recipe) = RecipeManager::get(query) {
            return IntrospectionResult::success(&recipe.format_detail()).with_evidence(&recipe.id);
        }
    }

    // Search by keywords
    let matches = RecipeManager::search(query, 5);

    if matches.is_empty() {
        return IntrospectionResult::success(&format!(
            "I don't have a recipe matching '{}'. Try 'list recipes' to see what I know.",
            query
        ));
    }

    if matches.len() == 1 {
        let recipe = &matches[0];
        return IntrospectionResult::success(&recipe.format_detail()).with_evidence(&recipe.id);
    }

    // Multiple matches
    let mut result = IntrospectionResult::success(&format!(
        "Found {} recipes matching '{}'. Here they are:",
        matches.len(),
        query
    ));

    for recipe in &matches {
        result = result.with_item(IntrospectionItem {
            evidence_id: recipe.id.clone(),
            title: recipe.name.clone(),
            summary: truncate(&recipe.description, 50),
            item_type: IntrospectionItemType::Recipe,
        });
    }

    result
}

/// Request to forget learning (requires confirmation)
fn forget_learning_request(query: &str) -> IntrospectionResult {
    let manager = MemoryManager::default();
    let matches = manager.search_sessions(query, 5);

    if matches.is_empty() {
        return IntrospectionResult::success(&format!(
            "I don't have any memory matching '{}' to forget.",
            query
        ));
    }

    let mut result = IntrospectionResult::needs_confirmation(&format!(
        "I found {} session(s) matching '{}'. To forget, type:\n  {}\n\nMatching sessions:",
        matches.len(),
        query,
        FORGET_CONFIRMATION
    ));

    for session in &matches {
        result = result.with_item(IntrospectionItem {
            evidence_id: session.evidence_id.clone(),
            title: truncate(&session.request_text, 50),
            summary: session.timestamp.format("%Y-%m-%d %H:%M").to_string(),
            item_type: IntrospectionItemType::Session,
        });
    }

    result
}

/// Request to delete a recipe (requires confirmation)
fn delete_recipe_request(recipe_id: &str) -> IntrospectionResult {
    let recipe = match RecipeManager::get(recipe_id) {
        Some(r) => r,
        None => {
            // Try search
            let matches = RecipeManager::search(recipe_id, 1);
            match matches.into_iter().next() {
                Some(r) => r,
                None => {
                    return IntrospectionResult::success(&format!(
                        "Recipe '{}' not found. Try 'list recipes' to see available recipes.",
                        recipe_id
                    ));
                }
            }
        }
    };

    IntrospectionResult::needs_confirmation(&format!(
        "To delete recipe [{}] '{}', type:\n  {}\n\nThis will archive the recipe (can be recovered if needed).",
        recipe.id, recipe.name, FORGET_CONFIRMATION
    ))
    .with_evidence(&recipe.id)
}

/// Explain the current/last plan
fn explain_plan() -> IntrospectionResult {
    // This would typically be called with context from the current session
    // For now, return a general explanation
    IntrospectionResult::success(
        "I choose plans based on:\n\
         1. Understanding your intent (question, query, or action)\n\
         2. Matching against known recipes (if any fit well)\n\
         3. Selecting appropriate tools based on targets\n\
         4. Applying safety gates for mutations\n\
         5. Verifying with Junior before responding\n\n\
         For specific plan explanations, ask right after I answer a question.",
    )
}

/// List recent sessions
fn list_recent_sessions() -> IntrospectionResult {
    let manager = MemoryManager::default();
    let sessions = manager.get_recent_sessions(10);

    if sessions.is_empty() {
        return IntrospectionResult::success(
            "No recent sessions recorded. Memory starts from the next request.",
        );
    }

    let mut result =
        IntrospectionResult::success(&format!("Your {} most recent sessions:", sessions.len()));

    for session in &sessions {
        result = result.with_item(IntrospectionItem {
            evidence_id: session.evidence_id.clone(),
            title: truncate(&session.request_text, 50),
            summary: format!(
                "{} - {}% reliability",
                session.timestamp.format("%Y-%m-%d %H:%M"),
                session.reliability_score
            ),
            item_type: IntrospectionItemType::Session,
        });
    }

    result
}

/// Search memory by query
fn search_memory(query: &str) -> IntrospectionResult {
    let manager = MemoryManager::default();
    let sessions = manager.search_sessions(query, 10);

    if sessions.is_empty() {
        return IntrospectionResult::success(&format!(
            "No sessions found matching '{}'. Try different keywords.",
            query
        ));
    }

    let mut result = IntrospectionResult::success(&format!(
        "Found {} session(s) matching '{}':",
        sessions.len(),
        query
    ));

    for session in &sessions {
        result = result.with_item(IntrospectionItem {
            evidence_id: session.evidence_id.clone(),
            title: truncate(&session.request_text, 50),
            summary: format!(
                "{} - {}%",
                session.timestamp.format("%Y-%m-%d %H:%M"),
                session.reliability_score
            ),
            item_type: IntrospectionItemType::Session,
        });
    }

    result
}

/// Execute confirmed forget operation
pub fn execute_forget(evidence_id: &str) -> IntrospectionResult {
    // Check if it's a memory or recipe
    if evidence_id.starts_with(MEMORY_EVIDENCE_PREFIX) {
        let manager = MemoryManager::default();
        match manager.archive_session(evidence_id) {
            Ok(true) => IntrospectionResult::success(&format!(
                "Session [{}] has been archived (forgotten). \
                 It's stored in the archive if you ever need to recover it.",
                evidence_id
            )),
            Ok(false) => {
                IntrospectionResult::failure(&format!("Session [{}] not found.", evidence_id))
            }
            Err(e) => IntrospectionResult::failure(&format!("Failed to archive session: {}", e)),
        }
    } else if evidence_id.starts_with(RECIPE_EVIDENCE_PREFIX) {
        match RecipeManager::archive(evidence_id, "user_requested_delete") {
            Ok(true) => IntrospectionResult::success(&format!(
                "Recipe [{}] has been archived (deleted). \
                 It's stored in the archive if you ever need to recover it.",
                evidence_id
            )),
            Ok(false) => {
                IntrospectionResult::failure(&format!("Recipe [{}] not found.", evidence_id))
            }
            Err(e) => IntrospectionResult::failure(&format!("Failed to archive recipe: {}", e)),
        }
    } else {
        IntrospectionResult::failure(&format!("Unknown evidence ID format: {}", evidence_id))
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_learning_intent() {
        assert_eq!(
            detect_introspection_intent("what have you learned recently?"),
            IntrospectionIntent::ListRecentLearning
        );
    }

    #[test]
    fn test_detect_list_recipes() {
        assert_eq!(
            detect_introspection_intent("list recipes"),
            IntrospectionIntent::ListRecipes
        );
        assert_eq!(
            detect_introspection_intent("show all recipes"),
            IntrospectionIntent::ListRecipes
        );
    }

    #[test]
    fn test_detect_show_recipe() {
        let intent = detect_introspection_intent("show recipe for checking cpu");
        assert!(
            matches!(intent, IntrospectionIntent::ShowRecipe { query } if query == "checking cpu")
        );
    }

    #[test]
    fn test_detect_forget() {
        let intent = detect_introspection_intent("forget what you learned about docker");
        assert!(
            matches!(intent, IntrospectionIntent::ForgetLearning { query } if query == "docker")
        );
    }

    #[test]
    fn test_detect_recent_sessions() {
        assert_eq!(
            detect_introspection_intent("show my recent questions"),
            IntrospectionIntent::ListRecentSessions
        );
    }

    #[test]
    fn test_detect_not_introspection() {
        assert_eq!(
            detect_introspection_intent("what cpu do I have?"),
            IntrospectionIntent::NotIntrospection
        );
    }

    #[test]
    fn test_extract_query_part() {
        assert_eq!(
            extract_query_part("show recipe for nginx", &["show recipe for"]),
            "nginx"
        );
        assert_eq!(
            extract_query_part("forget about \"docker\"", &["forget about"]),
            "docker"
        );
    }

    #[test]
    fn test_introspection_result_builder() {
        let result = IntrospectionResult::success("test")
            .with_evidence("E1")
            .with_evidence("E2");
        assert!(result.success);
        assert_eq!(result.evidence_ids.len(), 2);
    }

    #[test]
    fn test_introspection_result_needs_confirmation() {
        let result = IntrospectionResult::needs_confirmation("test");
        assert!(result.requires_confirmation);
    }
}
