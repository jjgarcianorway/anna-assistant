// v0.0.600: Settings Aggregator Module (Phase 176)
// Aggregation and summarization of settings

mod aggregator;
mod types;
mod utils;

// Re-export all public types to preserve API
pub use aggregator::SettingsAggregator;
pub use types::{
    AggValue, AggregationDef, AggregationResult, AggregationScope, AggregationType,
    SettingsSummary,
};
pub use utils::{aggregator_fun_fact, format_aggregator, is_aggregator_query};
