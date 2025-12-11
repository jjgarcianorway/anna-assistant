//! Unified specialist response schema (v0.0.409).
//!
//! This is the SINGLE source of truth for specialist JSON responses.
//! All specialists must produce exactly this structure.
//!
//! Key principles:
//! - can_answer is REQUIRED (boolean)
//! - All fields have sensible defaults via serde
//! - Parse failures are classified explicitly
//! - Validation is strict and separate from parsing

use serde::{Deserialize, Serialize};

/// The unified response that ALL specialists must produce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSpecialistResponse {
    /// Can the specialist answer this question with confidence?
    pub can_answer: bool,

    /// Confidence level (0.0-1.0). Required if can_answer=true.
    #[serde(default)]
    pub confidence: f32,

    /// Short technical summary of the situation
    #[serde(default)]
    pub problem_summary: String,

    /// Explanation grounded in evidence
    #[serde(default)]
    pub diagnosis: String,

    /// Recommended actions (possibly empty)
    #[serde(default)]
    pub recommended_actions: Vec<RecommendedAction>,

    /// Optional notes/caveats
    #[serde(default)]
    pub notes: Vec<String>,

    /// Evidence references used (probe IDs, doc IDs)
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

/// A recommended action with commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    /// Unique ID for this action
    pub id: String,

    /// Short title
    pub title: String,

    /// Description of what this does
    pub description: String,

    /// Shell commands to run
    #[serde(default)]
    pub commands: Vec<ActionCommand>,

    /// Evidence backing this action
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

/// A shell command with explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCommand {
    /// The shell command
    pub shell: String,

    /// What this command does
    pub explain: String,
}

/// Parse result with explicit error classification
#[derive(Debug, Clone)]
pub enum ParseOutcome {
    /// Successfully parsed and validated
    Success(UnifiedSpecialistResponse),
    /// JSON extraction failed (no valid JSON found)
    NoJson { raw: String },
    /// JSON parsed but invalid structure
    InvalidJson { raw: String, error: String },
    /// Schema validation failed (missing required fields, bad values)
    SchemaError { response: UnifiedSpecialistResponse, errors: Vec<String> },
    /// LLM timed out
    Timeout { elapsed_secs: u64 },
}

impl ParseOutcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn error_kind(&self) -> &'static str {
        match self {
            Self::Success(_) => "none",
            Self::NoJson { .. } => "no_json",
            Self::InvalidJson { .. } => "invalid_json",
            Self::SchemaError { .. } => "schema_error",
            Self::Timeout { .. } => "timeout",
        }
    }
}

impl Default for UnifiedSpecialistResponse {
    fn default() -> Self {
        Self {
            can_answer: false,
            confidence: 0.0,
            problem_summary: String::new(),
            diagnosis: String::new(),
            recommended_actions: vec![],
            notes: vec![],
            evidence_refs: vec![],
        }
    }
}

impl UnifiedSpecialistResponse {
    /// Create a "cannot answer" response
    pub fn cannot_answer(reason: &str) -> Self {
        Self {
            can_answer: false,
            confidence: 0.0,
            problem_summary: String::new(),
            diagnosis: reason.to_string(),
            recommended_actions: vec![],
            notes: vec![],
            evidence_refs: vec![],
        }
    }

    /// Create a successful response
    pub fn success(summary: &str, diagnosis: &str, confidence: f32) -> Self {
        Self {
            can_answer: true,
            confidence: confidence.clamp(0.0, 1.0),
            problem_summary: summary.to_string(),
            diagnosis: diagnosis.to_string(),
            recommended_actions: vec![],
            notes: vec![],
            evidence_refs: vec![],
        }
    }

    /// Add an action
    pub fn with_action(mut self, action: RecommendedAction) -> Self {
        self.recommended_actions.push(action);
        self
    }

    /// Add evidence refs
    pub fn with_evidence(mut self, refs: Vec<String>) -> Self {
        self.evidence_refs = refs;
        self
    }

    /// Validate the response schema
    pub fn validate(&self) -> Vec<String> {
        let mut errors = vec![];

        if self.can_answer {
            // When can_answer=true, confidence must be valid
            if self.confidence < 0.0 || self.confidence > 1.0 {
                errors.push(format!(
                    "confidence {} out of range [0.0, 1.0]",
                    self.confidence
                ));
            }
            if self.confidence < 0.5 && self.problem_summary.is_empty() {
                errors.push("low confidence but no problem_summary".to_string());
            }
        }

        // Check for "unknown" hallucinations
        let forbidden = ["unknown is installed", "2 is installed", "**unknown**"];
        for f in forbidden {
            if self.diagnosis.to_lowercase().contains(f)
                || self.problem_summary.to_lowercase().contains(f)
            {
                errors.push(format!("contains forbidden pattern: '{}'", f));
            }
        }

        // Check actions have required fields
        for (i, action) in self.recommended_actions.iter().enumerate() {
            if action.title.is_empty() {
                errors.push(format!("action[{}].title is empty", i));
            }
            if action.description.is_empty() {
                errors.push(format!("action[{}].description is empty", i));
            }
        }

        errors
    }

