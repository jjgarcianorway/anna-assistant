//! Helper functions for JSON repair and extraction.

/// Remove trailing commas from JSON (common model error).
pub(super) fn remove_trailing_commas(json: &str) -> String {
    // Simple regex-free approach: remove comma before } or ]
    let mut result = json.to_string();

    // Remove ", }" patterns
    while result.contains(", }") {
        result = result.replace(", }", " }");
    }
    while result.contains(",}") {
        result = result.replace(",}", "}");
    }

    // Remove ", ]" patterns
    while result.contains(", ]") {
        result = result.replace(", ]", " ]");
    }
    while result.contains(",]") {
        result = result.replace(",]", "]");
    }

    // Remove trailing comma before closing (with newlines)
    let lines: Vec<&str> = result.lines().collect();
    if lines.len() > 1 {
        let mut new_lines = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if i < lines.len() - 1 {
                let next_trimmed = lines.get(i + 1).map(|l| l.trim()).unwrap_or("");
                if trimmed.ends_with(',')
                    && (next_trimmed.starts_with('}') || next_trimmed.starts_with(']'))
                {
                    new_lines.push(line.trim_end_matches(','));
                    continue;
                }
            }
            new_lines.push(line);
        }
        result = new_lines.join("\n");
    }

    result
}

/// Extract the largest JSON object from text.
pub(super) fn extract_json_object(text: &str) -> Option<String> {
    // Find first '{' and last matching '}'
    let first_brace = text.find('{')?;

    // Count braces to find matching close
    let mut depth = 0;
    let mut last_close = None;
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
                    last_close = Some(first_brace + i);
                    break;
                }
            }
            _ => {}
        }
    }

    if let Some(end) = last_close {
        Some(text[first_brace..=end].to_string())
    } else {
        None
    }
}

/// Truncate output for error messages.
pub(super) fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        format!("{}...[truncated]", &output[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_object() {
        let text = "Some text { \"key\": \"value\" } more text";
        let json = extract_json_object(text);
        assert!(json.is_some());
        assert!(json.unwrap().contains("key"));

        let nested = r#"Prefix {"outer": {"inner": 1}} suffix"#;
        let json = extract_json_object(nested);
        assert!(json.is_some());
        assert!(json.unwrap().contains("inner"));
    }

    #[test]
    fn test_remove_trailing_commas() {
        let with_comma = r#"{"a": 1, "b": 2,}"#;
        let fixed = remove_trailing_commas(with_comma);
        assert!(!fixed.ends_with(",}"));

        let array_comma = r#"[1, 2, 3,]"#;
        let fixed = remove_trailing_commas(array_comma);
        assert!(!fixed.ends_with(",]"));
    }
}
