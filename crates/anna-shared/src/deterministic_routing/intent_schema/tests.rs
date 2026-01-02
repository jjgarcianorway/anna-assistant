//! Intent Schema Tests - v0.0.439.
//!
//! Unit tests for intent schema types, parsing, and validation.

#[cfg(test)]
mod tests {
    use crate::deterministic_routing::intent_schema::{
        CanonicalIntent, Department, IntentSchemaParser, TicketIntentSchema,
    };

    #[test]
    fn test_canonical_intent_from_str() {
        assert_eq!(
            CanonicalIntent::from_str_loose("boot_perf"),
            CanonicalIntent::BootPerf
        );
        assert_eq!(
            CanonicalIntent::from_str_loose("GPU_INFO"),
            CanonicalIntent::GpuInfo
        );
        assert_eq!(
            CanonicalIntent::from_str_loose("unknown_thing"),
            CanonicalIntent::Unknown
        );
    }

    #[test]
    fn test_department_from_str() {
        assert_eq!(
            Department::from_str_loose("Performance"),
            Some(Department::Performance)
        );
        assert_eq!(
            Department::from_str_loose("hardware"),
            Some(Department::Hardware)
        );
        assert_eq!(Department::from_str_loose("bogus"), None);
    }

    #[test]
    fn test_schema_creation() {
        let schema = TicketIntentSchema::new(
            "how much RAM?",
            CanonicalIntent::MemStatus,
            Department::Performance,
        )
        .with_required_evidence(vec!["meminfo", "free_h"]);

        assert_eq!(schema.intent, CanonicalIntent::MemStatus);
        assert_eq!(schema.department, Department::Performance);
        assert_eq!(schema.required_evidence.len(), 2);
    }

    #[test]
    fn test_clarification_truncation() {
        let long_question = "a".repeat(200);
        let schema =
            TicketIntentSchema::new("query", CanonicalIntent::Unknown, Department::Performance)
                .needs_clarification(&long_question);

        assert!(schema.clarifying_question.unwrap().len() <= 120);
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{
            "user_query": "how much RAM?",
            "intent": "mem_status",
            "department": "Performance",
            "required_evidence": ["meminfo"],
            "optional_evidence": [],
            "need_clarification": false,
            "clarifying_question": null,
            "risk_level": "read_only"
        }"#;

        let result = IntentSchemaParser::parse(json);
        assert!(result.is_ok());
        let schema = result.unwrap();
        assert_eq!(schema.intent, CanonicalIntent::MemStatus);
    }

    #[test]
    fn test_parse_with_preamble() {
        let raw = r#"Let me analyze...
        {"user_query": "test", "intent": "disk_usage", "department": "Storage", "required_evidence": [], "need_clarification": false, "risk_level": "read_only"}"#;

        let result = IntentSchemaParser::parse(raw);
        assert!(result.is_ok());
    }
}