    /// Check if this is a meaningful answer (not just empty or low confidence)
    pub fn is_meaningful(&self) -> bool {
        self.can_answer
            && self.confidence >= 0.6
            && (!self.diagnosis.is_empty() || !self.problem_summary.is_empty())
    }
}

/// Extract JSON from raw LLM output
pub fn extract_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // Try clean JSON object first
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed.to_string());
    }

    // Try markdown code block with json
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start + 7..].find("```") {
            let json = trimmed[start + 7..start + 7 + end].trim();
            if json.starts_with('{') {
                return Some(json.to_string());
            }
        }
    }

    // Try bare code block
    if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed[start + 3..].find("```") {
            let json = trimmed[start + 3..start + 3 + end].trim();
            if json.starts_with('{') {
                return Some(json.to_string());
            }
        }
    }

    // Find first { and last }
    let first_brace = trimmed.find('{')?;
    let last_brace = trimmed.rfind('}')?;
    if last_brace > first_brace {
        return Some(trimmed[first_brace..=last_brace].to_string());
    }

    None
}

/// Parse raw LLM output into a validated response
pub fn parse_specialist_output(raw: &str) -> ParseOutcome {
    // Step 1: Extract JSON
    let json_str = match extract_json(raw) {
        Some(j) => j,
        None => {
            return ParseOutcome::NoJson {
                raw: truncate(raw, 500),
            }
        }
    };

    // Step 2: Parse JSON
    let response: UnifiedSpecialistResponse = match serde_json::from_str(&json_str) {
        Ok(r) => r,
        Err(e) => {
            return ParseOutcome::InvalidJson {
                raw: truncate(&json_str, 500),
                error: e.to_string(),
            }
        }
    };

    // Step 3: Validate schema
    let errors = response.validate();
    if !errors.is_empty() {
        return ParseOutcome::SchemaError { response, errors };
    }

    ParseOutcome::Success(response)
}

/// Create a timeout parse outcome
pub fn timeout_outcome(elapsed_secs: u64) -> ParseOutcome {
    ParseOutcome::Timeout { elapsed_secs }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Format a parse failure for user display
pub fn format_parse_failure(outcome: &ParseOutcome, suggestions: &[String]) -> String {
    let mut output = match outcome {
        ParseOutcome::NoJson { .. } => {
            "I had an internal error: the specialist did not return valid data.".to_string()
        }
        ParseOutcome::InvalidJson { error, .. } => {
            format!("I had an internal error parsing the response: {}", truncate(error, 100))
        }
        ParseOutcome::SchemaError { errors, .. } => {
            format!("The specialist response was invalid: {}", errors.join(", "))
        }
        ParseOutcome::Timeout { elapsed_secs } => {
            format!("The specialist timed out after {}s.", elapsed_secs)
        }
        ParseOutcome::Success(_) => return String::new(),
    };

    if !suggestions.is_empty() {
        output.push_str("\n\nYou can try these manual steps:");
        for s in suggestions.iter().take(5) {
            output.push_str(&format!("\n  - {}", s));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_clean() {
        let raw = r#"{"can_answer": true, "confidence": 0.9}"#;
        let json = extract_json(raw).unwrap();
        assert!(json.starts_with('{'));
    }

    #[test]
    fn test_extract_json_markdown() {
        let raw = r#"Here is the response:
```json
{"can_answer": true, "confidence": 0.9}
```"#;
        let json = extract_json(raw).unwrap();
        assert!(json.contains("can_answer"));
    }

    #[test]
    fn test_extract_json_with_prose() {
        let raw = r#"I analyzed the data. {"can_answer": true, "confidence": 0.9} That's my answer."#;
        let json = extract_json(raw).unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn test_validate_success() {
        let response = UnifiedSpecialistResponse::success("test", "diagnosis", 0.9);
        let errors = response.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_forbidden_pattern() {
        let mut response = UnifiedSpecialistResponse::success("test", "unknown is installed", 0.9);
        let errors = response.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("forbidden")));
    }

    #[test]
    fn test_validate_invalid_confidence() {
        let response = UnifiedSpecialistResponse {
            can_answer: true,
            confidence: 1.5,
            ..Default::default()
        };
        let errors = response.validate();
        assert!(errors.iter().any(|e| e.contains("out of range")));
    }

    #[test]
    fn test_parse_valid_response() {
        let raw = r#"{"can_answer": true, "confidence": 0.85, "diagnosis": "All good"}"#;
        let outcome = parse_specialist_output(raw);
        assert!(outcome.is_success());
    }

    #[test]
    fn test_parse_no_json() {
        let raw = "This is just prose without any JSON";
        let outcome = parse_specialist_output(raw);
        assert_eq!(outcome.error_kind(), "no_json");
    }

    #[test]
    fn test_is_meaningful() {
        let good = UnifiedSpecialistResponse::success("summary", "diagnosis", 0.8);
        assert!(good.is_meaningful());

        let low_conf = UnifiedSpecialistResponse::success("summary", "diagnosis", 0.4);
        assert!(!low_conf.is_meaningful());

        let cannot = UnifiedSpecialistResponse::cannot_answer("no data");
        assert!(!cannot.is_meaningful());
    }
}
