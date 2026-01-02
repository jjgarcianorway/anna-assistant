// v0.0.711: Settings Summary Module (Phase 287)
// Comprehensive summary of settings state

mod types;
mod summary;
mod registry;
mod helpers;

// Re-export all public types and functions
pub use types::{
    SummaryType,
    SummaryDepth,
    SummaryConfig,
    SummaryEntry,
    SummaryMetadata,
    SummaryStats,
};

pub use summary::SettingsSummary;
pub use registry::SummaryRegistry;
pub use helpers::{
    format_summary_registry,
    is_summary_query,
    summary_fun_fact,
};
