//! Specialist Output Contract v2 (Part B) - Types and Core Structures - v0.0.438.
//!
//! Core types for specialist output contract.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum tokens for specialist completion.
pub const MAX_SPECIALIST_TOKENS: usize = 220;

/// Maximum characters for answer summary.
pub const MAX_SUMMARY_CHARS: usize = 160;

/// Maximum characters for notes.
pub const MAX_NOTES_CHARS: usize = 200;

/// Maximum fields in answer payload.
pub const MAX_ANSWER_FIELDS: usize = 10;

/// Specialist verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Question answered successfully.
    Answered,
    /// Need more data (probes).
    NeedMoreData,
    /// Escalate to senior specialist.
    Escalate,
    /// Cannot answer.
    CannotAnswer,
}

impl Verdict {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Answered => "answered",
            Self::NeedMoreData => "need_more_data",
            Self::Escalate => "escalate",
            Self::CannotAnswer => "cannot_answer",
        }
    }

    /// Whether this is a success verdict.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Answered)
    }

    /// Whether this requires more work.
    pub fn needs_more_work(&self) -> bool {
        matches!(self, Self::NeedMoreData | Self::Escalate)
    }
}

/// Validation result.
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Output is valid.
    Valid,
    /// Output has issues.
    Invalid { issues: Vec<String> },
}

impl ValidationResult {
    /// Check if valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// Answer payload within specialist output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerPayload {
    /// Key-value fields in the answer.
    pub fields: HashMap<String, String>,
    /// Summary (max 160 chars).
    pub summary: String,
}

impl AnswerPayload {
    /// Create a new answer payload.
    pub fn new(summary: &str) -> Self {
        Self {
            fields: HashMap::new(),
            summary: truncate_string(summary, MAX_SUMMARY_CHARS),
        }
    }

    /// Add a field.
    pub fn with_field(mut self, key: &str, value: &str) -> Self {
        if self.fields.len() < MAX_ANSWER_FIELDS {
            self.fields.insert(key.to_string(), value.to_string());
        }
        self
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.summary.is_empty()
    }

    /// Validate payload.
    pub fn validate(&self) -> ValidationResult {
        let mut issues = Vec::new();

        if self.summary.len() > MAX_SUMMARY_CHARS {
            issues.push(format!("Summary exceeds {} chars", MAX_SUMMARY_CHARS));
        }

        if self.fields.len() > MAX_ANSWER_FIELDS {
            issues.push(format!("Too many fields (max {})", MAX_ANSWER_FIELDS));
        }

        if issues.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid { issues }
        }
    }
}

impl Default for AnswerPayload {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
            summary: String::new(),
        }
    }
}

/// The specialist output contract v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistOutputV2 {
    /// Case/ticket ID.
    pub case_id: String,
    /// Verdict.
    pub verdict: Verdict,
    /// Confidence (0.0-1.0).
    pub confidence: f64,
    /// Answer payload.
    pub answer: AnswerPayload,
    /// Required next probes.
    #[serde(default)]
    pub required_next_probes: Vec<String>,
    /// Notes (max 200 chars).
    #[serde(default)]
    pub notes: String,
}

impl SpecialistOutputV2 {
    /// Create a successful answer.
    pub fn answered(case_id: &str, summary: &str, confidence: f64) -> Self {
        Self {
            case_id: case_id.to_string(),
            verdict: Verdict::Answered,
            confidence: confidence.clamp(0.0, 1.0),
            answer: AnswerPayload::new(summary),
            required_next_probes: Vec::new(),
            notes: String::new(),
        }
    }

    /// Create a "need more data" response.
    pub fn need_more_data(case_id: &str, probes: Vec<&str>) -> Self {
        Self {
            case_id: case_id.to_string(),
            verdict: Verdict::NeedMoreData,
            confidence: 0.0,
            answer: AnswerPayload::default(),
            required_next_probes: probes.into_iter().map(String::from).collect(),
            notes: String::new(),
        }
    }

