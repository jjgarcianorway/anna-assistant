//! Query Pattern Analyzer Utilities
//!
//! Helper functions and constants for query pattern analysis.

use super::types::PatternCategory;

/// Check if query is about patterns
pub fn is_pattern_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "pattern",
        "patterns",
        "query pattern",
        "how do i ask",
        "query analysis",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Common query patterns
pub const COMMON_PATTERNS: &[(&str, PatternCategory)] = &[
    ("how to {action} {target}", PatternCategory::Question),
    ("what is {target}", PatternCategory::Question),
    ("why is {target} {state}", PatternCategory::Troubleshoot),
    ("install {package}", PatternCategory::Install),
    ("remove {package}", PatternCategory::Command),
    ("start {service}", PatternCategory::Command),
    ("stop {service}", PatternCategory::Command),
    ("restart {service}", PatternCategory::Command),
    ("status of {target}", PatternCategory::Status),
    ("show {target}", PatternCategory::Status),
    ("configure {target}", PatternCategory::Config),
    ("set {setting} to {value}", PatternCategory::Config),
];
