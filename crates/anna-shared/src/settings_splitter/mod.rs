// v0.0.656: Settings Splitter Module (Phase 232)
// Splitter for dividing settings into separate groups

mod registry;
mod splitter;
mod types;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{
    SplitCriteria, SplitGroup, SplitMode, SplitResult, SplitterConfig, SplitterStats,
};

// Re-export splitter
pub use splitter::SettingsSplitter;

// Re-export registry and utilities
pub use registry::{format_splitter_registry, is_splitter_query, splitter_fun_fact, SettingsSplitterRegistry};
