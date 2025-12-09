//! Answer trimming functions (v0.0.209).

use super::contract::AnswerContract;
use super::types::{RequestedField, Verbosity};

/// Trim answer to only include requested fields (best effort)
/// Returns None if trimming is not possible
pub fn trim_answer(answer: &str, contract: &AnswerContract) -> Option<String> {
    // Don't trim in teaching mode or if generic
    if contract.teaching_mode || contract.requested_fields.contains(&RequestedField::Generic) {
        return Some(answer.to_string());
    }

    // For minimal mode with single specific field, try to extract just that value
    if contract.verbosity == Verbosity::Minimal && contract.requested_fields.len() == 1 {
        match &contract.requested_fields[0] {
            RequestedField::CpuCores => extract_number_with_context(answer, &["core", "thread"]),
            RequestedField::RamFree => extract_size_with_context(answer, &["free", "available"]),
            RequestedField::RamTotal => extract_size_with_context(answer, &["total"]),
            _ => Some(answer.to_string()),
        }
    } else {
        Some(answer.to_string())
    }
}

/// Extract a number with context keywords
fn extract_number_with_context(text: &str, contexts: &[&str]) -> Option<String> {
    let text_lower = text.to_lowercase();

    for context in contexts {
        if let Some(pos) = text_lower.find(context) {
            // Look for number before or after the context word
            let before = &text[..pos];
            let after = &text[pos..];

            // Try to extract number from nearby
            for word in before.split_whitespace().rev().take(3) {
                if let Ok(n) = word
                    .trim_matches(|c: char| !c.is_ascii_digit())
                    .parse::<u32>()
                {
                    return Some(format!("{} {}", n, context));
                }
            }
            for word in after.split_whitespace().take(5) {
                if let Ok(n) = word
                    .trim_matches(|c: char| !c.is_ascii_digit())
                    .parse::<u32>()
                {
                    return Some(format!("{} {}", n, context));
                }
            }
        }
    }

    None
}

/// Extract a size value (like "8 GB") with context keywords
fn extract_size_with_context(text: &str, contexts: &[&str]) -> Option<String> {
    let text_lower = text.to_lowercase();

    for context in contexts {
        if text_lower.contains(context) {
            // Look for size patterns like "8 GB", "1.5GB", "512 MB"
            for word in text.split_whitespace() {
                let w = word.to_uppercase();
                if w.ends_with("GB") || w.ends_with("MB") || w.ends_with("TB") {
                    return Some(word.to_string());
                }
            }
        }
    }

    None
}
