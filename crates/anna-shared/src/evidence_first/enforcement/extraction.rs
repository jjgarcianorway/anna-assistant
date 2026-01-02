//! Claim extraction from response text.

use super::claim::{Claim, ClaimType};

/// Extract claims from response text (simple heuristic).
pub fn extract_claims(text: &str) -> Vec<Claim> {
    let mut claims = Vec::new();

    // Patterns that indicate factual claims
    let factual_patterns = [
        "is running",
        "is not running",
        "is installed",
        "is not installed",
        "takes",
        "uses",
        "shows",
        "indicates",
        "currently",
        "GB",
        "MB",
        "seconds",
        "ms",
        "%",
    ];

    // Patterns that indicate documentation claims
    let doc_patterns = [
        "according to",
        "the manual",
        "documentation",
        "man page",
        "wiki",
        "means",
        "is used to",
        "allows",
    ];

    // Patterns that indicate uncertainty
    let uncertainty_patterns = [
        "might",
        "may",
        "could",
        "possibly",
        "I'm not sure",
        "unclear",
        "unknown",
    ];

    for sentence in text.split('.').filter(|s| !s.trim().is_empty()) {
        let sentence_lower = sentence.to_lowercase();

        // Check for uncertainty first
        if uncertainty_patterns
            .iter()
            .any(|p| sentence_lower.contains(p))
        {
            claims.push(Claim::new(sentence.trim(), ClaimType::Uncertainty));
            continue;
        }

        // Check for factual claims
        if factual_patterns.iter().any(|p| sentence_lower.contains(p)) {
            claims.push(Claim::factual(sentence.trim()));
            continue;
        }

        // Check for documentation claims
        if doc_patterns.iter().any(|p| sentence_lower.contains(p)) {
            claims.push(Claim::documentation(sentence.trim()));
            continue;
        }

        // Default to recommendation if contains action words
        if sentence_lower.contains("should")
            || sentence_lower.contains("recommend")
            || sentence_lower.contains("try")
        {
            claims.push(Claim::recommendation(sentence.trim()));
        }
    }

    claims
}
