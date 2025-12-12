//! Query scenario corpus for comprehensive testing (v0.0.268).
//!
//! 100+ queries categorized by:
//! - Expected team routing
//! - Expected junior/senior involvement
//! - Difficulty (simple vs complex)
//! - Whether recipe learning should apply

mod corpus;
mod stats;
#[cfg(test)]
mod tests;

pub use corpus::{Difficulty, ExpectedPath, QueryScenario, ScenarioCorpus};
pub use stats::{ResolutionOutcome, ScenarioStats, TeamStats};
