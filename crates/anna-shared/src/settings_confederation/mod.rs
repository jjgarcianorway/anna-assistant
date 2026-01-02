// v0.0.738: Settings Confederation (Phase 314)
// Loose union for settings governance

mod types;
mod config;
mod article;
mod member;
mod stats;
mod confederation;
mod registry;
mod utils;

// Re-export all public items to preserve API
pub use types::{ConfederationType, ConfederationStatus};
pub use config::ConfederationConfig;
pub use article::ConfederationArticle;
pub use member::ConfederationMember;
pub use stats::ConfederationStats;
pub use confederation::SettingsConfederation;
pub use registry::ConfederationRegistry;
pub use utils::{format_confederation_registry, is_confederation_query, confederation_fun_fact};
