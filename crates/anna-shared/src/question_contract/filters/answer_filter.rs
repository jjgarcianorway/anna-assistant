//! Answer content filter implementation.
//!
//! Help Text Leakage Filters (Part D) - v0.0.437.
//!
//! If category=fact or status:
//! - Anna must NEVER output:
//!   - Tutorials
//!   - Debug steps
//!   - Commands
//!   - Suggestions
//! Unless the user explicitly asks "how", "why", or "fix".
//!
//! Example failure to eliminate:
//! User: "do I have failed systemd services?"
//! Wrong: Showing how to debug systemd.
//! Correct: "No failed systemd services detected."

use super::types::{DetectedLeakage, FilterResult, LeakageType};
use crate::question_contract::intent::QuestionIntent;

/// Answer content filter.
pub struct AnswerFilter;

impl AnswerFilter {
    /// Filter an answer based on the intent.
    pub fn filter(intent: &QuestionIntent, raw_text: &str) -> FilterResult {
        // If category allows tutorials, don't filter
        if intent.category.allows_tutorials() {
            return FilterResult::clean(raw_text.to_string());
        }

        // If extras are allowed, don't filter
        if intent.allows_extras() {
            return FilterResult::clean(raw_text.to_string());
        }

        let mut filtered = raw_text.to_string();
        let mut leakages = Vec::new();

        // Filter tutorials
        filtered = Self::filter_tutorials(&filtered, &mut leakages);

        // Filter debug steps
        filtered = Self::filter_debug_steps(&filtered, &mut leakages);

        // Filter commands
        filtered = Self::filter_commands(&filtered, &mut leakages);

        // Filter suggestions
        filtered = Self::filter_suggestions(&filtered, &mut leakages);

        // Clean up whitespace
        filtered = Self::cleanup_whitespace(&filtered);

        if leakages.is_empty() {
            FilterResult::clean(filtered)
        } else {
            FilterResult::filtered(filtered, leakages)
        }
    }

    /// Filter tutorial-style text.
    fn filter_tutorials(text: &str, leakages: &mut Vec<DetectedLeakage>) -> String {
        let patterns = [
            ("you can ", "tutorial_you_can"),
            ("you could ", "tutorial_you_could"),
            ("try running ", "tutorial_try_running"),
            ("to do this, ", "tutorial_to_do"),
            ("here's how ", "tutorial_heres_how"),
            ("follow these steps", "tutorial_follow_steps"),
            ("first, ", "tutorial_first"),
            ("next, ", "tutorial_next"),
            ("then, ", "tutorial_then"),
            ("finally, ", "tutorial_finally"),
            ("for more information", "tutorial_more_info"),
            ("if you want to ", "tutorial_if_want"),
        ];

        Self::filter_sentences_with_patterns(text, &patterns, LeakageType::Tutorial, leakages)
    }

    /// Filter debug step text.
    fn filter_debug_steps(text: &str, leakages: &mut Vec<DetectedLeakage>) -> String {
        let patterns = [
            ("to diagnose ", "debug_diagnose"),
            ("to debug ", "debug_debug"),
            ("check the logs", "debug_check_logs"),
            ("inspect the ", "debug_inspect"),
            ("troubleshoot ", "debug_troubleshoot"),
            ("to find out why", "debug_find_out"),
            ("to investigate ", "debug_investigate"),
        ];

        Self::filter_sentences_with_patterns(text, &patterns, LeakageType::DebugSteps, leakages)
    }

    /// Filter command suggestions.
    fn filter_commands(text: &str, leakages: &mut Vec<DetectedLeakage>) -> String {
        let mut result = text.to_string();

        // Filter code blocks
        let code_block_pattern = "```";
        if let Some(start) = result.find(code_block_pattern) {
            if let Some(end) = result[start + 3..].find(code_block_pattern) {
                let removed = result[start..start + 3 + end + 3].to_string();
                leakages.push(DetectedLeakage {
                    leakage_type: LeakageType::Commands,
                    removed_text: removed.clone(),
                    pattern: "code_block".to_string(),
                });
                result = format!("{}{}", &result[..start], &result[start + 3 + end + 3..]);
            }
        }

        // Filter inline commands (backticks with command-like content)
        let command_patterns = [
            ("run `", "inline_run"),
            ("execute `", "inline_execute"),
            ("use `", "inline_use"),
            ("type `", "inline_type"),
        ];

        for (pattern, name) in command_patterns {
            while result.to_lowercase().contains(pattern) {
                if let Some(start) = result.to_lowercase().find(pattern) {
                    // Find the end of the command (closing backtick)
                    let search_start = start + pattern.len();
                    if let Some(end) = result[search_start..].find('`') {
                        let removed = result[start..search_start + end + 1].to_string();
                        leakages.push(DetectedLeakage {
                            leakage_type: LeakageType::Commands,
                            removed_text: removed,
                            pattern: name.to_string(),
                        });
                        result =
                            format!("{}{}", &result[..start], &result[search_start + end + 1..]);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        result
    }

    /// Filter unsolicited suggestions.
    fn filter_suggestions(text: &str, leakages: &mut Vec<DetectedLeakage>) -> String {
        let patterns = [
            ("you should ", "suggest_should"),
            ("you might want to ", "suggest_might"),
            ("consider ", "suggest_consider"),
            ("i recommend ", "suggest_recommend"),
            ("i suggest ", "suggest_suggest"),
            ("it's recommended ", "suggest_recommended"),
            ("a good practice ", "suggest_practice"),
            ("you may want to ", "suggest_may"),
        ];

        Self::filter_sentences_with_patterns(text, &patterns, LeakageType::Suggestions, leakages)
    }

    /// Filter sentences containing specific patterns.
    fn filter_sentences_with_patterns(
        text: &str,
        patterns: &[(&str, &str)],
        leakage_type: LeakageType,
        leakages: &mut Vec<DetectedLeakage>,
    ) -> String {
        let mut result = text.to_string();

        for (pattern, name) in patterns {
            let lower = result.to_lowercase();
            if let Some(pos) = lower.find(pattern) {
                // Find sentence boundaries
                let sentence_start = result[..pos]
                    .rfind(|c| c == '.' || c == '\n')
                    .map(|p| p + 1)
                    .unwrap_or(0);

                let sentence_end = result[pos..]
                    .find(|c| c == '.' || c == '\n')
                    .map(|p| pos + p + 1)
                    .unwrap_or(result.len());

                let removed = result[sentence_start..sentence_end].trim().to_string();

                if !removed.is_empty() {
                    leakages.push(DetectedLeakage {
                        leakage_type,
                        removed_text: removed,
                        pattern: name.to_string(),
                    });

                    result = format!("{}{}", &result[..sentence_start], &result[sentence_end..]);
                }
            }
        }

        result
    }

    /// Clean up excessive whitespace.
    fn cleanup_whitespace(text: &str) -> String {
        let mut result = text.to_string();

        // Replace multiple newlines with double newline
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }

        // Remove leading/trailing whitespace
        result = result.trim().to_string();

        // Remove multiple spaces
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }

        result
    }

    /// Quick check if text likely has leakage (without full filtering).
    pub fn likely_has_leakage(intent: &QuestionIntent, text: &str) -> bool {
        if intent.category.allows_tutorials() || intent.allows_extras() {
            return false;
        }

        let lower = text.to_lowercase();

        // Quick pattern checks
        lower.contains("you can ")
            || lower.contains("try running ")
            || lower.contains("```")
            || lower.contains("you should ")
            || lower.contains("to diagnose ")
            || lower.contains("for more information")
    }
}
