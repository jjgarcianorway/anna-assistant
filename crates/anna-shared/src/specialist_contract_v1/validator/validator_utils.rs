//! Validation helper functions - v0.0.440.

/// Check for markdown characters.
pub(super) fn check_markdown(text: &str) -> Option<&str> {
    // Check for common markdown patterns
    let patterns = [
        ("```", "code block"),
        ("**", "bold"),
        ("__", "bold"),
        ("##", "heading"),
        ("- ", "list item"),
        ("* ", "list item"),
        ("|", "table"),
    ];

    for (pattern, name) in patterns {
        if text.contains(pattern) {
            return Some(name);
        }
    }

    // Check for heading at start of line
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') && !trimmed.starts_with("{") {
            return Some("heading");
        }
    }

    None
}

/// Extract JSON object from mixed content.
pub(super) fn extract_json(text: &str) -> Option<String> {
    let first_brace = text.find('{')?;
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in text[first_brace..].char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[first_brace..first_brace + i + 1].to_string());
                }
            }
            _ => {}
        }
    }

    None
}

/// Truncate raw response for error logging.
pub(super) fn truncate_raw(raw: &str, max_len: usize) -> String {
    if raw.len() <= max_len {
        raw.to_string()
    } else {
        format!("{}...[truncated]", &raw[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_markdown() {
        assert!(check_markdown("## Heading").is_some());
        assert!(check_markdown("**bold**").is_some());
        assert!(check_markdown("```code```").is_some());
        assert!(check_markdown("plain text").is_none());
    }

    #[test]
    fn test_extract_json_from_mixed() {
        let mixed = r#"Let me analyze this...
        {"case_id": "DSK-0101", "department": "Performance", "assessment": {"summary": "Test", "confidence": 0.9, "risk": "read_only"}}"#;

        let json = extract_json(mixed);
        assert!(json.is_some());
        assert!(json.unwrap().contains("DSK-0101"));
    }
}
