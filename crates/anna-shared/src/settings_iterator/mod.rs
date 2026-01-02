// v0.0.681: Settings Iterator (Phase 257)
// Iterate over settings with various traversal strategies

mod types;
mod config;
mod item;
mod result;
mod stats;
mod iterator;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{IterationFilter, IterationOrder};
pub use config::IteratorConfig;
pub use item::IterationItem;
pub use result::IterationResult;
pub use stats::IteratorStats;
pub use iterator::SettingsIterator;
pub use registry::IteratorRegistry;
pub use helpers::{format_iterator_registry, is_iterator_query, iterator_fun_fact};
