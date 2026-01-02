//! JSON extraction from raw text with multiple fallback strategies.

/// Extract JSON object from raw text using multiple strategies
pub fn extract_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // Strategy 1: Clean JSON object at start
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed.to_string());
    }

    // Strategy 2: Markdown JSON code block
    if let Some(start) = trimmed.find("```json") {
        let after_marker = &trimmed[start + 7..];
        if let Some(end) = after_marker.find("```") {
            let json_content = after_marker[..end].trim();
            if json_content.starts_with('{') && json_content.ends_with('}') {
                return Some(json_content.to_string());
            }
        }
    }

    // Strategy 3: Generic code block
    if let Some(start) = trimmed.find("```") {
        let after_marker = &trimmed[start + 3..];
        // Skip language identifier if present
        let content_start = after_marker.find('\n').map(|i| i + 1).unwrap_or(0);
        let after_newline = &after_marker[content_start..];
        if let Some(end) = after_newline.find("```") {
            let json_content = after_newline[..end].trim();
            if json_content.starts_with('{') && json_content.ends_with('}') {
                return Some(json_content.to_string());
            }
        }
    }

    // Strategy 4: Find first { to last }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                return Some(trimmed[start..=end].to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_strategies() {
        // Clean JSON
        assert!(extract_json(r#"{"test": 1}"#).is_some());

        // With whitespace
        assert!(extract_json(r#"   {"test": 1}   "#).is_some());

        // In markdown block
        assert!(extract_json("```json\n{\"test\": 1}\n```").is_some());

        // Embedded in text
        assert!(extract_json("Here is the result: {\"test\": 1} end").is_some());

        // No JSON
        assert!(extract_json("No JSON here").is_none());
    }
}
