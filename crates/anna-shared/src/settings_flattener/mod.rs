// v0.0.679: Settings Flattener (Phase 255)
// Flatten nested settings structures

mod types;
mod flattener;
mod registry;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{
    FlattenMode,
    DepthLimit,
    FlattenerConfig,
    FlattenResult,
    FlattenerStats,
};

// Re-export flattener
pub use flattener::SettingsFlattener;

// Re-export registry and helper functions
pub use registry::{
    FlattenerRegistry,
    format_flattener_registry,
    is_flattener_query,
    flattener_fun_fact,
};
