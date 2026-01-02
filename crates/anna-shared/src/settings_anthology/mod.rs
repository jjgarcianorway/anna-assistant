// v0.0.701: Settings Anthology (Phase 277)
// Curated anthology of settings collections

mod anthology;
mod helpers;
mod stats;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use anthology::{AnthologyRegistry, SettingsAnthology};
pub use helpers::{anthology_fun_fact, format_anthology_registry, is_anthology_query};
pub use stats::AnthologyStats;
pub use types::{AnthologyConfig, AnthologyPiece, AnthologyStatus, AnthologyType, AnthologyWork};
