//! Clarification types and flow for the Translator.
//!
//! When the Translator cannot determine user intent with high confidence,
//! it generates structured clarification requests.

use serde::{Deserialize, Serialize};

/// Types of clarification needed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClarificationType {
    /// Intent is unclear - what does the user want to do?
    IntentUnclear,
    /// Missing required parameter (e.g., package name, service name)
    MissingParameter,
    /// Ambiguous request - multiple interpretations possible
    AmbiguousRequest,
    /// Confirmation needed before system-modifying action
    ConfirmationNeeded,
    /// Choice between multiple options
    MultipleOptions,
}

/// A clarification request to the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    /// Type of clarification needed
    pub clarification_type: ClarificationType,
    /// Question to ask the user
    pub question: String,
    /// Suggested options (if applicable)
    pub options: Vec<String>,
    /// Original context that triggered this clarification
    pub context: String,
}

impl Clarification {
    /// Create a new intent clarification
    pub fn intent_unclear(original_input: &str) -> Self {
        Self {
            clarification_type: ClarificationType::IntentUnclear,
            question: "I'm not sure what you're asking. What would you like to do?".to_string(),
            options: vec![
                "Check system information".to_string(),
                "Install or manage packages".to_string(),
                "Troubleshoot a problem".to_string(),
                "Learn how to do something".to_string(),
            ],
            context: original_input.to_string(),
        }
    }

    /// Create a missing parameter clarification
    pub fn missing_parameter(param_type: &str, original_input: &str) -> Self {
        Self {
            clarification_type: ClarificationType::MissingParameter,
            question: format!("Which {} are you referring to?", param_type),
            options: vec![],
            context: original_input.to_string(),
        }
    }

    /// Create an ambiguous request clarification
    pub fn ambiguous(question: &str, options: Vec<String>, original_input: &str) -> Self {
        Self {
            clarification_type: ClarificationType::AmbiguousRequest,
            question: question.to_string(),
            options,
            context: original_input.to_string(),
        }
    }

    /// Create a confirmation request
    pub fn confirm_action(action_description: &str, original_input: &str) -> Self {
        Self {
            clarification_type: ClarificationType::ConfirmationNeeded,
            question: format!("Do you want me to {}?", action_description),
            options: vec!["Yes, proceed".to_string(), "No, cancel".to_string()],
            context: original_input.to_string(),
        }
    }

    /// Create a multiple choice clarification
    pub fn choose_option(question: &str, options: Vec<String>, original_input: &str) -> Self {
        Self {
            clarification_type: ClarificationType::MultipleOptions,
            question: question.to_string(),
            options,
            context: original_input.to_string(),
        }
    }

    /// Check if this is a yes/no confirmation
    pub fn is_confirmation(&self) -> bool {
        self.clarification_type == ClarificationType::ConfirmationNeeded
    }

    /// Check if this has predefined options
    pub fn has_options(&self) -> bool {
        !self.options.is_empty()
    }

    /// Format for display to user
    pub fn format_for_user(&self) -> String {
        let mut output = self.question.clone();

        if !self.options.is_empty() {
            output.push_str("\n");
            for (i, option) in self.options.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, option));
            }
        }

        output
    }
}

/// Result of processing a clarification response
#[derive(Debug, Clone)]
pub enum ClarificationResult {
    /// User provided a valid response
    Resolved {
        /// The selected option index (if options were provided)
        selected_index: Option<usize>,
        /// The user's response text
        response: String,
    },
    /// User cancelled the clarification
    Cancelled,
    /// User response was not understood
    Invalid { reason: String },
}

impl ClarificationResult {
    /// Parse user response to a clarification
    pub fn parse_response(
        response: &str,
        clarification: &Clarification,
    ) -> Self {
        let response_lower = response.trim().to_lowercase();

        // Check for cancellation
        if response_lower == "cancel" || response_lower == "nevermind" || response_lower == "no" {
            return ClarificationResult::Cancelled;
        }

        // If options are available, try to match
        if !clarification.options.is_empty() {
            // Try numeric selection (1, 2, 3, ...)
            if let Ok(num) = response_lower.parse::<usize>() {
                if num > 0 && num <= clarification.options.len() {
                    return ClarificationResult::Resolved {
                        selected_index: Some(num - 1),
                        response: clarification.options[num - 1].clone(),
                    };
                }
            }

            // Try text match against options
            for (i, option) in clarification.options.iter().enumerate() {
                if response_lower.contains(&option.to_lowercase())
                    || option.to_lowercase().contains(&response_lower)
                {
                    return ClarificationResult::Resolved {
                        selected_index: Some(i),
                        response: option.clone(),
                    };
                }
            }

            // For confirmation, check for yes/no
            if clarification.is_confirmation() {
                if response_lower == "yes" || response_lower == "y" || response_lower == "proceed" {
                    return ClarificationResult::Resolved {
                        selected_index: Some(0),
                        response: "yes".to_string(),
                    };
                }
            }
        }

        // Free-form response (for missing parameter, etc.)
        if !response.trim().is_empty() {
            return ClarificationResult::Resolved {
                selected_index: None,
                response: response.trim().to_string(),
            };
        }

        ClarificationResult::Invalid {
            reason: "Could not understand your response.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarification_intent_unclear() {
        let c = Clarification::intent_unclear("some input");
        assert_eq!(c.clarification_type, ClarificationType::IntentUnclear);
        assert!(!c.options.is_empty());
    }

    #[test]
    fn test_clarification_format() {
        let c = Clarification::ambiguous(
            "Which service?",
            vec!["nginx".to_string(), "apache".to_string()],
            "restart service",
        );
        let formatted = c.format_for_user();
        assert!(formatted.contains("Which service?"));
        assert!(formatted.contains("1. nginx"));
        assert!(formatted.contains("2. apache"));
    }

    #[test]
    fn test_parse_numeric_response() {
        let c = Clarification::ambiguous(
            "Choose:",
            vec!["option A".to_string(), "option B".to_string()],
            "input",
        );

        let result = ClarificationResult::parse_response("1", &c);
        match result {
            ClarificationResult::Resolved { selected_index, response } => {
                assert_eq!(selected_index, Some(0));
                assert_eq!(response, "option A");
            }
            _ => panic!("Expected Resolved"),
        }
    }

    #[test]
    fn test_parse_cancel() {
        let c = Clarification::intent_unclear("input");
        let result = ClarificationResult::parse_response("cancel", &c);
        assert!(matches!(result, ClarificationResult::Cancelled));
    }

    #[test]
    fn test_parse_confirmation_yes() {
        let c = Clarification::confirm_action("restart nginx", "restart nginx");
        let result = ClarificationResult::parse_response("yes", &c);
        match result {
            ClarificationResult::Resolved { selected_index, response } => {
                assert_eq!(selected_index, Some(0));
                // Response is the matched option text, not user input
                assert!(response.to_lowercase().contains("yes"));
            }
            _ => panic!("Expected Resolved"),
        }
    }

    #[test]
    fn test_parse_freeform() {
        let c = Clarification::missing_parameter("package", "install");
        let result = ClarificationResult::parse_response("neovim", &c);
        match result {
            ClarificationResult::Resolved { selected_index, response } => {
                assert_eq!(selected_index, None);
                assert_eq!(response, "neovim");
            }
            _ => panic!("Expected Resolved"),
        }
    }
}
