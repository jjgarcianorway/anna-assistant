// v0.0.686: Settings Counter Module (Phase 262)
// Count settings by various criteria

mod types;
mod counter;
mod registry;
mod utils;

// Re-export all public types to maintain API compatibility
pub use types::{
    CountType,
    ValueType,
    CounterConfig,
    CountEntry,
    CountResult,
    CounterStats,
};

pub use counter::SettingsCounter;
pub use registry::CounterRegistry;
pub use utils::{
    format_counter_registry,
    is_counter_query,
    counter_fun_fact,
};
