//! Intent Schema Parser - v0.0.439.
//!
//! Parses translator JSON output to TicketIntentSchema.

use super::schema::TicketIntentSchema;

/// Parser for translator JSON output.
pub struct IntentSchemaParser;

impl IntentSchemaParser {
    /// Parse raw JSON to TicketIntentSchema.
    pub fn parse(raw: &str) -> Result<TicketIntentSchema, ParseError> {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(ParseError::Empty);
        }

        // Try direct parse
        match serde_json::from_str::<TicketIntentSchema>(trimmed) {
            Ok(schema) => {
                if let Err(issues) = schema.validate() {
                    Err(ParseError::ValidationFailed { issues })
                } else {
                    Ok(schema)
                }
            }
            Err(e) => {
                // Try to extract JSON from mixed content
                if let Some(json_str) = Self::extract_json(trimmed) {
                    match serde_json::from_str::<TicketIntentSchema>(&json_str) {
                        Ok(schema) => {
                            if let Err(issues) = schema.validate() {
                                Err(ParseError::ValidationFailed { issues })
                            } else {
                                Ok(schema)
                            }
                        }
                        Err(_) => Err(ParseError::InvalidJson {
                            message: e.to_string(),
                        }),
                    }
                } else {
                    Err(ParseError::InvalidJson {
                        message: e.to_string(),
                    })
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
}

/// Parse error types.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Empty input.
    Empty,
    /// Invalid JSON.
    InvalidJson { message: String },
    /// Validation failed.
    ValidationFailed { issues: Vec<String> },
}

impl ParseError {
    /// Get error message.
    pub fn message(&self) -> String {
        match self {
            Self::Empty => "Empty translator output".to_string(),
            Self::InvalidJson { message } => format!("Invalid JSON: {}", message),
            Self::ValidationFailed { issues } => {
                format!("Validation failed: {}", issues.join(", "))
            }
        }
    }
}
