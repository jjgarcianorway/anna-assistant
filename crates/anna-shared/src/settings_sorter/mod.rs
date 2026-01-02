// v0.0.675: Settings Sorter Module (Phase 251)
// Sort settings by various criteria and orders

mod types;
mod config;
mod sorter;
mod registry;
mod helpers;
#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{
    SortOrder,
    SortField,
    SortResult,
    SorterStats,
};

pub use config::{
    SorterConfig,
    SortCriteria,
};

pub use sorter::SettingsSorter;
pub use registry::SorterRegistry;
pub use helpers::{
    format_sorter_registry,
    is_sorter_query,
    sorter_fun_fact,
};
