//! Utility functions for config parsing.

use regex::Regex;

/// Check if query matches any of the patterns
pub(super) fn matches_any(query: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| query.contains(p))
}

/// Extract email address from text using regex
pub(super) fn extract_email(text: &str) -> Option<String> {
    let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").ok()?;
    re.find(text).map(|m| m.as_str().to_string())
}
