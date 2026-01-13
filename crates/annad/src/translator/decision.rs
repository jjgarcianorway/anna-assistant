//! Decision Pipeline - Determines how to handle user intents.
//!
//! The Translator decision pipeline takes a UserIntent and decides:
//! - Execute a known recipe directly
//! - Ask for clarification
//! - Report inability to handle (no escalation to AI agents in Phase 7)

use anna_shared::profile::SystemInfo;
use anna_shared::recipe::{Recipe, RecipeBook};

use super::clarification::{Clarification, ClarificationType};
use super::intent::{IntentAction, IntentSubject, UserIntent};

/// Result of the Translator decision pipeline
#[derive(Debug, Clone)]
pub enum TranslatorDecision {
    /// Execute a recipe directly (high confidence, recipe found)
    ExecuteRecipe {
        recipe: Recipe,
        intent: UserIntent,
        confidence: f32,
    },

    /// Need clarification from user before proceeding
    NeedsClarification {
        clarification: Clarification,
        intent: UserIntent,
    },

    /// Cannot handle this request (no matching recipe, needs future specialist)
    CannotHandle {
        reason: String,
        intent: UserIntent,
        suggestions: Vec<String>,
    },

    /// Request requires confirmation before execution
    NeedsConfirmation {
        recipe: Recipe,
        intent: UserIntent,
        warning: Option<String>,
    },
}

/// Decision pipeline configuration
#[derive(Debug, Clone)]
pub struct DecisionConfig {
    /// Minimum confidence to execute without clarification
    pub min_confidence_execute: f32,
    /// Minimum confidence to ask for confirmation (vs clarification)
    pub min_confidence_confirm: f32,
    /// Whether to require confirmation for system-modifying actions
    pub require_confirmation_for_mods: bool,
}

impl Default for DecisionConfig {
    fn default() -> Self {
        Self {
            min_confidence_execute: 0.85,
            min_confidence_confirm: 0.60,
            require_confirmation_for_mods: true,
        }
    }
}

/// The decision pipeline
pub struct DecisionPipeline {
    config: DecisionConfig,
}

impl DecisionPipeline {
    pub fn new(config: DecisionConfig) -> Self {
        Self { config }
    }

    /// Make a decision based on intent and available recipes
    pub fn decide(
        &self,
        intent: &UserIntent,
        recipe_book: &RecipeBook,
        system_info: &SystemInfo,
    ) -> TranslatorDecision {
        // Step 1: Check if we need clarification due to low confidence
        if intent.needs_clarification() {
            return self.create_clarification_decision(intent);
        }

        // Step 2: Try to find a matching recipe
        let matches = recipe_book.find_matches(&intent.original_input, system_info);

        if let Some(recipe) = matches.first() {
            let combined_confidence = self.calculate_combined_confidence(intent, recipe);

            // Step 3: Determine if confirmation is needed
            if self.config.require_confirmation_for_mods && intent.action.modifies_system() {
                return TranslatorDecision::NeedsConfirmation {
                    recipe: (*recipe).clone(),
                    intent: intent.clone(),
                    warning: self.generate_warning(intent, recipe),
                };
            }

            // Step 4: Check if confidence is high enough for direct execution
            if combined_confidence >= self.config.min_confidence_execute {
                return TranslatorDecision::ExecuteRecipe {
                    recipe: (*recipe).clone(),
                    intent: intent.clone(),
                    confidence: combined_confidence,
                };
            }

            // Step 5: Medium confidence - ask for confirmation
            if combined_confidence >= self.config.min_confidence_confirm {
                return TranslatorDecision::NeedsConfirmation {
                    recipe: (*recipe).clone(),
                    intent: intent.clone(),
                    warning: None,
                };
            }
        }

        // Step 6: No recipe found or confidence too low
        self.create_cannot_handle_decision(intent)
    }

    /// Calculate combined confidence from intent and recipe match
    fn calculate_combined_confidence(&self, intent: &UserIntent, recipe: &Recipe) -> f32 {
        // Base confidence from intent classification
        let intent_conf = intent.confidence;

        // Recipe reliability from success history
        let recipe_reliability = if recipe.success_count > 0 {
            // Logarithmic scaling, maxes out at ~0.95
            0.7 + ((recipe.success_count as f32).ln_1p() / 20.0).min(0.25)
        } else {
            0.7 // New recipes start at 70% reliability
        };

        // Combined score: weighted average
        (intent_conf * 0.6) + (recipe_reliability * 0.4)
    }

    /// Generate a warning message for system-modifying actions
    fn generate_warning(&self, intent: &UserIntent, recipe: &Recipe) -> Option<String> {
        if !intent.action.modifies_system() {
            return None;
        }

        let modifying_commands: Vec<_> = recipe
            .commands
            .iter()
            .filter(|c| c.modifies_system)
            .collect();

        if modifying_commands.is_empty() {
            return None;
        }

        let cmd_list: Vec<_> = modifying_commands
            .iter()
            .map(|c| c.command.as_str())
            .collect();

        Some(format!(
            "This will run system-modifying commands: {}",
            cmd_list.join(", ")
        ))
    }

