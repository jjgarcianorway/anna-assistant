//! Interactive Mode - Multi-turn conversations with follow-up questions
//!
//! v6.46.0: Enables LLM to ask clarifying questions when results are ambiguous
//! or suggest alternative approaches when more data is needed.

use crate::planner_core::Intent;
use crate::validation_loop::{ValidationDecision, ValidationResponse};
use serde::{Deserialize, Serialize};

/// Interactive session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveSession {
    /// Original user query
    pub original_query: String,

    /// Current round number
    pub round: usize,

    /// Session history
    pub history: Vec<InteractionTurn>,

    /// Whether the session is still active
    pub active: bool,
}

/// A single turn in the interactive session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionTurn {
    /// Round number
    pub round: usize,

    /// Intent for this round
    pub intent: Intent,

    /// Validation response from LLM
    pub validation: ValidationResponse,

    /// User's response (if any)
    pub user_response: Option<String>,
}

/// Interactive prompt types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InteractivePromptType {
    /// Ask user to clarify ambiguous results
    ClarificationRequest {
        /// What is ambiguous
        reason: String,
        /// Possible interpretations
        alternatives: Vec<String>,
    },

    /// Suggest alternative approach for insufficient data
    AlternativeSuggestion {
        /// Why current approach failed
        reason: String,
        /// Suggested next steps
        suggested_approach: String,
    },

    /// Inform user that we're done
    Completion {
        /// Summary of findings
        summary: String,
    },
}

/// Interactive prompt to show user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractivePrompt {
    /// Type of prompt
    pub prompt_type: InteractivePromptType,

    /// Human-readable message
    pub message: String,

    /// Whether user input is required
    pub requires_user_input: bool,
}

impl InteractiveSession {
    /// Create new interactive session
    pub fn new(query: &str) -> Self {
        Self {
            original_query: query.to_string(),
            round: 0,
            history: Vec::new(),
            active: true,
        }
    }

    /// Add a turn to the session
    pub fn add_turn(&mut self, intent: Intent, validation: ValidationResponse, user_response: Option<String>) {
        self.round += 1;
        self.history.push(InteractionTurn {
            round: self.round,
            intent,
            validation,
            user_response,
        });
    }

    /// Check if session should continue
    pub fn should_continue(&self) -> bool {
        self.active && self.round < 3
    }

    /// Mark session as complete
    pub fn complete(&mut self) {
        self.active = false;
    }
}

/// Build interactive prompt from validation response
pub fn build_interactive_prompt(validation: &ValidationResponse) -> InteractivePrompt {
    match &validation.decision {
        ValidationDecision::Sufficient => {
            InteractivePrompt {
                prompt_type: InteractivePromptType::Completion {
                    summary: validation.reasoning.clone(),
                },
                message: format!("✓ {}", validation.reasoning),
                requires_user_input: false,
            }
        }

        ValidationDecision::NeedMoreData { reason, suggested_approach } => {
            let message = format!(
                "I found some information, but it's incomplete.\n\n\
                 Issue: {}\n\n\
                 Suggestion: {}\n\n\
                 Would you like me to try this approach? (yes/no)",
                reason, suggested_approach
            );

            InteractivePrompt {
                prompt_type: InteractivePromptType::AlternativeSuggestion {
                    reason: reason.clone(),
                    suggested_approach: suggested_approach.clone(),
                },
                message,
                requires_user_input: true,
            }
        }

        ValidationDecision::Ambiguous { reason, alternatives } => {
            let mut message = format!(
                "The results are ambiguous.\n\n\
                 Issue: {}\n\n\
                 Which interpretation makes sense to you?\n",
                reason
            );

            for (i, alt) in alternatives.iter().enumerate() {
                message.push_str(&format!("  {}. {}\n", i + 1, alt));
            }

            message.push_str("\nPlease select a number (1-");
            message.push_str(&alternatives.len().to_string());
            message.push(')');

            InteractivePrompt {
                prompt_type: InteractivePromptType::ClarificationRequest {
                    reason: reason.clone(),
                    alternatives: alternatives.clone(),
                },
                message,
                requires_user_input: true,
            }
        }
    }
}

/// Parse user response for alternative suggestion
pub fn parse_alternative_response(response: &str) -> bool {
    let normalized = response.trim().to_lowercase();
    matches!(normalized.as_str(), "yes" | "y" | "sure" | "ok" | "proceed")
}

