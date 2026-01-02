//! Follow-up hint formatting functions.

use super::types::FollowupHint;

/// Format hints as a string to append to answer
pub fn format_hints(hints: &[FollowupHint]) -> String {
    if hints.is_empty() {
        return String::new();
    }

    let mut output = String::from("\n\n---\n**Related:**");

    for hint in hints {
        if let Some(cmd) = &hint.command {
            output.push_str(&format!("\n- {} → `{}`", hint.hint, cmd));
        } else {
            output.push_str(&format!("\n- {}", hint.hint));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hints() {
        let hints = vec![FollowupHint {
            hint: "Test hint".to_string(),
            command: Some("test cmd".to_string()),
            relevance: 80,
        }];
        let formatted = format_hints(&hints);
        assert!(formatted.contains("Related"));
        assert!(formatted.contains("test cmd"));
    }
}