    /// Create an "escalate" response.
    pub fn escalate(case_id: &str, reason: &str) -> Self {
        Self {
            case_id: case_id.to_string(),
            verdict: Verdict::Escalate,
            confidence: 0.0,
            answer: AnswerPayload::default(),
            required_next_probes: Vec::new(),
            notes: truncate_string(reason, MAX_NOTES_CHARS),
        }
    }

    /// Create a "cannot answer" response.
    pub fn cannot_answer(case_id: &str, reason: &str) -> Self {
        Self {
            case_id: case_id.to_string(),
            verdict: Verdict::CannotAnswer,
            confidence: 0.0,
            answer: AnswerPayload::default(),
            required_next_probes: Vec::new(),
            notes: truncate_string(reason, MAX_NOTES_CHARS),
        }
    }

    /// Create output limit exceeded response.
    pub fn output_limit_exceeded(case_id: &str) -> Self {
        Self::cannot_answer(case_id, "output_limit_exceeded")
    }

    /// Add a field to the answer.
    pub fn with_field(mut self, key: &str, value: &str) -> Self {
        self.answer = self.answer.with_field(key, value);
        self
    }

    /// Set notes.
    pub fn with_notes(mut self, notes: &str) -> Self {
        self.notes = truncate_string(notes, MAX_NOTES_CHARS);
        self
    }

    /// Validate the output.
    pub fn validate(&self) -> ValidationResult {
        let mut issues = Vec::new();

        // Check answer validation
        if let ValidationResult::Invalid {
            issues: answer_issues,
        } = self.answer.validate()
        {
            issues.extend(answer_issues);
        }

        // Check notes length
        if self.notes.len() > MAX_NOTES_CHARS {
            issues.push(format!("Notes exceeds {} chars", MAX_NOTES_CHARS));
        }

        // Check confidence range
        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push("Confidence out of range [0,1]".to_string());
        }

        // Validate verdict-specific constraints
        match self.verdict {
            Verdict::Answered => {
                if self.answer.is_empty() {
                    issues.push("Answered verdict but answer is empty".to_string());
                }
            }
            Verdict::NeedMoreData => {
                if self.required_next_probes.is_empty() {
                    issues.push("NeedMoreData but no probes specified".to_string());
                }
            }
            _ => {}
        }

        if issues.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid { issues }
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Estimate token count (rough approximation).
    pub fn estimate_tokens(&self) -> usize {
        // Rough estimate: 1 token ~= 4 chars
        let json = self.to_json().unwrap_or_default();
        json.len() / 4
    }
}

/// Truncate a string to max length with ellipsis.
pub(crate) fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_output_answered() {
        let output = SpecialistOutputV2::answered("DSK-001", "4.2 GB free RAM", 0.95)
            .with_field("free_ram", "4.2 GB");

        assert_eq!(output.verdict, Verdict::Answered);
        assert!(output.validate().is_valid());
    }

    #[test]
    fn test_specialist_output_need_more_data() {
        let output = SpecialistOutputV2::need_more_data("DSK-002", vec!["sys.mem.free"]);

        assert_eq!(output.verdict, Verdict::NeedMoreData);
        assert!(!output.required_next_probes.is_empty());
        assert!(output.validate().is_valid());
    }

    #[test]
    fn test_specialist_output_cannot_answer() {
        let output = SpecialistOutputV2::cannot_answer("DSK-003", "Insufficient data");

        assert_eq!(output.verdict, Verdict::CannotAnswer);
        assert!(output.validate().is_valid());
    }

    #[test]
    fn test_truncate_string() {
        let long = "a".repeat(200);
        let truncated = truncate_string(&long, 50);
        assert!(truncated.len() <= 50);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_verdict_methods() {
        assert!(Verdict::Answered.is_success());
        assert!(!Verdict::CannotAnswer.is_success());
        assert!(Verdict::NeedMoreData.needs_more_work());
    }

    #[test]
    fn test_validation_answered_empty() {
        let mut output = SpecialistOutputV2::answered("DSK-001", "", 0.9);
        output.answer.summary = String::new();

        let result = output.validate();
        assert!(!result.is_valid());
    }
}
