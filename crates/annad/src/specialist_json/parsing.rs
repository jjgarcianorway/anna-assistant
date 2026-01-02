//! JSON parsing functions for specialist responses

use anna_shared::specialist_contract::SpecialistResponse;
use tracing::warn;

/// Parse JSON from LLM output, handling common issues
/// v0.0.409: Now includes validation for forbidden patterns
pub fn parse_specialist_json(raw: &str, ticket_id: &str) -> Result<SpecialistResponse, String> {
    // Try to find JSON object in the output
    let json_str = extract_json_object(raw)?;

    // Parse JSON
    let mut response: SpecialistResponse =
        serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    // v0.0.409: Validate for forbidden patterns
    let validation_errors = response.validate();
    if !validation_errors.is_empty() {
        warn!(
            "Specialist response validation failed for {}: {:?}",
            ticket_id, validation_errors
        );
        // Downgrade confidence and mark as error if validation fails
        response.confidence = response.confidence.min(0.3);
        if let Some(ref mut staff_view) = response.staff_view {
            staff_view.mood = anna_shared::specialist_contract::Mood::Blocked;
            staff_view.short_note = Some(format!(
                "Validation failed: {}",
                validation_errors.join(", ")
            ));
        }
        // If forbidden pattern detected, this is an error
        if validation_errors
            .iter()
            .any(|e| e.contains("forbidden pattern"))
        {
            return Err(format!(
                "Validation failed: {}",
                validation_errors.join(", ")
            ));
        }
    }

    Ok(response)
}

/// Extract JSON object from LLM output
/// Handles cases where LLM adds prose before/after JSON
pub fn extract_json_object(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();

    // If it starts with {, try to find matching }
    if trimmed.starts_with('{') {
        // Find the last } which should close the object
        if let Some(end) = trimmed.rfind('}') {
            return Ok(trimmed[..=end].to_string());
        }
    }

    // Try to find JSON block in markdown
    if let Some(start) = raw.find("```json") {
        let after_marker = &raw[start + 7..];
        if let Some(end) = after_marker.find("```") {
            return Ok(after_marker[..end].trim().to_string());
        }
    }

    // Try to find bare code block
    if let Some(start) = raw.find("```") {
        let after_marker = &raw[start + 3..];
        if let Some(end) = after_marker.find("```") {
            let content = after_marker[..end].trim();
            if content.starts_with('{') {
                return Ok(content.to_string());
            }
        }
    }

    // Look for { anywhere in the output
    if let Some(start) = raw.find('{') {
        if let Some(end) = raw.rfind('}') {
            if end > start {
                return Ok(raw[start..=end].to_string());
            }
        }
    }

    Err("No JSON object found in output".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_object() {
        // Clean JSON
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok"}"#;
        assert!(extract_json_object(json).is_ok());

        // JSON with surrounding prose
        let with_prose = r#"Here is the response:
{"ticket_id": "DSK-0101", "status": "ok"}
Done."#;
        let extracted = extract_json_object(with_prose).unwrap();
        assert!(extracted.contains("ticket_id"));

        // JSON in markdown block
        let markdown = r#"```json
{"ticket_id": "DSK-0101", "status": "ok"}
```"#;
        assert!(extract_json_object(markdown).is_ok());
    }

    #[test]
    fn test_parse_specialist_json() {
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok", "answer": {"short": "Test"}, "evidence": [], "confidence": 0.9}"#;
        let result = parse_specialist_json(json, "DSK-0101");
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.answer.short, "Test");
    }

    #[test]
    fn test_validation_rejects_unknown_is_installed() {
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok", "answer": {"short": "unknown is installed"}, "evidence": [], "confidence": 0.9}"#;
        let result = parse_specialist_json(json, "DSK-0101");
        // Should fail validation due to forbidden pattern
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("forbidden pattern"));
    }

    #[test]
    fn test_validation_rejects_number_as_package() {
        let json = r#"{"ticket_id": "DSK-0101", "status": "ok", "answer": {"short": "2 is installed on your system"}, "evidence": [], "confidence": 0.9}"#;
        let result = parse_specialist_json(json, "DSK-0101");
        assert!(result.is_err());
    }
}
