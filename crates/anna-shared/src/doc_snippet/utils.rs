//! Utility functions for documentation snippets.

use super::types::DocSourceKind;

/// Extract keywords from doc content
pub fn extract_doc_keywords(text: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "is", "are", "to", "of", "in", "for", "with", "on", "at",
    ];
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// Compute unique ID for a snippet
pub fn compute_snippet_id(kind: DocSourceKind, reference: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    kind.to_string().hash(&mut hasher);
    reference.hash(&mut hasher);
    format!("doc_{:016x}", hasher.finish())
}

/// Get current time in seconds since UNIX epoch
pub fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