    /// Create a clarification decision when intent is unclear
    fn create_clarification_decision(&self, intent: &UserIntent) -> TranslatorDecision {
        let clarification = match &intent.action {
            IntentAction::Unknown => Clarification {
                clarification_type: ClarificationType::IntentUnclear,
                question: "I'm not sure what you're asking. Could you rephrase?".to_string(),
                options: vec![
                    "Check system status".to_string(),
                    "Install/remove packages".to_string(),
                    "Troubleshoot a problem".to_string(),
                    "Learn how to do something".to_string(),
                ],
                context: intent.original_input.clone(),
            },
            IntentAction::Package => Clarification {
                clarification_type: ClarificationType::MissingParameter,
                question: "Which package are you referring to?".to_string(),
                options: vec![],
                context: intent.original_input.clone(),
            },
            IntentAction::Configure => Clarification {
                clarification_type: ClarificationType::MissingParameter,
                question: "What would you like to configure?".to_string(),
                options: vec![
                    "A service".to_string(),
                    "A file".to_string(),
                    "Network settings".to_string(),
                ],
                context: intent.original_input.clone(),
            },
            _ => Clarification {
                clarification_type: ClarificationType::AmbiguousRequest,
                question: "Could you be more specific about what you need?".to_string(),
                options: vec![],
                context: intent.original_input.clone(),
            },
        };

        TranslatorDecision::NeedsClarification {
            clarification,
            intent: intent.clone(),
        }
    }

    /// Create a cannot-handle decision when no recipe matches
    fn create_cannot_handle_decision(&self, intent: &UserIntent) -> TranslatorDecision {
        let suggestions = self.generate_suggestions(intent);

        let reason = match &intent.action {
            IntentAction::Unknown => "I couldn't understand your request.".to_string(),
            _ => format!(
                "I don't have a recipe for '{}' yet.",
                intent.subject_raw
            ),
        };

        TranslatorDecision::CannotHandle {
            reason,
            intent: intent.clone(),
            suggestions,
        }
    }

    /// Generate helpful suggestions based on intent
    fn generate_suggestions(&self, intent: &UserIntent) -> Vec<String> {
        match &intent.subject {
            IntentSubject::DiskUsage => vec![
                "Try: 'show disk usage'".to_string(),
                "Try: 'how much disk space do I have'".to_string(),
            ],
            IntentSubject::MemoryUsage => vec![
                "Try: 'show memory usage'".to_string(),
                "Try: 'how much RAM is free'".to_string(),
            ],
            IntentSubject::ServiceStatus => vec![
                "Try: 'show failing services'".to_string(),
                "Try: 'check systemd status'".to_string(),
            ],
            _ => vec![
                "Try asking about disk, memory, or CPU usage".to_string(),
                "Try asking how to do something specific".to_string(),
            ],
        }
    }
}

impl Default for DecisionPipeline {
    fn default() -> Self {
        Self::new(DecisionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::recipe::RecipeSource;

    fn mock_recipe() -> Recipe {
        Recipe {
            id: "test-recipe".to_string(),
            name: "Test Recipe".to_string(),
            keywords: vec!["test".to_string()],
            patterns: vec!["test pattern".to_string()],
            context: Default::default(),
            commands: vec![],
            verification: None,
            source: RecipeSource::BuiltIn,
            success_count: 10,
            last_used: None,
            enabled: true,
        }
    }

    #[test]
    fn test_decision_needs_clarification() {
        let pipeline = DecisionPipeline::default();
        let intent = UserIntent {
            action: IntentAction::Unknown,
            confidence: 0.3,
            ..Default::default()
        };
        let book = RecipeBook::default();
        let info = SystemInfo::default();

        let decision = pipeline.decide(&intent, &book, &info);
        assert!(matches!(decision, TranslatorDecision::NeedsClarification { .. }));
    }

    #[test]
    fn test_decision_cannot_handle_no_recipe() {
        let pipeline = DecisionPipeline::default();
        let intent = UserIntent {
            action: IntentAction::Query,
            confidence: 0.9,
            original_input: "something obscure".to_string(),
            ..Default::default()
        };
        let book = RecipeBook::default(); // Empty
        let info = SystemInfo::default();

        let decision = pipeline.decide(&intent, &book, &info);
        assert!(matches!(decision, TranslatorDecision::CannotHandle { .. }));
    }

    #[test]
    fn test_combined_confidence_calculation() {
        let pipeline = DecisionPipeline::default();
        let intent = UserIntent {
            confidence: 0.9,
            ..Default::default()
        };
        let recipe = mock_recipe();

        let combined = pipeline.calculate_combined_confidence(&intent, &recipe);
        // Should be weighted: 0.9 * 0.6 + recipe_reliability * 0.4
        assert!(combined > 0.7);
        assert!(combined < 1.0);
    }
}
