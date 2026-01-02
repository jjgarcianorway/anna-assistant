// v0.0.677: Settings Reducer Module (Phase 253)
// Reduce settings to aggregated values

mod types;
mod config;
mod result;
mod stats;
mod reducer;
mod registry;
mod utils;

// Re-export all public types to preserve the API
pub use types::{ReduceOp, ReduceTarget};
pub use config::ReducerConfig;
pub use result::{ReducedValue, ReduceResult};
pub use stats::ReducerStats;
pub use reducer::SettingsReducer;
pub use registry::{ReducerRegistry, format_reducer_registry};
pub use utils::{is_reducer_query, reducer_fun_fact};
