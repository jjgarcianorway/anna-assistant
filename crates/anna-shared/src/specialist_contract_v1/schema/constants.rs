//! Constants and helper functions for SRC v1.

/// Maximum characters for summary.
pub const MAX_SUMMARY_CHARS: usize = 140;

/// Maximum actions per response.
pub const MAX_ACTIONS: usize = 5;

/// Maximum citations per response.
pub const MAX_CITATIONS: usize = 5;

/// Maximum characters for citation snippet.
pub const MAX_SNIPPET_CHARS: usize = 140;

/// Truncate string to max length.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_str() {
        let short = "hello";
        assert_eq!(truncate_str(short, 10), "hello");

        let long = "a".repeat(200);
        let truncated = truncate_str(&long, 50);
        assert!(truncated.len() <= 50);
        assert!(truncated.ends_with("..."));
    }
}
