//! JSON enforcement for LLM responses (v0.0.433).
//!
//! Ensures LLM output is strictly JSON matching our schema.

use super::contract::{
    EvidenceRef, ProposedStep, SpecialistResult, StepCategory, TicketMetrics, TicketOutcome,
};
use serde::{Deserialize, Serialize};

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

/// JSON enforcer for LLM responses.
pub struct JsonEnforcer {
    events: Vec<JsonParseEvent>,
}

impl JsonEnforcer {
    /// Create a new enforcer.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Parse LLM output into SpecialistResult.
    pub fn parse(&mut self, raw: &str) -> ParseResult {
        self.events.clear();

        // Try to extract JSON from the response
        let json_str = self.extract_json(raw);

        // Check for prose outside JSON
        if json_str.len() < raw.trim().len() {
            let prose_len = raw.len() - json_str.len();
            if prose_len > 10 {
                self.events.push(JsonParseEvent::ProseDetected {
                    prose_length: prose_len,
                });
            }
        }

        // Try to parse as our schema
        match self.parse_specialist_result(&json_str) {
            Ok(result) => {
                self.events.push(JsonParseEvent::Success {
                    tokens_parsed: json_str.len(),
                });
                ParseResult::Ok(result)
            }
            Err(e) => ParseResult::Failed {
                error: e.clone(),
                events: self.events.clone(),
                raw_output: raw.to_string(),
            },
        }
    }

    /// Extract JSON object from text.
    fn extract_json(&self, raw: &str) -> String {
        let trimmed = raw.trim();

        // If it starts with {, try to find matching }
        if trimmed.starts_with('{') {
            if let Some(end) = find_matching_brace(trimmed) {
                return trimmed[..=end].to_string();
            }
        }

        // Try to find JSON anywhere in the text
        if let Some(start) = trimmed.find('{') {
            let rest = &trimmed[start..];
            if let Some(end) = find_matching_brace(rest) {
                return rest[..=end].to_string();
            }
        }

        // Try markdown code block
        if let Some(start) = trimmed.find("```json") {
            let after_marker = &trimmed[start + 7..];
            if let Some(end) = after_marker.find("```") {
                let json = after_marker[..end].trim();
                if json.starts_with('{') {
                    return json.to_string();
                }
            }
        }

        trimmed.to_string()
    }

