// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration

mod types;
mod config;
mod provision;
mod member;
mod stats;
mod union;
mod registry;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{UnionType, UnionStatus};
pub use config::UnionConfig;
pub use provision::UnionProvision;
pub use member::UnionMember;
pub use stats::UnionStats;
pub use union::SettingsUnion;
pub use registry::{UnionRegistry, format_union_registry, is_union_query, union_fun_fact};
