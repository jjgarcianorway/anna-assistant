//! Query scenario corpus for comprehensive testing (v0.0.268).
//!
//! 100+ queries categorized by:
//! - Expected team routing
//! - Expected junior/senior involvement
//! - Difficulty (simple vs complex)
//! - Whether recipe learning should apply

pub mod corpus;
mod stats;
#[cfg(test)]
mod helpers;
#[cfg(test)]
mod corpus_tests;
#[cfg(test)]
mod routing_tests;
#[cfg(test)]
mod fast_path_tests;
#[cfg(test)]
mod recipe_learning_tests;

pub use corpus::{Difficulty, ExpectedPath, QueryScenario, ScenarioCorpus};
pub use stats::{ResolutionOutcome, ScenarioStats, TeamStats};
