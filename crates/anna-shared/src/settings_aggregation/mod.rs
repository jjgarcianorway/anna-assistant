// v0.0.671: Settings Aggregation (Phase 247)
// Aggregate settings values with various functions

mod aggregator;
mod registry;
mod stats;
mod types;
mod utils;

// Re-export all public items to maintain the same API
pub use aggregator::SettingsAggregator;
pub use registry::{format_aggregator_registry, AggregatorRegistry};
pub use stats::AggregatorStats;
pub use types::{
    AggregateEntry, AggregateFunction, AggregationResult, AggregatorConfig, GroupByType,
};
pub use utils::{aggregator_fun_fact, is_aggregator_query};
