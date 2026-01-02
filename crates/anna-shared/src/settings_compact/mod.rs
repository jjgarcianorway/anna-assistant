// v0.0.729: Settings Compact (Phase 305)
// Formal compact for settings governance

mod types;
mod config;
mod term;
mod member;
mod stats;
mod compact;
mod registry;
mod utils;

// Re-export all public types to preserve the original API
pub use types::{CompactType, CompactStatus};
pub use config::CompactConfig;
pub use term::CompactTerm;
pub use member::CompactMember;
pub use stats::CompactStats;
pub use compact::SettingsCompact;
pub use registry::CompactRegistry;
pub use utils::{format_compact_registry, is_compact_query, compact_fun_fact};
