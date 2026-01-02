// v0.0.785: Settings Retreat (Phase 361)
// Peaceful retreat for settings relaxation

mod types;
mod config;
mod visitor;
mod guide;
mod stats;
mod retreat;
mod registry;
mod utils;

// Re-export all public types to maintain the same API
pub use types::{RetreatType, RetreatStatus};
pub use config::RetreatConfig;
pub use visitor::RetreatVisitor;
pub use guide::RetreatGuide;
pub use stats::RetreatStats;
pub use retreat::SettingsRetreat;
pub use registry::RetreatRegistry;
pub use utils::{format_retreat_registry, is_retreat_query, retreat_fun_fact};
