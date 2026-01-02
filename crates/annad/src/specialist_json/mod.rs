//! JSON-only specialist handler (v0.0.404).
//!
//! This module implements the new architecture where:
//! - Specialists ONLY output JSON (no prose, no personality)
//! - JSON is parsed into SpecialistResponse struct
//! - Personality is added by the renderer layer
//!
//! Key benefits:
//! - Deterministic parsing (JSON is parseable)
//! - LLM can't hallucinate prose or wrong formats
//! - Personality is template-based, not LLM-generated
//! - Learning hooks in discovery field
//!
//! v0.0.410: Evidence pipeline integration - specialists now receive structured evidence

mod api;
mod llm;
mod parsing;
mod types;
mod utils;

// Re-export public API
pub use api::{
    call_json_specialist,
    call_json_specialist_with_evidence,
    get_answer_text,
    get_display_output,
};
pub use types::JsonSpecialistResult;

// Re-export utility functions if needed by other modules
pub(crate) use utils::domain_to_string;

#[cfg(test)]
mod tests {
    use crate::specialist_json::parsing::{extract_json_object, parse_specialist_json};

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
