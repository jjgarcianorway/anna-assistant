//! Types and constants for recipe fast path functionality.

use anna_shared::recipe::Recipe;
use anna_shared::recipe_matcher::MatchResult;
use anna_shared::rpc::TranslatorTicket;

/// Minimum score to skip LLM and use recipe directly
pub const RECIPE_SKIP_LLM_THRESHOLD: u32 = 70;

/// Result of recipe fast path check
#[derive(Debug)]
pub struct RecipeFastPathResult {
    /// Whether a recipe was matched
    pub matched: bool,
    /// The ticket to use (if matched)
    pub ticket: Option<TranslatorTicket>,
    /// The recipe that was matched
    pub recipe: Option<Recipe>,
    /// Match score
    pub score: u32,
    /// Matched tokens
    pub matched_tokens: Vec<String>,
    /// Whether we can skip the LLM
    pub skip_llm: bool,
    /// v0.0.412: Learned recipe ID (for RecipeStoreV2 recipes)
    pub learned_recipe_id: Option<String>,
}

impl RecipeFastPathResult {
    pub fn no_match() -> Self {
        Self {
            matched: false,
            ticket: None,
            recipe: None,
            score: 0,
            matched_tokens: vec![],
            skip_llm: false,
            learned_recipe_id: None,
        }
    }

    pub fn from_recipe(result: MatchResult) -> Self {
        let skip_llm = result.score >= RECIPE_SKIP_LLM_THRESHOLD && result.high_confidence;
        let ticket = if skip_llm {
            Some(super::converter::ticket_from_recipe(&result.recipe))
        } else {
            None
        };

        Self {
            matched: true,
            ticket,
            recipe: Some(result.recipe),
            score: result.score,
            matched_tokens: result.matched_tokens,
            skip_llm,
            learned_recipe_id: None,
        }
    }
}
