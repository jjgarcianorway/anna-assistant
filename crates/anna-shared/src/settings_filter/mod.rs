// v0.0.674: Settings Filter Module (Phase 250)
// Filter settings with predicates and conditions

mod types;
mod filter;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{
    FilterType,
    FilterPredicate,
    FilterConfig,
    FilterRule,
    FilterResult,
    FilterStats,
};

// Re-export main filter implementation
pub use filter::SettingsFilter;

// Re-export registry
pub use registry::FilterRegistry;

// Re-export helper functions
pub use helpers::{
    format_filter_registry,
    is_filter_query,
    filter_fun_fact,
};
