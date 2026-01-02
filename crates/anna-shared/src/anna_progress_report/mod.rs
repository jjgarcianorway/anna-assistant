//! Anna Progress Report (Phase 72)
//!
//! Generates comprehensive progress reports showing Anna's learning journey,
//! improvements over time, and achievements.
//!
//! This module is organized into:
//! - `types`: Core data structures for progress tracking
//! - `formatters`: Output formatting functions
//! - `utils`: Utility functions and helpers

pub mod formatters;
pub mod types;
pub mod utils;

// Re-export all public items to preserve the original API
pub use formatters::{
    format_progress_report, format_progress_report_compact, format_progress_report_oneline,
    progress_bar, progress_summary_message,
};
pub use types::{
    Milestone, PeriodSnapshot, ProgressMetric, ProgressReport, TimePeriod, Trend,
};
pub use utils::{calculate_change_percent, calculate_trend, default_milestones, is_progress_query};
