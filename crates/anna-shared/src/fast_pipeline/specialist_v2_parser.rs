//! Specialist Output Contract v2 - Parser - v0.0.438.
//!
//! Parsing logic for specialist outputs.

use super::specialist_v2_types::{
    truncate_string, SpecialistOutputV2, ValidationResult, MAX_SPECIALIST_TOKENS,
};

/// Parse result.
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Successfully parsed.
    Success(SpecialistOutputV2),
    /// Parsed but invalid.
    Invalid {
        output: SpecialistOutputV2,
        issues: Vec<String>,
    },
    /// Parse error.
    ParseError { message: String, raw_output: String },
    /// Empty output.
    Empty,
}

impl ParseResult {
    /// Check if successful.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Get output if successful.
    pub fn output(&self) -> Option<&SpecialistOutputV2> {
        match self {
            Self::Success(o) | Self::Invalid { output: o, .. } => Some(o),
            _ => None,
        }
    }
}

/// Parser for specialist output v2.
pub struct SpecialistParser;

impl SpecialistParser {
    /// Parse raw output to SpecialistOutputV2.
    pub fn parse(raw: &str, case_id: &str) -> ParseResult {
        let trimmed = raw.trim();

        // Empty output
        if trimmed.is_empty() {
            return ParseResult::Empty;
        }

        // Try to parse as JSON
        match serde_json::from_str::<SpecialistOutputV2>(trimmed) {
            Ok(output) => {
                // Validate
                match output.validate() {
                    ValidationResult::Valid => ParseResult::Success(output),
                    ValidationResult::Invalid { issues } => ParseResult::Invalid { output, issues },
                }
            }
            Err(e) => {
                // Try to extract JSON from mixed content
                if let Some(json_str) = Self::extract_json(trimmed) {
                    match serde_json::from_str::<SpecialistOutputV2>(&json_str) {
                        Ok(output) => match output.validate() {
                            ValidationResult::Valid => ParseResult::Success(output),
                            ValidationResult::Invalid { issues } => {
                                ParseResult::Invalid { output, issues }
                            }
                        },
                        Err(_) => ParseResult::ParseError {
                            message: e.to_string(),
                            raw_output: truncate_string(trimmed, 500),
                        },
                    }
                } else {
                    ParseResult::ParseError {
                        message: e.to_string(),
                        raw_output: truncate_string(trimmed, 500),
                    }
                }
            }
        }
    }

    /// Extract JSON object from mixed content.
    fn extract_json(text: &str) -> Option<String> {
        let first_brace = text.find('{')?;
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, c) in text[first_brace..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(text[first_brace..first_brace + i + 1].to_string());
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Truncate output if too long and return cannot_answer.
    pub fn truncate_if_needed(raw: &str, case_id: &str) -> ParseResult {
        // Estimate tokens
        let estimated_tokens = raw.len() / 4;

        if estimated_tokens > MAX_SPECIALIST_TOKENS {
            return ParseResult::Success(SpecialistOutputV2::output_limit_exceeded(case_id));
        }

        Self::parse(raw, case_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_success() {
        let json = r#"{
            "case_id": "DSK-001",
            "verdict": "answered",
            "confidence": 0.9,
            "answer": {
                "fields": {"free_ram": "4.2 GB"},
                "summary": "You have 4.2 GB free RAM"
            },
            "required_next_probes": [],
            "notes": ""
        }"#;

        let result = SpecialistParser::parse(json, "DSK-001");
        assert!(result.is_success());
    }

    #[test]
    fn test_parser_with_preamble() {
        let raw = r#"Let me analyze this...
        {"case_id": "DSK-001", "verdict": "answered", "confidence": 0.8, "answer": {"fields": {}, "summary": "Test"}, "required_next_probes": [], "notes": ""}"#;

        let result = SpecialistParser::parse(raw, "DSK-001");
        assert!(result.is_success());
    }

    #[test]
    fn test_parser_empty() {
        let result = SpecialistParser::parse("", "DSK-001");
        assert!(matches!(result, ParseResult::Empty));
    }
}
