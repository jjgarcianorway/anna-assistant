// v0.0.680: Settings Expander Module (Phase 256)
// Expand settings with variables and templates

mod types;
mod config;
mod result;
mod stats;
mod expander;
mod registry;
mod utils;

// Re-export public API
pub use types::{ExpandMode, VariableSyntax};
pub use config::ExpanderConfig;
pub use result::ExpandResult;
pub use stats::ExpanderStats;
pub use expander::SettingsExpander;
pub use registry::ExpanderRegistry;
pub use utils::{format_expander_registry, is_expander_query, expander_fun_fact};
