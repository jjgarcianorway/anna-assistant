//! Helper functions for knowledge fetching.

use std::collections::HashSet;

/// Extract keywords from question
pub(super) fn extract_keywords(question: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "i", "my",
        "me", "you", "your", "we", "our", "they", "their", "it", "its", "this", "that", "what",
        "which", "who", "whom", "how", "why", "when", "where", "to", "of", "in", "on", "at", "by",
        "for", "with", "about", "into", "through", "during", "before", "after", "above", "below",
        "from", "up", "down", "out", "off", "over", "under", "again", "further", "then", "once",
    ]
    .into_iter()
    .collect();

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// Extract summary from content (first N sentences)
pub(super) fn extract_summary(content: &str, sentences: usize) -> String {
    let mut result = String::new();
    let mut count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip section headers (all caps)
        if trimmed
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_whitespace())
            && trimmed.len() < 30
        {
            continue;
        }

        result.push_str(trimmed);
        result.push(' ');
        count += 1;

        if count >= sentences {
            break;
        }
    }

    result.trim().to_string()
}

/// Extract key points containing keywords
pub(super) fn extract_key_points(content: &str, keywords: &[String], max: usize) -> Vec<String> {
    let mut points = vec![];

    for line in content.lines() {
        if points.len() >= max {
            break;
        }

        let line_lower = line.to_lowercase();
        let has_keyword = keywords.iter().any(|k| line_lower.contains(k));

        if has_keyword && line.len() > 10 && line.len() < 200 {
            points.push(line.trim().to_string());
        }
    }

    points
}

/// Count keyword matches in content
pub(super) fn count_keyword_matches(content: &str, keywords: &[String]) -> usize {
    let content_lower = content.to_lowercase();
    keywords
        .iter()
        .filter(|k| content_lower.contains(k.as_str()))
        .count()
}

/// Check if topic looks like a command
pub(super) fn is_command_like(topic: &str) -> bool {
    let topic_lower = topic.to_lowercase();
    topic_lower
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !topic_lower.contains(' ')
        && topic.len() < 30
}

/// Check if topic looks like a package name
pub(super) fn is_package_like(topic: &str) -> bool {
    is_command_like(topic) && !topic.starts_with('-')
}
