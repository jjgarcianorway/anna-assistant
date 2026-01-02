//! Specialist Contract v2 - Reasoning Only (Part C) - v0.0.441.
//!
//! Specialists NO LONGER answer the user.
//! They ONLY reason over evidence:
//!
//! - can_answer: true/false
//! - reasoning: short, factual explanation
//! - derived: root_cause, metric (extracted from evidence)
//! - confidence: 0.0-1.0
//! - requires: additional facts needed if can_answer=false
//!
//! NO prose. NO commands. NO explanations to user.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::evidence::EvidenceBundle;

/// Maximum reasoning length.
pub const MAX_REASONING_CHARS: usize = 200;

/// Reasoning request sent to specialist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningRequest {
    /// Case ID.
    pub case_id: String,
    /// Intent (what is being asked).
    pub intent: String,
    /// Evidence bundle (ONLY source of truth).
    pub evidence: ReasoningEvidence,
}

/// Simplified evidence for reasoning request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningEvidence {
    /// Available facts.
    pub facts: HashMap<String, serde_json::Value>,
    /// Domains with confidence.
    pub confidence: HashMap<String, f64>,
}

impl ReasoningEvidence {
    /// Build from full evidence bundle.
    pub fn from_bundle(bundle: &EvidenceBundle) -> Self {
        let facts: HashMap<String, serde_json::Value> = bundle
            .facts
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::to_value(v).unwrap_or(serde_json::Value::Null),
                )
            })
            .collect();

        Self {
            facts,
            confidence: bundle.confidence.clone(),
        }
    }
}

/// Reasoning output from specialist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOutput {
    /// Case ID.
    pub case_id: String,
    /// Whether the question can be answered from evidence.
    pub can_answer: bool,
    /// Short, factual reasoning (max 200 chars).
    pub reasoning: String,
    /// Derived values from evidence.
    #[serde(default)]
    pub derived: DerivedValues,
    /// Confidence in reasoning (0.0-1.0).
    pub confidence: f64,
    /// Required facts if can_answer=false.
    #[serde(default)]
    pub requires: Vec<String>,
}

/// Derived values extracted from evidence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DerivedValues {
    /// Root cause (if identified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<String>,
    /// Primary metric/value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<String>,
    /// Secondary values.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
}

impl DerivedValues {
    /// Create empty derived values.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create with metric only.
    pub fn metric(value: &str) -> Self {
        Self {
            metric: Some(value.to_string()),
            ..Default::default()
        }
    }

    /// Create with root cause.
    pub fn root_cause(cause: &str) -> Self {
        Self {
            root_cause: Some(cause.to_string()),
            ..Default::default()
        }
    }

    /// Add other value.
    pub fn with_other(mut self, key: &str, value: &str) -> Self {
        self.other.insert(key.to_string(), value.to_string());
        self
    }
}

impl ReasoningOutput {
    /// Create a "can answer" response.
    pub fn answerable(case_id: &str, reasoning: &str, confidence: f64) -> Self {
        Self {
            case_id: case_id.to_string(),
            can_answer: true,
            reasoning: truncate(reasoning, MAX_REASONING_CHARS),
            derived: DerivedValues::empty(),
            confidence: confidence.clamp(0.0, 1.0),
            requires: Vec::new(),
        }
    }

    /// Create a "cannot answer" response.
    pub fn unanswerable(case_id: &str, reasoning: &str, requires: Vec<&str>) -> Self {
        Self {
            case_id: case_id.to_string(),
            can_answer: false,
            reasoning: truncate(reasoning, MAX_REASONING_CHARS),
            derived: DerivedValues::empty(),
            confidence: 0.0,
            requires: requires.into_iter().map(String::from).collect(),
        }
    }

    /// Add derived metric.
    pub fn with_metric(mut self, value: &str) -> Self {
        self.derived.metric = Some(value.to_string());
        self
    }

    /// Add root cause.
    pub fn with_root_cause(mut self, cause: &str) -> Self {
        self.derived.root_cause = Some(cause.to_string());
        self
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

/// Truncate string to max length.
pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_output_answerable() {
        let output =
            ReasoningOutput::answerable("DSK-0127", "Memory free is 17 GiB from evidence.", 0.95)
                .with_metric("17.0 GiB");

        assert!(output.can_answer);
        assert_eq!(output.derived.metric, Some("17.0 GiB".to_string()));
        assert!(output.requires.is_empty());
    }

    #[test]
    fn test_reasoning_output_unanswerable() {
        let output = ReasoningOutput::unanswerable(
            "DSK-0127",
            "Boot blame data not in evidence.",
            vec!["boot.blame"],
        );

        assert!(!output.can_answer);
        assert!(!output.requires.is_empty());
        assert_eq!(output.confidence, 0.0);
    }
}
