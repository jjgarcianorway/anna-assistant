//! Query Pattern Analyzer - Phase 95
//!
//! Analyzes query patterns to improve Anna's understanding.
//! Learns from common query structures for better matching.

mod analyzer;
mod formatters;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use analyzer::QueryPatternAnalyzer;
pub use formatters::{
    format_pattern_analyzer, format_pattern_analyzer_compact, format_pattern_analyzer_oneline,
    pattern_fun_fact,
};
pub use types::{ConfidenceLevel, PatternCategory, QueryPattern};
pub use utils::{is_pattern_query, COMMON_PATTERNS};
