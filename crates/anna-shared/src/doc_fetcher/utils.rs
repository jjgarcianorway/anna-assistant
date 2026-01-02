//! Utility functions for doc fetchers.

use crate::evidence_engine::DocSnippet;

/// Extract section relevant to topic
pub fn extract_relevant_section(content: &str, topic: &str, max_len: usize) -> String {
    let topic_lower = topic.to_lowercase();
    let content_lower = content.to_lowercase();

    // Find first occurrence of topic
    if let Some(pos) = content_lower.find(&topic_lower) {
        let start = pos.saturating_sub(50);
        let end = (pos + max_len - 50).min(content.len());

        // Extend to line boundaries
        let slice = &content[start..end];
        return truncate_to_lines(slice, max_len);
    }

    // Fallback: return beginning
    truncate_to_lines(content, max_len)
}

/// Truncate to line boundaries
fn truncate_to_lines(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }

    let truncated = &s[..max];
    if let Some(pos) = truncated.rfind('\n') {
        format!("{}...", &truncated[..pos])
    } else {
        format!("{}...", truncated)
    }
}

/// Simple truncate
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Shell escape a string
pub fn shell_escape(s: &str) -> String {
    // Basic escaping - just alphanumeric and some safe chars
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('\'');
    for c in s.chars() {
        if c == '\'' {
            escaped.push_str("'\"'\"'");
        } else {
            escaped.push(c);
        }
    }
    escaped.push('\'');
    escaped
}

/// Deduplicate docs by title
pub fn dedup_docs(docs: &mut Vec<DocSnippet>) {
    let mut seen = std::collections::HashSet::new();
    docs.retain(|d| seen.insert(d.title.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("test"), "'test'");
        assert_eq!(shell_escape("test's"), "'test'\"'\"'s'");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_extract_relevant_section() {
        let content = "First line\nSecond line about vim\nThird line";
        let section = extract_relevant_section(content, "vim", 100);
        assert!(section.contains("vim"));
    }
}
