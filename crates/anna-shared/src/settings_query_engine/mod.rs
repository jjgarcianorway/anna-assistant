// v0.0.670: Settings Query Engine Module (Phase 246)
// Query engine for complex settings queries

pub mod config;
pub mod engine;
pub mod registry;
pub mod stats;
pub mod types;
pub mod utils;

// Re-export main types for API compatibility
pub use config::QueryEngineConfig;
pub use engine::SettingsQueryEngine;
pub use registry::{format_query_engine_registry, QueryEngineRegistry};
pub use stats::QueryEngineStats;
pub use types::{Query, QueryCondition, QueryOperator, QueryResult, QueryType};
pub use utils::{is_query_engine_query, query_engine_fun_fact};
