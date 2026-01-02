// v0.0.690: Settings Combiner Module (Phase 266)
// Merge multiple settings collections

mod types;
mod combiner;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{
    CombineStrategy,
    CombineDepth,
    CombinerConfig,
    CombineConflict,
    CombineResult,
    CombinerStats,
};

pub use combiner::SettingsCombiner;

pub use registry::{
    CombinerRegistry,
    format_combiner_registry,
};

pub use utils::{
    is_combiner_query,
    combiner_fun_fact,
};
