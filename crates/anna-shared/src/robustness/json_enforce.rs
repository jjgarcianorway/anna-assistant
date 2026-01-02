//! JSON enforcement for LLM responses (v0.0.433).
//!
//! Ensures LLM output is strictly JSON matching our schema.

// Re-export public types and functions from sibling modules
pub use super::json_enforce_parser::JsonEnforcer;
pub use super::json_enforce_types::{JsonParseEvent, ParseResult, SchemaHint};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_hint() {
        let hint = SchemaHint::default();
        let prompt = hint.format_for_prompt();
        assert!(prompt.contains("outcome"));
        assert!(prompt.contains("human_summary"));
    }
}
