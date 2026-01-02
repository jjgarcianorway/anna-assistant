// v0.0.765: Settings Grove (Phase 341)
// Tree grove for settings forestry

mod types;
mod config;
mod tree;
mod tender;
mod stats;
mod grove;
mod registry;
mod utils;

// Public re-exports to preserve API
pub use types::{GroveType, GroveStatus};
pub use config::GroveConfig;
pub use tree::GroveTree;
pub use tender::GroveTender;
pub use stats::GroveStats;
pub use grove::SettingsGrove;
pub use registry::GroveRegistry;
pub use utils::{format_grove_registry, is_grove_query, grove_fun_fact};