/// Parse user response for clarification
pub fn parse_clarification_response(response: &str, alternatives: &[String]) -> Option<usize> {
    let normalized = response.trim();

    // Try to parse as number
    if let Ok(num) = normalized.parse::<usize>() {
        if num > 0 && num <= alternatives.len() {
            return Some(num - 1);
        }
    }

    // Try to match against alternative text
    for (i, alt) in alternatives.iter().enumerate() {
        if alt.to_lowercase().contains(&normalized.to_lowercase()) {
            return Some(i);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_session_creation() {
        let session = InteractiveSession::new("test query");
        assert_eq!(session.original_query, "test query");
        assert_eq!(session.round, 0);
        assert!(session.history.is_empty());
        assert!(session.active);
    }

    #[test]
    fn test_session_add_turn() {
        let mut session = InteractiveSession::new("test");
        let intent = Intent {
            goal: crate::planner_core::GoalType::Inspect,
            domain: crate::planner_core::DomainType::Packages,
            constraints: vec![],
            query: "test".to_string(),
        };
        let validation = ValidationResponse {
            decision: ValidationDecision::Sufficient,
            confidence: 0.9,
            reasoning: "All good".to_string(),
        };

        session.add_turn(intent, validation, None);

        assert_eq!(session.round, 1);
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].round, 1);
    }

    #[test]
    fn test_session_should_continue() {
        let mut session = InteractiveSession::new("test");
        assert!(session.should_continue());

        session.round = 3;
        assert!(!session.should_continue());

        session.round = 1;
        session.complete();
        assert!(!session.should_continue());
    }

    #[test]
    fn test_build_prompt_sufficient() {
        let validation = ValidationResponse {
            decision: ValidationDecision::Sufficient,
            confidence: 0.9,
            reasoning: "Found all packages".to_string(),
        };

        let prompt = build_interactive_prompt(&validation);
        assert!(!prompt.requires_user_input);
        assert!(matches!(prompt.prompt_type, InteractivePromptType::Completion { .. }));
        assert!(prompt.message.contains("Found all packages"));
    }

    #[test]
    fn test_build_prompt_need_more_data() {
        let validation = ValidationResponse {
            decision: ValidationDecision::NeedMoreData {
                reason: "Missing logs".to_string(),
                suggested_approach: "Check system journal".to_string(),
            },
            confidence: 0.6,
            reasoning: "Need more context".to_string(),
        };

        let prompt = build_interactive_prompt(&validation);
        assert!(prompt.requires_user_input);
        assert!(matches!(prompt.prompt_type, InteractivePromptType::AlternativeSuggestion { .. }));
        assert!(prompt.message.contains("Missing logs"));
        assert!(prompt.message.contains("Check system journal"));
    }

    #[test]
    fn test_build_prompt_ambiguous() {
        let validation = ValidationResponse {
            decision: ValidationDecision::Ambiguous {
                reason: "Multiple interpretations".to_string(),
                alternatives: vec![
                    "Steam gaming".to_string(),
                    "Game development".to_string(),
                ],
            },
            confidence: 0.5,
            reasoning: "Unclear intent".to_string(),
        };

        let prompt = build_interactive_prompt(&validation);
        assert!(prompt.requires_user_input);
        assert!(matches!(prompt.prompt_type, InteractivePromptType::ClarificationRequest { .. }));
        assert!(prompt.message.contains("1. Steam gaming"));
        assert!(prompt.message.contains("2. Game development"));
    }

    #[test]
    fn test_parse_alternative_response_yes() {
        assert!(parse_alternative_response("yes"));
        assert!(parse_alternative_response("Yes"));
        assert!(parse_alternative_response("  y  "));
        assert!(parse_alternative_response("sure"));
        assert!(parse_alternative_response("ok"));
    }

    #[test]
    fn test_parse_alternative_response_no() {
        assert!(!parse_alternative_response("no"));
        assert!(!parse_alternative_response("nope"));
        assert!(!parse_alternative_response("cancel"));
    }

    #[test]
    fn test_parse_clarification_response_by_number() {
        let alternatives = vec!["Option A".to_string(), "Option B".to_string()];

        assert_eq!(parse_clarification_response("1", &alternatives), Some(0));
        assert_eq!(parse_clarification_response("2", &alternatives), Some(1));
        assert_eq!(parse_clarification_response("3", &alternatives), None);
        assert_eq!(parse_clarification_response("0", &alternatives), None);
    }

    #[test]
    fn test_parse_clarification_response_by_text() {
        let alternatives = vec![
            "Steam gaming".to_string(),
            "Game development".to_string(),
        ];

        assert_eq!(parse_clarification_response("steam", &alternatives), Some(0));
        assert_eq!(parse_clarification_response("development", &alternatives), Some(1));
        assert_eq!(parse_clarification_response("unknown", &alternatives), None);
    }
}
