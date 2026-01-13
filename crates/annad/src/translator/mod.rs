//! Translator Module - Central decision brain for Anna.
//!
//! The Translator is NOT an LLM chatbot. It is a deterministic decision engine that:
//! 1. Parses user natural language using pattern and keyword matching
//! 2. Classifies intent into structured types
//! 3. Decides whether to execute a recipe, ask for clarification, or report inability
//! 4. Produces structured internal plans (never exposed to user)
//!
//! User-facing text is produced only by the display layer.
//!
//! # Architecture
//!
//! ```text
//! User Input
//!     |
//!     v
//! Classifier (pattern/keyword matching)
//!     |
//!     v
//! UserIntent (structured)
//!     |
//!     v
//! DecisionPipeline
//!     |
//!     +---> ExecuteRecipe (high confidence + recipe match)
//!     +---> NeedsClarification (low confidence or ambiguous)
//!     +---> NeedsConfirmation (system-modifying action)
//!     +---> CannotHandle (no recipe, future specialist needed)
//! ```

pub mod clarification;
pub mod classifier;
pub mod confidence;
pub mod decision;
pub mod intent;

// Re-export main types
pub use clarification::{Clarification, ClarificationResult, ClarificationType};
pub use classifier::classify;
pub use confidence::{ConfidenceScore, CONFIDENCE_HIGH, CONFIDENCE_LOW, CONFIDENCE_MEDIUM};
pub use decision::{DecisionConfig, DecisionPipeline, TranslatorDecision};
pub use intent::{ClassificationMethod, IntentAction, IntentSubject, UserIntent};

use anna_shared::profile::SystemInfo;
use anna_shared::recipe::RecipeBook;

/// Main Translator entry point
pub struct Translator {
    pipeline: DecisionPipeline,
}

impl Translator {
    /// Create a new Translator with default configuration
    pub fn new() -> Self {
        Self {
            pipeline: DecisionPipeline::default(),
        }
    }

    /// Create a new Translator with custom configuration
    pub fn with_config(config: DecisionConfig) -> Self {
        Self {
            pipeline: DecisionPipeline::new(config),
        }
    }

    /// Process user input and produce a decision
    ///
    /// This is the main entry point for the Translator.
    /// It performs classification and decision-making in a single call.
    pub fn process(
        &self,
        input: &str,
        recipe_book: &RecipeBook,
        system_info: &SystemInfo,
    ) -> TranslatorDecision {
        // Step 1: Classify the input
        let intent = classify(input);

        // Step 2: Make a decision based on intent
        self.pipeline.decide(&intent, recipe_book, system_info)
    }

    /// Classify input only (without decision-making)
    ///
    /// Useful for testing or when you need just the intent.
    pub fn classify_only(&self, input: &str) -> UserIntent {
        classify(input)
    }

    /// Make a decision for a pre-classified intent
    ///
    /// Useful when you have an intent from elsewhere.
    pub fn decide(
        &self,
        intent: &UserIntent,
        recipe_book: &RecipeBook,
        system_info: &SystemInfo,
    ) -> TranslatorDecision {
        self.pipeline.decide(intent, recipe_book, system_info)
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to translate user input
pub fn translate(
    input: &str,
    recipe_book: &RecipeBook,
    system_info: &SystemInfo,
) -> TranslatorDecision {
    let translator = Translator::new();
    translator.process(input, recipe_book, system_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translator_process_query() {
        let translator = Translator::new();
        let book = RecipeBook::load().unwrap_or_default();
        let info = SystemInfo::default();

        let decision = translator.process("how much disk space", &book, &info);

        // Should either execute a recipe or cannot handle (depending on book contents)
        match decision {
            TranslatorDecision::ExecuteRecipe { intent, .. } => {
                assert_eq!(intent.action, IntentAction::Query);
            }
            TranslatorDecision::NeedsConfirmation { intent, .. } => {
                assert_eq!(intent.action, IntentAction::Query);
            }
            TranslatorDecision::CannotHandle { intent, .. } => {
                assert_eq!(intent.action, IntentAction::Query);
            }
            TranslatorDecision::NeedsClarification { .. } => {
                panic!("Should not need clarification for clear query");
            }
        }
    }

    #[test]
    fn test_translator_classify_only() {
        let translator = Translator::new();
        let intent = translator.classify_only("show memory usage");

        assert_eq!(intent.action, IntentAction::Query);
        assert_eq!(intent.subject, IntentSubject::MemoryUsage);
    }

    #[test]
    fn test_translator_unknown_needs_clarification() {
        let translator = Translator::new();
        let book = RecipeBook::default();
        let info = SystemInfo::default();

        let decision = translator.process("xyz abc 123", &book, &info);

        assert!(matches!(
            decision,
            TranslatorDecision::NeedsClarification { .. }
                | TranslatorDecision::CannotHandle { .. }
        ));
    }

    #[test]
    fn test_translate_convenience() {
        let book = RecipeBook::default();
        let info = SystemInfo::default();

        let decision = translate("check cpu usage", &book, &info);

        // Should process without panic
        match decision {
            TranslatorDecision::ExecuteRecipe { .. }
            | TranslatorDecision::NeedsConfirmation { .. }
            | TranslatorDecision::CannotHandle { .. }
            | TranslatorDecision::NeedsClarification { .. } => {}
        }
    }
}
