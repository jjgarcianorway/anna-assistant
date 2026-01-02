//! Helper functions for JSON parsing (v0.0.433).

/// Find matching closing brace.
pub(super) fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in s.char_indices() {
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
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_brace() {
        // {} - closing brace at index 1
        assert_eq!(find_matching_brace("{}"), Some(1));
        // {"key": "value"} - 16 chars, closing } at index 15
        assert_eq!(find_matching_brace("{\"key\": \"value\"}"), Some(15));
        // {"nested": {}} - 14 chars, closing } at index 13
        assert_eq!(find_matching_brace("{\"nested\": {}}"), Some(13));
        // {"key": "}"} - tests } inside string (not matching)
        assert_eq!(find_matching_brace("{\"key\": \"}\"}"), Some(11));
        // Unclosed brace
        assert_eq!(find_matching_brace("{"), None);
    }
}
