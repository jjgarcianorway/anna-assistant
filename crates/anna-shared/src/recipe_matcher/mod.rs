//! Recipe matcher for fast-path resolution (v0.0.373).
//!
//! The translator uses this to check if a learned recipe can answer a query
//! WITHOUT calling the LLM specialist. This is the key to Anna's learning:
//!
//! 1. First query: Specialist (LLM) generates answer -> Recipe is learned
//! 2. Similar query: Translator finds matching recipe -> No LLM needed!
//!
//! The matcher uses semantic similarity based on:
//! - Intent (what the user wants to do)
//! - Target (what they want to do it to)
//! - Action verbs (enable, install, configure, etc.)
//!
//! v0.0.373: Dynamic thresholds based on recipe maturity and reliability.

mod action_matching;
mod config_matching;
mod helpers;
mod matching;
mod substitutions;
mod threshold;
mod types;
mod utils;

// Re-export public API
pub use action_matching::match_action_recipe;
pub use config_matching::match_config_recipe;
pub use matching::match_recipe;
pub use types::{MatchResult, BASE_MATCH_THRESHOLD, MIN_MATCHING_TOKENS};
pub use utils::{load_recipe_index, match_ssh_recipe, recipe_count};
