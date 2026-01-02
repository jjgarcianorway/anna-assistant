// v0.0.673: Settings Selector Module (Phase 249)
// Select settings based on criteria and patterns

mod types;
mod config;
mod criteria;
mod result;
mod stats;
mod selector;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{SelectorType, MatchMode};
pub use config::SelectorConfig;
pub use criteria::SelectionCriteria;
pub use result::SelectionResult;
pub use stats::SelectorStats;
pub use selector::SettingsSelector;
pub use registry::{SelectorRegistry, format_selector_registry};
pub use utils::{is_selector_query, selector_fun_fact};
