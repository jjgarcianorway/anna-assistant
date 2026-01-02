//! Core data types for specialist responses.

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
    SchemaError {
        response: UnifiedSpecialistResponse,
        errors: Vec<String>,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_is_meaningful() {
        let good = UnifiedSpecialistResponse::success("summary", "diagnosis", 0.8);
        assert!(good.is_meaningful());

        let low_conf = UnifiedSpecialistResponse::success("summary", "diagnosis", 0.4);
        assert!(!low_conf.is_meaningful());

        let cannot = UnifiedSpecialistResponse::cannot_answer("no data");
        assert!(!cannot.is_meaningful());
    }
}
