//! Formatting utilities for specialist responses.

use super::types::ParseOutcome;

/// Format a parse failure for user display
pub fn format_parse_failure(outcome: &ParseOutcome, suggestions: &[String]) -> String {
    let mut output = match outcome {
        ParseOutcome::NoJson { .. } => {
            "I had an internal error: the specialist did not return valid data.".to_string()
        }
        ParseOutcome::InvalidJson { error, .. } => {
            format!(
                "I had an internal error parsing the response: {}",
                truncate(error, 100)
            )
        }
        ParseOutcome::SchemaError { errors, .. } => {
            format!("The specialist response was invalid: {}", errors.join(", "))
        }
        ParseOutcome::Timeout { elapsed_secs } => {
            format!("The specialist timed out after {}s.", elapsed_secs)
        }
        ParseOutcome::Success(_) => return String::new(),
    };

    if !suggestions.is_empty() {
        output.push_str("\n\nYou can try these manual steps:");
        for s in suggestions.iter().take(5) {
            output.push_str(&format!("\n  - {}", s));
        }
    }

    output
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
