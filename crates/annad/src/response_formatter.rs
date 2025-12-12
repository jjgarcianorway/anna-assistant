//! LLM-based response formatter (v0.0.292).
//!
//! Uses the translator model to rephrase responses for variety
//! while maintaining consistent content and facts.
//!
//! v0.0.292: Added preference-aware formatting with learning mode,
//! verbosity, personality traits integration.

use anna_shared::user_profile::ResponsePreferences;
use tracing::{info, warn};

use crate::ollama;

/// Format a response using the translator LLM for varied phrasing
/// Uses default preferences (loaded from user profile)
pub async fn format_response(
    model: &str,
    original_response: &str,
    query: &str,
    timeout_secs: u64,
) -> String {
    let prefs = ResponsePreferences::load();
    format_response_with_prefs(model, original_response, query, timeout_secs, &prefs).await
}

/// Format a response with explicit preferences
pub async fn format_response_with_prefs(
    model: &str,
    original_response: &str,
    query: &str,
    timeout_secs: u64,
    prefs: &ResponsePreferences,
) -> String {
    // Skip empty or very short responses
    if original_response.len() < 20 {
        return original_response.to_string();
    }

    // Skip if already looks naturally varied (contains informal language)
    if looks_naturally_varied(original_response) {
        return original_response.to_string();
    }

    let prompt = build_format_prompt_with_prefs(original_response, query, prefs);

    match ollama::chat_with_timeout(model, &prompt, timeout_secs).await {
        Ok(response) => {
            let formatted = response.trim().to_string();

            // Validate the response maintains key facts
            if validate_formatted_response(original_response, &formatted) {
                info!(
                    "Response formatted: {} -> {} chars (learning={}, verbosity={})",
                    original_response.len(),
                    formatted.len(),
                    prefs.learning_mode,
                    prefs.verbosity
                );
                formatted
            } else {
                warn!("Formatted response failed validation, using original");
                original_response.to_string()
            }
        }
        Err(e) => {
            warn!("Response formatting failed: {}, using original", e);
            original_response.to_string()
        }
    }
}

/// Check if response already looks naturally varied (not deterministic-style)
fn looks_naturally_varied(response: &str) -> bool {
    // Deterministic responses tend to be very structured
    // Natural responses have more conversational patterns
    let informal_patterns = [
        "I see",
        "Let me",
        "Looking at",
        "It looks like",
        "Based on",
        "I found",
        "Here's what",
        "I noticed",
        "Looks like",
        "appears to",
        "seems to",
    ];

    informal_patterns
        .iter()
        .any(|p| response.to_lowercase().contains(&p.to_lowercase()))
}

/// Build the formatting prompt with user preferences
fn build_format_prompt_with_prefs(
    original: &str,
    query: &str,
    prefs: &ResponsePreferences,
) -> String {
    let mut rules = vec![
        "Keep ALL factual information intact (numbers, paths, commands, settings)".to_string(),
        "Keep technical accuracy - don't change the meaning".to_string(),
        "Make it sound conversational and friendly".to_string(),
        "Vary the sentence structure".to_string(),
        "Don't add new information".to_string(),
        "Don't remove important details".to_string(),
        "NO markdown unless it was in the original".to_string(),
    ];

    // Add verbosity rule
    match prefs.verbosity {
        0 => rules.push("Be VERY brief - minimum words, just essential info".to_string()),
        2 => rules.push("Be detailed - include full explanations and helpful context".to_string()),
        _ => rules.push("Keep it concise but complete".to_string()),
    }

    // Add learning mode rule
    if prefs.learning_mode {
        rules.push("Include a brief 'why this works' explanation if relevant".to_string());
    }

    // Add formality rule
    match prefs.formality {
        0 => rules.push("Use casual, relaxed language".to_string()),
        2 => rules.push("Use professional, formal language".to_string()),
        _ => rules.push("Use friendly but professional tone".to_string()),
    }

    // Add humor rule
    match prefs.humor {
        0 => {} // No humor - don't mention it
        2 => rules.push("Include light humor if appropriate".to_string()),
        _ => rules.push("A subtle touch of warmth is welcome".to_string()),
    }

    // Add technical depth rule
    match prefs.technical_depth {
        0 => rules.push("Avoid jargon, explain any technical terms".to_string()),
        2 => rules.push("Assume technical knowledge, use precise terms".to_string()),
        _ => {} // Balanced - no special rule needed
    }

    let rules_text = rules
        .iter()
        .enumerate()
        .map(|(i, r)| format!("{}. {}", i + 1, r))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"Rephrase this IT support response according to the user's preferences.

USER QUERY: {}

ORIGINAL RESPONSE:
{}

FORMATTING RULES:
{}

Output ONLY the rephrased response, nothing else."#,
        query, original, rules_text
    )
}

/// Validate that the formatted response maintains key facts
fn validate_formatted_response(original: &str, formatted: &str) -> bool {
    // Check length is reasonable
    if formatted.len() < original.len() / 3 || formatted.len() > original.len() * 3 {
        return false;
    }

    // Extract numbers from original and ensure they're preserved
    // Strip trailing punctuation for better matching
    let original_numbers: Vec<String> = original
        .split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_ascii_digit()))
        .map(|w| {
            w.trim_end_matches(|c: char| c.is_ascii_punctuation() && c != '%')
                .to_string()
        })
        .collect();

    // At least 60% of numbers should be preserved
    if !original_numbers.is_empty() {
        let preserved = original_numbers
            .iter()
            .filter(|n| formatted.contains(n.as_str()))
            .count();
        let ratio = preserved as f32 / original_numbers.len() as f32;
        if ratio < 0.6 {
            return false;
        }
    }

    // Check for command/path preservation (things that look like paths or commands)
    let original_paths: Vec<&str> = original
        .split_whitespace()
        .filter(|w| w.contains('/') || w.starts_with('-') || w.contains("="))
        .collect();

    if !original_paths.is_empty() {
        let preserved = original_paths
            .iter()
            .filter(|p| formatted.contains(*p))
            .count();
        let ratio = preserved as f32 / original_paths.len() as f32;
        if ratio < 0.5 {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_naturally_varied() {
        assert!(looks_naturally_varied(
            "I see that your disk is at 85% usage."
        ));
        assert!(looks_naturally_varied("Let me check that for you."));
        assert!(!looks_naturally_varied(
            "Disk usage: 85%. Free space: 15GB."
        ));
    }

    #[test]
    fn test_validate_preserves_numbers() {
        let original = "Memory usage is 85%. You have 4 cores.";
        let good = "Your memory is at 85% and you have 4 CPU cores.";
        let bad = "Memory is high. Multiple cores available.";

        assert!(validate_formatted_response(original, good));
        assert!(!validate_formatted_response(original, bad));
    }

    #[test]
    fn test_validate_preserves_paths() {
        let original = "Edit /etc/ssh/sshd_config and set PermitRootLogin=no.";
        let good = "You should edit /etc/ssh/sshd_config and set PermitRootLogin=no.";
        let bad = "Edit the SSH config and disable root login.";

        assert!(validate_formatted_response(original, good));
        assert!(!validate_formatted_response(original, bad));
    }

    #[test]
    fn test_validate_length_bounds() {
        let original = "This is a test response with some content.";
        let too_short = "Test.";
        let too_long = &"x".repeat(500);

        assert!(!validate_formatted_response(original, too_short));
        assert!(!validate_formatted_response(original, too_long));
    }
}
