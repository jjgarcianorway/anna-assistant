// v0.0.733: Settings Convention (Phase 309)
// Formal convention for settings governance

mod types;
mod config;
mod article;
mod party;
mod stats;
mod core;
mod registry;
mod utils;

// Re-export all public types and functions
pub use types::{ConventionType, ConventionStatus};
pub use config::ConventionConfig;
pub use article::ConventionArticle;
pub use party::ConventionParty;
pub use stats::ConventionStats;
pub use core::SettingsConvention;
pub use registry::{ConventionRegistry, format_convention_registry};
pub use utils::{is_convention_query, convention_fun_fact};
