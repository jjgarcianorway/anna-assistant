//! Response validation for specialist outputs (v0.0.421).
//!
//! Validates specialist responses and catches invalid data before
//! it reaches the user.

use super::schema::{SpecialistResponseV2, SpecialistStatus};

/// Result of validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the response is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<String>,
    /// Whether we should use fallback
    pub use_fallback: bool,
    /// Adjusted confidence (may be lowered)
    pub adjusted_confidence: f32,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn ok(confidence: f32) -> Self {
        Self {
            is_valid: true,
            errors: vec![],
            use_fallback: false,
            adjusted_confidence: confidence,
        }
    }

    /// Create a failed validation result
    pub fn failed(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            use_fallback: true,
            adjusted_confidence: 0.0,
        }
    }
}

/// Validate a specialist response
pub fn validate_response(response: &SpecialistResponseV2) -> ValidationResult {
    let mut errors = vec![];
    let mut adjusted_confidence = response.confidence.clamp(0.0, 1.0);

    // Check for forbidden patterns in direct answer
    if let Some(ref answer) = response.direct_answer {
        let text_lower = answer.short_text.to_lowercase();

        for pattern in FORBIDDEN_PATTERNS {
            if text_lower.contains(pattern) {
                errors.push(format!("Forbidden pattern in answer: '{}'", pattern));
            }
        }

        // Check for empty answer when status is ok
        if response.status == SpecialistStatus::Ok && answer.short_text.trim().is_empty() {
            errors.push("Empty answer with status=ok".to_string());
        }

        // Check answer length (too long suggests essay not direct answer)
        if answer.short_text.len() > 500 {
            errors.push("Answer too long (>500 chars), should be concise".to_string());
            adjusted_confidence = adjusted_confidence.min(0.5);
        }
    }

    // Check for status/content consistency
    if response.status == SpecialistStatus::Ok {
        if response.direct_answer.is_none() && response.key_findings.is_empty() {
            errors.push("Status=ok but no direct_answer or key_findings".to_string());
        }

        // High confidence without evidence is suspicious
        if response.confidence > 0.8 && response.citations.is_empty() {
            adjusted_confidence = adjusted_confidence.min(0.6);
        }
    }

    // Check confidence range
    if response.confidence < 0.0 || response.confidence > 1.0 {
        errors.push(format!(
            "Confidence {} out of range [0.0, 1.0]",
            response.confidence
        ));
    }

    // Check notes length (should be brief, not an essay)
    if let Some(ref notes) = response.notes {
        if notes.len() > 300 {
            errors.push("Notes too long (>300 chars), should be brief".to_string());
            adjusted_confidence = adjusted_confidence.min(0.5);
        }
    }

    // Check key findings consistency
    for finding in &response.key_findings {
        if finding.label.is_empty() || finding.value.is_empty() {
            errors.push("Key finding with empty label or value".to_string());
        }
    }

    // Check recommended actions
    for action in &response.recommended_actions {
        if action.label.is_empty() || action.summary.is_empty() {
            errors.push("Recommended action with empty label or summary".to_string());
        }
    }

    // Determine if we should use fallback
    let use_fallback = !errors.is_empty()
        || (response.status == SpecialistStatus::Ok && !response.has_direct_answer());

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        use_fallback,
        adjusted_confidence,
    }
}

/// Parse JSON and validate in one step
pub fn parse_and_validate(json_str: &str) -> Result<(SpecialistResponseV2, ValidationResult), String> {
    // Try to extract JSON from potentially wrapped text
    let clean_json = extract_json(json_str)?;

    // Parse JSON
    let mut response: SpecialistResponseV2 =
        serde_json::from_str(&clean_json).map_err(|e| format!("JSON parse error: {}", e))?;

    // Clamp confidence
    response.clamp_confidence();

    // Validate
    let validation = validate_response(&response);

    // Apply adjusted confidence
    if validation.adjusted_confidence != response.confidence {
        response.confidence = validation.adjusted_confidence;
    }

    Ok((response, validation))
}

/// Extract JSON object from potentially wrapped text
fn extract_json(text: &str) -> Result<String, String> {
    let trimmed = text.trim();

    // Try clean JSON first
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Ok(trimmed.to_string());
    }

    // Try markdown code block
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start..].find("```\n").or(trimmed[start..].rfind("```")) {
            let json_start = start + 7; // len("```json")
            let json_content = &trimmed[json_start..start + end];
            let clean = json_content.trim();
            if clean.starts_with('{') && clean.ends_with('}') {
                return Ok(clean.to_string());
            }
        }
    }

    // Try bare code block
    if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed[start + 3..].find("```") {
            let json_content = &trimmed[start + 3..start + 3 + end];
            let clean = json_content.trim();
            if clean.starts_with('{') && clean.ends_with('}') {
                return Ok(clean.to_string());
            }
        }
    }

    // Last resort: find first { and last }
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            return Ok(trimmed[start..=end].to_string());
        }
    }

    Err("No valid JSON object found in response".to_string())
}

/// Forbidden patterns that indicate hallucination or parse bugs
const FORBIDDEN_PATTERNS: &[&str] = &[
    "unknown is installed",
    "unknown is not installed",
    "**unknown**",
    "2 is installed",
    "1 is installed",
    "i don't have access",
    "i cannot determine",
    "as an ai",
    "i'm unable to",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_v2::answer::DirectAnswer;

    #[test]
    fn test_validate_good_response() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple("17.0 GiB available"))
            .with_confidence(0.9)
            .with_citation("probe:free");

        let result = validate_response(&response);
        assert!(result.is_valid);
        assert!(!result.use_fallback);
    }

    #[test]
    fn test_validate_forbidden_pattern() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple("unknown is installed"))
            .with_confidence(0.9);

        let result = validate_response(&response);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Forbidden pattern")));
    }

    #[test]
    fn test_validate_empty_answer() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple(""))
            .with_confidence(0.9);

        let result = validate_response(&response);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_extract_json() {
        // Clean JSON
        let clean = extract_json(r#"{"status": "ok"}"#);
        assert!(clean.is_ok());

        // Markdown code block
        let md = extract_json("Here is the response:\n```json\n{\"status\": \"ok\"}\n```");
        assert!(md.is_ok());

        // With surrounding text
        let wrapped = extract_json("Some text {\"status\": \"ok\"} more text");
        assert!(wrapped.is_ok());
    }

    #[test]
    fn test_confidence_adjustment() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple("Test"))
            .with_confidence(0.95); // High confidence, no citations

        let result = validate_response(&response);
        assert!(result.adjusted_confidence <= 0.6); // Should be lowered
    }
}
