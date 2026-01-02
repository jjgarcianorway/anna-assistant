//! JSON enforcement types and schema hints (v0.0.433).

use serde::{Deserialize, Serialize};

use super::contract::SpecialistResult;

/// Event emitted during JSON parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JsonParseEvent {
    /// Successfully parsed JSON.
    Success { tokens_parsed: usize },
    /// Failed to parse - syntax error.
    SyntaxError {
        error: String,
        position: Option<usize>,
    },
    /// Parsed but missing required fields.
    MissingFields { fields: Vec<String> },
    /// Parsed but has invalid values.
    InvalidValues { field: String, error: String },
    /// Found prose outside JSON.
    ProseDetected { prose_length: usize },
    /// Retrying with stricter prompt.
    RetryAttempt { attempt: usize },
}

/// Result of JSON parsing.
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Successfully parsed.
    Ok(SpecialistResult),
    /// Parse failed.
    Failed {
        error: String,
        events: Vec<JsonParseEvent>,
        raw_output: String,
    },
}

impl ParseResult {
    /// Check if successful.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Get the result if successful.
    pub fn into_result(self) -> Option<SpecialistResult> {
        match self {
            Self::Ok(r) => Some(r),
            Self::Failed { .. } => None,
        }
    }

    /// Get error info.
    pub fn error_info(&self) -> Option<String> {
        match self {
            Self::Ok(_) => None,
            Self::Failed { error, .. } => Some(error.clone()),
        }
    }
}

/// Hint for schema in prompts.
#[derive(Debug, Clone)]
pub struct SchemaHint {
    /// JSON schema description.
    pub schema: String,
    /// Example response.
    pub example: String,
    /// Strict reminder.
    pub reminder: String,
}

impl Default for SchemaHint {
    fn default() -> Self {
        Self::specialist_result_schema()
    }
}

impl SchemaHint {
    /// Schema hint for SpecialistResult.
    pub fn specialist_result_schema() -> Self {
        Self {
            schema: r#"
Your response MUST be a valid JSON object with this exact structure:
{
  "outcome": "Success" | "Partial" | "ClarificationRequired" | "Unsupported" | "InternalError",
  "human_summary": "2-4 line plain language summary",
  "diagnosis": "optional detailed explanation" | null,
  "steps": [
    {
      "description": "what this step does",
      "command": "command to run",
      "needs_sudo": true | false,
      "category": "Diagnostic" | "Fix" | "Cleanup" | "Info"
    }
  ],
  "evidence_refs": [
    {
      "probe_name": "name of probe",
      "snippet_id": "evidence id",
      "excerpt": "optional excerpt" | null
    }
  ],
  "error_info": null,
  "confidence": 0.0 to 1.0
}
"#
            .to_string(),
            example: r#"
Example valid response:
{
  "outcome": "Success",
  "human_summary": "You have 17.0 GiB free out of 31.0 GiB total RAM (54% available).",
  "diagnosis": "Memory usage is healthy. No immediate action needed.",
  "steps": [],
  "evidence_refs": [{"probe_name": "proc_meminfo", "snippet_id": "mem_001", "excerpt": "MemFree: 17825792 kB"}],
  "error_info": null,
  "confidence": 0.95
}
"#
            .to_string(),
            reminder: "CRITICAL: Any content outside the JSON object will be discarded. Output ONLY the JSON object, nothing else.".to_string(),
        }
    }

    /// Format for inclusion in prompt.
    pub fn format_for_prompt(&self) -> String {
        format!("{}\n{}\n\n{}", self.schema, self.example, self.reminder)
    }

    /// Stricter version for retry.
    pub fn stricter_version(&self) -> Self {
        Self {
            schema: self.schema.clone(),
            example: self.example.clone(),
            reminder: format!(
                "RETRY ATTEMPT: Your previous response was invalid JSON.\n\
                 You MUST output EXACTLY a JSON object matching the schema.\n\
                 NO explanations, NO prose, NO markdown - ONLY the raw JSON object.\n\
                 If you cannot answer, use outcome: \"InternalError\".\n\n{}",
                self.reminder
            ),
        }
    }
}