    /// Parse into SpecialistResult.
    fn parse_specialist_result(&mut self, json: &str) -> Result<SpecialistResult, String> {
        // First try direct parse
        if let Ok(result) = serde_json::from_str::<SpecialistResult>(json) {
            return Ok(result);
        }

        // Try lenient parse into intermediate format
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
            self.events.push(JsonParseEvent::SyntaxError {
                error: e.to_string(),
                position: None,
            });
            format!("JSON syntax error: {}", e)
        })?;

        // Extract fields manually
        let outcome = self.extract_outcome(&value)?;
        let human_summary = self.extract_string(&value, "human_summary", true)?;
        let diagnosis = self.extract_string(&value, "diagnosis", false).ok();
        let steps = self.extract_steps(&value)?;
        let evidence_refs = self.extract_evidence(&value)?;
        let error_info = self.extract_string(&value, "error_info", false).ok();
        let confidence = self.extract_confidence(&value);

        Ok(SpecialistResult {
            outcome,
            human_summary,
            diagnosis,
            steps,
            evidence_refs,
            error_info,
            metrics: TicketMetrics::default(),
            handler: None,
            department: None,
            confidence,
        })
    }

    /// Extract outcome field.
    fn extract_outcome(&mut self, value: &serde_json::Value) -> Result<TicketOutcome, String> {
        let outcome_str = value
            .get("outcome")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                self.events.push(JsonParseEvent::MissingFields {
                    fields: vec!["outcome".to_string()],
                });
                "Missing required field: outcome".to_string()
            })?;

        match outcome_str.to_lowercase().as_str() {
            "success" => Ok(TicketOutcome::Success),
            "partial" => Ok(TicketOutcome::Partial),
            "clarificationrequired" | "clarification_required" => {
                Ok(TicketOutcome::ClarificationRequired)
            }
            "unsupported" => Ok(TicketOutcome::Unsupported),
            "internalerror" | "internal_error" => Ok(TicketOutcome::InternalError),
            "timeout" => Ok(TicketOutcome::Timeout),
            "parseerror" | "parse_error" => Ok(TicketOutcome::ParseError),
            _ => {
                self.events.push(JsonParseEvent::InvalidValues {
                    field: "outcome".to_string(),
                    error: format!("Unknown outcome: {}", outcome_str),
                });
                Err(format!("Invalid outcome: {}", outcome_str))
            }
        }
    }

    /// Extract string field.
    fn extract_string(
        &mut self,
        value: &serde_json::Value,
        field: &str,
        required: bool,
    ) -> Result<String, String> {
        match value.get(field) {
            Some(v) if v.is_string() => Ok(v.as_str().unwrap().to_string()),
            Some(v) if v.is_null() && !required => Err("null".to_string()),
            None if !required => Err("missing".to_string()),
            _ => {
                if required {
                    self.events.push(JsonParseEvent::MissingFields {
                        fields: vec![field.to_string()],
                    });
                }
                Err(format!("Missing or invalid field: {}", field))
            }
        }
    }

    /// Extract confidence.
    fn extract_confidence(&self, value: &serde_json::Value) -> f32 {
        value
            .get("confidence")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.8)
            .clamp(0.0, 1.0)
    }

    /// Extract steps array.
    fn extract_steps(&mut self, value: &serde_json::Value) -> Result<Vec<ProposedStep>, String> {
        let arr = match value.get("steps") {
            Some(v) if v.is_array() => v.as_array().unwrap(),
            Some(v) if v.is_null() => return Ok(Vec::new()),
            None => return Ok(Vec::new()),
            _ => return Ok(Vec::new()),
        };

        let mut steps = Vec::new();
        for item in arr {
            if let (Some(desc), Some(cmd)) = (
                item.get("description").and_then(|v| v.as_str()),
                item.get("command").and_then(|v| v.as_str()),
            ) {
                let needs_sudo = item
                    .get("needs_sudo")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let category = match item.get("category").and_then(|v| v.as_str()) {
                    Some("Diagnostic" | "diagnostic") => StepCategory::Diagnostic,
                    Some("Fix" | "fix") => StepCategory::Fix,
                    Some("Cleanup" | "cleanup") => StepCategory::Cleanup,
                    _ => StepCategory::Info,
                };

                steps.push(ProposedStep {
                    description: desc.to_string(),
                    command: cmd.to_string(),
                    needs_sudo,
                    category,
                });
            }
        }
        Ok(steps)
    }

    /// Extract evidence refs.
    fn extract_evidence(&mut self, value: &serde_json::Value) -> Result<Vec<EvidenceRef>, String> {
        let arr = match value.get("evidence_refs") {
            Some(v) if v.is_array() => v.as_array().unwrap(),
            _ => return Ok(Vec::new()),
        };

        let mut refs = Vec::new();
        for item in arr {
            if let Some(probe_name) = item.get("probe_name").and_then(|v| v.as_str()) {
                let snippet_id = item
                    .get("snippet_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let excerpt = item.get("excerpt").and_then(|v| v.as_str());

                refs.push(EvidenceRef {
                    probe_name: probe_name.to_string(),
                    snippet_id: snippet_id.to_string(),
                    excerpt: excerpt.map(String::from),
                });
            }
        }
        Ok(refs)
    }

    /// Get parse events.
    pub fn events(&self) -> &[JsonParseEvent] {
        &self.events
    }
}

impl Default for JsonEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Find matching closing brace.
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in s.char_indices() {
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
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let mut enforcer = JsonEnforcer::new();
        let json = r#"{"outcome": "Success", "human_summary": "All good", "steps": [], "evidence_refs": [], "confidence": 0.9}"#;

        let result = enforcer.parse(json);
        assert!(result.is_ok());
        if let ParseResult::Ok(r) = result {
            assert_eq!(r.outcome, TicketOutcome::Success);
            assert_eq!(r.human_summary, "All good");
        }
    }

    #[test]
    fn test_parse_with_prose() {
        let mut enforcer = JsonEnforcer::new();
        let json = r#"Here is my response:
        {"outcome": "Success", "human_summary": "Test", "steps": [], "evidence_refs": [], "confidence": 0.8}
        Hope this helps!"#;

        let result = enforcer.parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_json() {
        let mut enforcer = JsonEnforcer::new();
        let json = "This is not JSON at all";

        let result = enforcer.parse(json);
        assert!(!result.is_ok());
    }

    #[test]
    fn test_schema_hint() {
        let hint = SchemaHint::default();
        let prompt = hint.format_for_prompt();
        assert!(prompt.contains("outcome"));
        assert!(prompt.contains("human_summary"));
    }
}
