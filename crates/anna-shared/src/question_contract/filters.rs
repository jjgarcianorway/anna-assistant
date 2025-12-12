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

use super::intent::{IntentCategory, QuestionIntent};
use serde::{Deserialize, Serialize};

/// Type of content leakage detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeakageType {
    /// Tutorial-style instructions ("you can...", "try...").
    Tutorial,
    /// Debug steps ("to diagnose...", "check the logs...").
    DebugSteps,
    /// Command suggestions ("run `command`", code blocks).
    Commands,
    /// Unsolicited suggestions ("you should...", "consider...").
    Suggestions,
    /// Extra context not asked for.
    ExtraContext,
    /// Off-topic information.
    OffTopic,
}

impl LeakageType {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Tutorial => "tutorial",
            Self::DebugSteps => "debug_steps",
            Self::Commands => "commands",
            Self::Suggestions => "suggestions",
            Self::ExtraContext => "extra_context",
            Self::OffTopic => "off_topic",
        }
    }
}

/// Result of filtering an answer.
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// The filtered text.
    pub filtered_text: String,
    /// Leakage types found.
    pub leakages: Vec<DetectedLeakage>,
    /// Whether filtering was applied.
    pub was_filtered: bool,
}

impl FilterResult {
    /// Create result with no filtering needed.
    pub fn clean(text: String) -> Self {
        Self {
            filtered_text: text,
            leakages: Vec::new(),
            was_filtered: false,
        }
    }

    /// Create result with filtering applied.
    pub fn filtered(text: String, leakages: Vec<DetectedLeakage>) -> Self {
        Self {
            filtered_text: text,
            leakages,
            was_filtered: true,
        }
    }

    /// Check if any leakage was detected.
    pub fn has_leakage(&self) -> bool {
        !self.leakages.is_empty()
    }
}

/// A detected leakage instance.
#[derive(Debug, Clone)]
pub struct DetectedLeakage {
    /// Type of leakage.
    pub leakage_type: LeakageType,
    /// The text that was removed.
    pub removed_text: String,
    /// Pattern that matched.
    pub pattern: String,
}

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

/// Strict filter that rejects answers with any leakage.
pub struct StrictFilter;

impl StrictFilter {
    /// Check if answer passes strict filtering.
    pub fn passes(intent: &QuestionIntent, text: &str) -> StrictFilterResult {
        if intent.category.allows_tutorials() {
            return StrictFilterResult::Pass;
        }

        let result = AnswerFilter::filter(intent, text);

        if result.has_leakage() {
            StrictFilterResult::Reject {
                leakages: result.leakages,
            }
        } else {
            StrictFilterResult::Pass
        }
    }
}

/// Result of strict filtering.
#[derive(Debug, Clone)]
pub enum StrictFilterResult {
    /// Answer passes.
    Pass,
    /// Answer rejected due to leakage.
    Reject { leakages: Vec<DetectedLeakage> },
}

impl StrictFilterResult {
    /// Check if passed.
    pub fn passed(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::question_contract::intent::{IntentBuilder, IntentCategory, Scope, Subject};

    #[test]
    fn test_fact_filters_tutorials() {
        let intent = IntentBuilder::new("int_001")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .build();

        let text = "You have 4.2 GB free RAM. You can try running free -h for more details.";
        let result = AnswerFilter::filter(&intent, text);

        assert!(result.has_leakage());
        assert!(result
            .leakages
            .iter()
            .any(|l| l.leakage_type == LeakageType::Tutorial));
    }

    #[test]
    fn test_status_filters_commands() {
        let intent = IntentBuilder::new("int_002")
            .category(IntentCategory::Status)
            .subject(Subject::Service)
            .build();

        let text = "No failed services. Run `systemctl --failed` to check again later.";
        let result = AnswerFilter::filter(&intent, text);

        assert!(result.has_leakage());
        assert!(result
            .leakages
            .iter()
            .any(|l| l.leakage_type == LeakageType::Commands));
    }

    #[test]
    fn test_explanation_allows_tutorials() {
        let intent = IntentBuilder::new("int_003")
            .category(IntentCategory::Explanation)
            .build();

        let text = "To understand this, you can check the manual. Try running man systemd.";
        let result = AnswerFilter::filter(&intent, text);

        assert!(!result.has_leakage());
        assert!(!result.was_filtered);
    }

    #[test]
    fn test_filters_suggestions() {
        let intent = IntentBuilder::new("int_004")
            .category(IntentCategory::Fact)
            .build();

        let text = "Your boot time is 15 seconds. You should consider disabling some services.";
        let result = AnswerFilter::filter(&intent, text);

        assert!(result.has_leakage());
        assert!(result
            .leakages
            .iter()
            .any(|l| l.leakage_type == LeakageType::Suggestions));
    }

    #[test]
    fn test_filters_code_blocks() {
        let intent = IntentBuilder::new("int_005")
            .category(IntentCategory::Status)
            .build();

        let text = "Service is running.\n```\nsystemctl status nginx\n```";
        let result = AnswerFilter::filter(&intent, text);

        assert!(result.has_leakage());
        assert!(!result.filtered_text.contains("```"));
    }

    #[test]
    fn test_clean_answer_passes() {
        let intent = IntentBuilder::new("int_006")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .build();

        let text = "4.2 GB free RAM.";
        let result = AnswerFilter::filter(&intent, text);

        assert!(!result.has_leakage());
        assert!(!result.was_filtered);
    }

    #[test]
    fn test_strict_filter_rejects_leakage() {
        let intent = IntentBuilder::new("int_007")
            .category(IntentCategory::Fact)
            .build();

        let text = "Result is 42. You should check this more often.";
        let result = StrictFilter::passes(&intent, text);

        assert!(!result.passed());
    }

    #[test]
    fn test_likely_has_leakage() {
        let intent = IntentBuilder::new("int_008")
            .category(IntentCategory::Status)
            .build();

        assert!(AnswerFilter::likely_has_leakage(&intent, "You can do this"));
        assert!(AnswerFilter::likely_has_leakage(&intent, "```code```"));
        assert!(!AnswerFilter::likely_has_leakage(
            &intent,
            "Service is running."
        ));
    }

    #[test]
    fn test_cleanup_whitespace() {
        let text = "Line one.\n\n\n\nLine two.  With   spaces.";
        let cleaned = AnswerFilter::cleanup_whitespace(text);

        assert!(!cleaned.contains("\n\n\n"));
        assert!(!cleaned.contains("  "));
    }
}
