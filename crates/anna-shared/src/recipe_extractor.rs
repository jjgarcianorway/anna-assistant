//! Recipe extractor for Anna's learning system.
//! v0.0.418: Extracts recipes from successful tickets.
//!
//! Given an eligible ticket, extracts:
//! - Intent and parameters (pattern)
//! - Plan steps from specialist actions
//! - Preconditions from probes used
//! - Matcher from user query keywords
//! - Citations from knowledge engine

use crate::recipe_eligibility::{check_eligibility, RecipeType, TicketForEligibility};
use crate::recipe_schema::{
    ConfirmationPolicy, PlanStep, Precondition, Recipe, RecipeMatcher, RecipePattern,
    SuccessCriteria,
};
use regex::Regex;
use std::collections::HashMap;

/// Data needed to extract a recipe from a ticket.
#[derive(Debug, Clone)]
pub struct TicketData {
    /// Ticket ID for generating recipe ID
    pub ticket_id: String,
    /// Eligibility data
    pub eligibility: TicketForEligibility,
    /// Probe results used in resolution
    pub probes_used: HashMap<String, String>,
    /// Commands that were executed
    pub commands: Vec<CommandRecord>,
    /// File edits that were made
    pub file_edits: Vec<FileEdit>,
    /// Citations from knowledge engine
    pub citations: Vec<String>,
    /// Translator-extracted slots
    pub slots: HashMap<String, String>,
}

/// Record of a command that was executed.
#[derive(Debug, Clone)]
pub struct CommandRecord {
    pub command: String,
    pub description: Option<String>,
    pub success: bool,
    pub is_verification: bool,
}

/// Record of a file edit.
#[derive(Debug, Clone)]
pub struct FileEdit {
    pub path: String,
    pub edit_type: FileEditType,
    pub content: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileEditType {
    AppendLine,
    PrependLine,
    ReplaceLine,
    EnsureLine,
    RemoveLines,
    WriteFile,
}

/// Result of recipe extraction.
#[derive(Debug)]
pub enum ExtractionResult {
    /// Successfully extracted a new recipe
    NewRecipe(Recipe),
    /// Should update existing recipe
    UpdateExisting { recipe_id: String, new_version: u32 },
    /// Not eligible for extraction
    NotEligible(String),
    /// Failed to extract
    ExtractionFailed(String),
}

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

/// Generate a recipe ID from ticket data.
fn generate_recipe_id(data: &TicketData, recipe_type: Option<RecipeType>) -> String {
    let intent = data.eligibility.intent.as_deref().unwrap_or("unknown");
    let domain = data.eligibility.domain.as_deref().unwrap_or("general");

    // Extract key nouns from query
    let query = data.eligibility.user_query.to_lowercase();
    let key_words: Vec<&str> = query
        .split_whitespace()
        .filter(|w| w.len() > 3 && !is_stop_word(w))
        .take(3)
        .collect();

    let type_suffix = match recipe_type {
        Some(RecipeType::ConfigChange) => "config",
        Some(RecipeType::RepeatableDiagnostic) => "check",
        Some(RecipeType::SimpleFix) => "fix",
        Some(RecipeType::ServiceAction) => "service",
        Some(RecipeType::PackageAction) => "package",
        None => "action",
    };

    if key_words.is_empty() {
        format!(
            "{}_{}_{}_{}",
            domain,
            intent,
            type_suffix,
            &data.ticket_id[..8.min(data.ticket_id.len())]
        )
    } else {
        format!("{}_{}_{}", domain, key_words.join("_"), type_suffix)
    }
}

fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "need", "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "during", "before", "after", "above", "below", "between",
        "under", "again", "further", "then", "once", "here", "there", "when", "where", "why",
        "how", "all", "each", "few", "more", "most", "other", "some", "such", "only", "own",
        "same", "than", "too", "very", "just", "also", "now",
    ];
    STOP_WORDS.contains(&word)
}

/// Extract pattern from ticket data.
fn extract_pattern(data: &TicketData) -> RecipePattern {
    RecipePattern {
        user_goal: data.eligibility.user_query.clone(),
        slots: data.slots.clone(),
    }
}

/// Extract matcher from ticket data.
fn extract_matcher(data: &TicketData) -> RecipeMatcher {
    let query = data.eligibility.user_query.to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();

    // Required keywords: nouns and key verbs from query
    let required: Vec<String> = words
        .iter()
        .filter(|w| w.len() > 3 && !is_stop_word(w))
        .take(4)
        .map(|s| s.to_string())
        .collect();

    // Optional keywords: from slots and summary
    let mut optional: Vec<String> = data.slots.values().cloned().collect();
    if let Some(summary) = &data.eligibility.specialist_summary {
        let summary_words: Vec<String> = summary
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 4 && !is_stop_word(w))
            .take(3)
            .map(String::from)
            .collect();
        optional.extend(summary_words);
    }

    // Negative keywords: detect similar but different tools
    let negative = detect_negative_keywords(&query);

    RecipeMatcher {
        required_keywords: required,
        optional_keywords: optional,
        negative_keywords: negative,
        min_confidence: 0.8,
        exact_intent: data.eligibility.intent.clone(),
    }
}

