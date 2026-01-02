//! Main recipe extraction logic.
//! v0.0.418: Extracts recipes from successful tickets.
//!
//! Given an eligible ticket, extracts:
//! - Intent and parameters (pattern)
//! - Plan steps from specialist actions
//! - Preconditions from probes used
//! - Matcher from user query keywords
//! - Citations from knowledge engine

use crate::recipe_eligibility::check_eligibility;
use crate::recipe_schema::Recipe;
use super::id_generator::generate_recipe_id;
use super::matcher::{extract_matcher, extract_pattern};
use super::plan::{build_success_criteria, determine_confirmation_policy, extract_plan};
use super::preconditions::extract_preconditions;
use super::types::{ExtractionResult, TicketData};

/// Extract a recipe from ticket data.
pub fn extract_recipe(data: &TicketData) -> ExtractionResult {
    // Check eligibility first
    let eligibility = check_eligibility(&data.eligibility);
    if !eligibility.eligible {
        return ExtractionResult::NotEligible(eligibility.reason);
    }

    // Generate recipe ID
    let recipe_id = generate_recipe_id(data, eligibility.recipe_type);

    // Extract pattern
    let pattern = extract_pattern(data);

    // Extract matcher
    let matcher = extract_matcher(data);

    // Extract preconditions
    let preconditions = extract_preconditions(data);

    // Extract plan steps
    let plan = extract_plan(data);
    if plan.is_empty() {
        return ExtractionResult::ExtractionFailed("No plan steps could be extracted".into());
    }

    // Determine confirmation policy
    let confirmation_policy = determine_confirmation_policy(&plan);

    // Build success criteria
    let success_criteria = build_success_criteria(&plan);

    // Create recipe
    let domain = data
        .eligibility
        .domain
        .clone()
        .unwrap_or_else(|| "general".into());
    let intent = data
        .eligibility
        .intent
        .clone()
        .unwrap_or_else(|| "unknown".into());

    let mut recipe = Recipe::new(recipe_id, domain, intent, pattern, matcher, plan);
    recipe.preconditions = preconditions;
    recipe.confirmation_policy = confirmation_policy;
    recipe.success_criteria = success_criteria;
    recipe.citations = data.citations.clone();

    ExtractionResult::NewRecipe(recipe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_eligibility::TicketForEligibility;
    use super::super::types::{FileEdit, FileEditType};
    use std::collections::HashMap;

    fn make_ticket_data() -> TicketData {
        TicketData {
            ticket_id: "abc123".into(),
            eligibility: TicketForEligibility {
                status: "ok".into(),
                confidence: 95,
                intent: Some("configure_editor_feature".into()),
                domain: Some("desktop".into()),
                user_query: "enable syntax highlighting in vim".into(),
                specialist_summary: Some("Added 'syntax enable' to ~/.vimrc".into()),
                actions: vec!["Edited ~/.vimrc".into()],
                commands_executed: vec![],
                files_modified: vec!["~/.vimrc".into()],
                has_citations: true,
            },
            probes_used: HashMap::new(),
            commands: vec![],
            file_edits: vec![FileEdit {
                path: "~/.vimrc".into(),
                edit_type: FileEditType::AppendLine,
                content: Some("syntax enable".into()),
                pattern: None,
            }],
            citations: vec!["archwiki:Vim#Syntax_highlighting".into()],
            slots: HashMap::from([
                ("editor".into(), "vim".into()),
                ("feature".into(), "syntax_highlighting".into()),
            ]),
        }
    }

    #[test]
    fn test_extract_recipe() {
        let data = make_ticket_data();
        let result = extract_recipe(&data);

        match result {
            ExtractionResult::NewRecipe(recipe) => {
                assert_eq!(recipe.domain, "desktop");
                assert!(!recipe.plan.is_empty());
                assert!(!recipe.matcher.required_keywords.is_empty());
                assert!(recipe
                    .matcher
                    .negative_keywords
                    .contains(&"neovim".to_string()));
            }
            _ => panic!("Expected NewRecipe"),
        }
    }
}
