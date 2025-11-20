//! Recipe Formatter - Format all LLM answers as structured recipes
//!
//! Every Anna answer must follow this format:
//! - Summary
//! - Commands to run
//! - Interpretation
//! - Restore steps (if config touched)
//! - Arch Wiki references
//!
//! NO made-up paths, NO "I think maybe", NO commands that don't exist

use anna_common::command_recipe::Recipe;

/// Format a recipe into a user-friendly answer (Beta.94: with emojis)
pub fn format_recipe_answer(recipe: &Recipe, _question: &str) -> String {
    let mut answer = String::new();

    // Section 1: Summary
    answer.push_str("📝 **Summary**\n\n");
    answer.push_str(&recipe.summary);
    answer.push_str("\n\n");

    // Section 2: Commands to run
    if !recipe.steps.is_empty() {
        answer.push_str("⚡ **Commands to Run**\n\n");
        answer.push_str("```bash\n");
        for step in &recipe.steps {
            answer.push_str(&step.command);
            answer.push('\n');
        }
        answer.push_str("```\n\n");
    }

    // Section 3: Interpretation
    answer.push_str("💡 **What This Does**\n\n");
    for step in &recipe.steps {
        if !step.explanation.is_empty() {
            answer.push_str("• **");
            answer.push_str(step.command.split_whitespace().next().unwrap_or("cmd"));
            answer.push_str("**: ");
            answer.push_str(&step.explanation);
            answer.push('\n');
        }
    }
    answer.push('\n');

    // Section 4: Restore steps (if any write operations)
    let has_writes = recipe.steps.iter().any(|s| s.rollback_command.is_some());
    if has_writes {
        answer.push_str("↩️ **Restore Steps**\n\n");
        answer.push_str("If you make changes and want to revert:\n\n");
        for step in &recipe.steps {
            if let Some(rollback) = &step.rollback_command {
                answer.push_str("```bash\n");
                answer.push_str(rollback);
                answer.push_str("\n```\n\n");
            }
        }
        answer.push_str("💾 Backups are created with `.ANNA_BACKUP.YYYYMMDD-HHMMSS` suffix.\n\n");
    }

    // Section 5: Arch Wiki references
    if !recipe.wiki_sources.is_empty() {
        answer.push_str("📚 **Arch Wiki References**\n\n");
        for source in &recipe.wiki_sources {
            answer.push_str("• ");
            answer.push_str(source);
            answer.push('\n');
        }
        answer.push('\n');
    }

    answer
}

/// Format a plain text answer (for when recipe system isn't used)
pub fn format_plain_answer(answer: &str, wiki_sources: &[String]) -> String {
    let mut formatted = String::new();

    formatted.push_str("## Answer\n\n");
    formatted.push_str(answer);
    formatted.push_str("\n\n");

    if !wiki_sources.is_empty() {
        formatted.push_str("## References\n\n");
        for source in wiki_sources {
            formatted.push_str("- ");
            formatted.push_str(source);
            formatted.push('\n');
        }
    }

    formatted
}

/// Detect if answer contains forbidden content
pub fn validate_answer_quality(answer: &str) -> Result<(), String> {
    // Forbidden patterns that indicate hallucination
    let forbidden_patterns = [
        "/var/spaceroot",
        "I think maybe",
        "You can try",
        "might work",
        "should probably",
        "/etc/randomfile",
        "/usr/fake",
    ];

    for pattern in &forbidden_patterns {
        if answer.to_lowercase().contains(&pattern.to_lowercase()) {
            return Err(format!("Answer contains forbidden pattern: {}", pattern));
        }
    }

    // Must contain at least one real command or explanation
    if answer.len() < 20 {
        return Err("Answer too short to be meaningful".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::command_recipe::{
        CommandCategory, CommandRecipe, OutputValidation, Recipe, SafetyLevel,
    };
    use std::collections::HashMap;

    #[test]
    fn test_format_recipe() {
        let recipe = Recipe {
            question: "Do I have swap?".to_string(),
            steps: vec![CommandRecipe {
                id: "check_swap".to_string(),
                command: "swapon --show".to_string(),
                category: CommandCategory::ReadOnly,
                safety_level: SafetyLevel::Safe,
                capture_output: true,
                expected_validation: Some(OutputValidation {
                    exit_code: 0,
                    stdout_must_match: None,
                    stdout_must_not_match: Some("error".to_string()),
                    stderr_must_match: None,
                    validation_description: "Shows swap devices".to_string(),
                }),
                explanation: "Display active swap devices".to_string(),
                doc_sources: vec!["https://wiki.archlinux.org/title/Swap".to_string()],
                rollback_command: None,
                template_id: Some("check_swap_status".to_string()),
                template_params: HashMap::new(),
            }],
            overall_safety: SafetyLevel::Safe,
            all_read_only: true,
            wiki_sources: vec!["https://wiki.archlinux.org/title/Swap".to_string()],
            summary: "Check if swap is enabled on your system".to_string(),
            generated_by: Some("test".to_string()),
            critic_approval: None,
        };

        let formatted = format_recipe_answer(&recipe, "Do I have swap?");

        assert!(formatted.contains("📝 **Summary**"));
        assert!(formatted.contains("⚡ **Commands to Run**"));
        assert!(formatted.contains("swapon --show"));
        assert!(formatted.contains("📚 **Arch Wiki References**"));
        assert!(formatted.contains("wiki.archlinux.org"));
    }

    #[test]
    fn test_validate_answer_quality() {
        assert!(validate_answer_quality("Check swap with swapon --show").is_ok());
        assert!(validate_answer_quality("I think maybe try /var/spaceroot").is_err());
        assert!(validate_answer_quality("short").is_err());
    }
}