/// Detect negative keywords (things this recipe should NOT match).
fn detect_negative_keywords(query: &str) -> Vec<String> {
    let mut negatives = Vec::new();

    // Editor-specific negatives
    if query.contains("vim") && !query.contains("neovim") {
        negatives.push("neovim".into());
        negatives.push("nvim".into());
    }
    if query.contains("neovim") || query.contains("nvim") {
        negatives.push("emacs".into());
    }
    if query.contains("emacs") {
        negatives.push("vim".into());
        negatives.push("nvim".into());
    }

    // Shell-specific negatives
    if query.contains("bash") {
        negatives.push("zsh".into());
        negatives.push("fish".into());
    }
    if query.contains("zsh") {
        negatives.push("bash".into());
        negatives.push("fish".into());
    }

    negatives
}

/// Extract preconditions from probe data.
fn extract_preconditions(data: &TicketData) -> Vec<Precondition> {
    let mut preconditions = Vec::new();

    // Check for tool existence from probes
    for (probe_name, probe_result) in &data.probes_used {
        if probe_name.contains("which") || probe_name.contains("command") {
            if !probe_result.is_empty() && !probe_result.contains("not found") {
                // Extract tool name
                if let Some(tool) = extract_tool_from_which(probe_result) {
                    preconditions.push(Precondition::ToolExists { tool });
                }
            }
        }
    }

    // Check for file existence from edits
    for edit in &data.file_edits {
        if edit.edit_type != FileEditType::WriteFile {
            // Existing file edit implies file should exist
            preconditions.push(Precondition::FileExists {
                path: edit.path.clone(),
            });
        }
    }

    preconditions
}

fn extract_tool_from_which(output: &str) -> Option<String> {
    let path = output.trim();
    if path.starts_with('/') {
        path.split('/').last().map(String::from)
    } else {
        Some(path.to_string())
    }
}

/// Extract plan steps from ticket data.
fn extract_plan(data: &TicketData) -> Vec<PlanStep> {
    let mut plan = Vec::new();

    // Add explanation if we have a summary
    if let Some(summary) = &data.eligibility.specialist_summary {
        if !summary.is_empty() {
            plan.push(PlanStep::Explain {
                message: summary.clone(),
            });
        }
    }

    // Add file edits with backups
    for edit in &data.file_edits {
        // Backup first for mutating edits
        if edit.edit_type != FileEditType::WriteFile {
            plan.push(PlanStep::BackupFile {
                path: edit.path.clone(),
            });
        }

        match edit.edit_type {
            FileEditType::AppendLine => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::AppendLine {
                        path: edit.path.clone(),
                        line: content.clone(),
                    });
                }
            }
            FileEditType::PrependLine => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::PrependLine {
                        path: edit.path.clone(),
                        line: content.clone(),
                    });
                }
            }
            FileEditType::ReplaceLine => {
                if let (Some(pattern), Some(content)) = (&edit.pattern, &edit.content) {
                    plan.push(PlanStep::ReplaceLine {
                        path: edit.path.clone(),
                        pattern: pattern.clone(),
                        replacement: content.clone(),
                    });
                }
            }
            FileEditType::EnsureLine => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::EnsureLine {
                        path: edit.path.clone(),
                        line: content.clone(),
                    });
                }
            }
            FileEditType::RemoveLines => {
                if let Some(pattern) = &edit.pattern {
                    plan.push(PlanStep::RemoveLines {
                        path: edit.path.clone(),
                        pattern: pattern.clone(),
                    });
                }
            }
            FileEditType::WriteFile => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::WriteFile {
                        path: edit.path.clone(),
                        content: content.clone(),
                        mode: None,
                    });
                }
            }
        }
    }

    // Add commands
    for cmd in &data.commands {
        if cmd.is_verification {
            plan.push(PlanStep::VerifyCommand {
                command: cmd.command.clone(),
                expect_success: true,
            });
        } else if let Some(service) = extract_systemctl_service(&cmd.command) {
            // Convert systemctl commands to service steps
            if cmd.command.contains("enable") {
                plan.push(PlanStep::EnableService {
                    service,
                    start: cmd.command.contains("--now"),
                });
            } else if cmd.command.contains("disable") {
                plan.push(PlanStep::DisableService {
                    service,
                    stop: cmd.command.contains("--now"),
                });
            } else if cmd.command.contains("restart") {
                plan.push(PlanStep::RestartService { service });
            }
        } else {
            plan.push(PlanStep::RunCommand {
                command: cmd.command.clone(),
                description: cmd.description.clone().unwrap_or_default(),
                rollback_command: None,
            });
        }
    }

    plan
}

fn extract_systemctl_service(command: &str) -> Option<String> {
    let re = Regex::new(r"systemctl\s+(?:enable|disable|start|stop|restart)\s+(\S+)").ok()?;
    re.captures(command)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Determine confirmation policy based on plan steps.
fn determine_confirmation_policy(plan: &[PlanStep]) -> ConfirmationPolicy {
    let has_mutating = plan.iter().any(|s| s.is_mutating());
    if has_mutating {
        ConfirmationPolicy::MutatingOnly
    } else {
        ConfirmationPolicy::Never
    }
}

/// Build success criteria from plan.
fn build_success_criteria(plan: &[PlanStep]) -> SuccessCriteria {
    let must_succeed: Vec<String> = plan
        .iter()
        .filter(|s| s.is_mutating())
        .map(|s| s.type_name().to_string())
        .collect();

    SuccessCriteria {
        must_succeed,
        rollback_on_failure: true,
        post_verification: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_eligibility::TicketForEligibility;

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

    #[test]
    fn test_systemctl_extraction() {
        let service = extract_systemctl_service("systemctl enable sshd");
        assert_eq!(service, Some("sshd".into()));

        let service = extract_systemctl_service("systemctl restart nginx.service");
        assert_eq!(service, Some("nginx.service".into()));
    }
}
