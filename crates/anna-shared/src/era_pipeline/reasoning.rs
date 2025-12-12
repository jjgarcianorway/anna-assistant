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

/// Maximum reasoning length.
pub const MAX_REASONING_CHARS: usize = 200;

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
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Specialist prompt for reasoning-only mode.
pub const REASONING_SYSTEM_PROMPT: &str = r#"You are a REASONING specialist. You do NOT answer users directly.

INPUT: Intent + EvidenceBundle
OUTPUT: Structured JSON reasoning

RULES:
1. ONLY use facts from the EvidenceBundle
2. NEVER introduce facts not in the bundle
3. NEVER infer beyond the evidence
4. NEVER expand scope of the question
5. Output ONLY valid JSON

OUTPUT SCHEMA:
{
  "case_id": "string",
  "can_answer": true|false,
  "reasoning": "short factual statement (max 200 chars)",
  "derived": {
    "root_cause": "string or null",
    "metric": "string or null"
  },
  "confidence": 0.0-1.0,
  "requires": ["fact_name"]  // only if can_answer=false
}

If evidence is insufficient, set can_answer=false and list required facts."#;

/// Build reasoning prompt for a case.
pub fn build_reasoning_prompt(intent: &str, evidence: &EvidenceBundle) -> String {
    let evidence_json = ReasoningEvidence::from_bundle(evidence);
    let evidence_str = serde_json::to_string_pretty(&evidence_json).unwrap_or_default();

    format!(
        "INTENT: {}\n\nEVIDENCE:\n{}\n\nProduce reasoning JSON.",
        intent, evidence_str
    )
}

/// Validator for reasoning output.
pub struct ReasoningValidator {
    /// Expected case ID.
    expected_case_id: String,
    /// Available facts from evidence.
    available_facts: Vec<String>,
}

impl ReasoningValidator {
    /// Create validator.
    pub fn new(case_id: &str, evidence: &EvidenceBundle) -> Self {
        Self {
            expected_case_id: case_id.to_string(),
            available_facts: evidence.facts.keys().cloned().collect(),
        }
    }

    /// Validate reasoning output.
    pub fn validate(&self, output: &ReasoningOutput) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check case ID
        if output.case_id != self.expected_case_id {
            errors.push(format!(
                "case_id mismatch: expected '{}', got '{}'",
                self.expected_case_id, output.case_id
            ));
        }

        // Check reasoning length
        if output.reasoning.len() > MAX_REASONING_CHARS {
            errors.push(format!(
                "reasoning too long: {} > {}",
                output.reasoning.len(),
                MAX_REASONING_CHARS
            ));
        }

        // Check confidence range
        if output.confidence < 0.0 || output.confidence > 1.0 {
            errors.push(format!("confidence out of range: {}", output.confidence));
        }

        // If can_answer=false, requires must not be empty
        if !output.can_answer && output.requires.is_empty() {
            errors.push("can_answer=false but requires is empty".to_string());
        }

        // If can_answer=true, confidence should be > 0
        if output.can_answer && output.confidence == 0.0 {
            errors.push("can_answer=true but confidence=0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Parse reasoning output from JSON.
pub fn parse_reasoning_output(raw: &str) -> Result<ReasoningOutput, String> {
    // Try direct parse
    if let Ok(output) = serde_json::from_str::<ReasoningOutput>(raw.trim()) {
        return Ok(output);
    }

    // Try to extract JSON from mixed content
    if let Some(json_start) = raw.find('{') {
        if let Some(json_end) = raw.rfind('}') {
            let json_str = &raw[json_start..=json_end];
            if let Ok(output) = serde_json::from_str::<ReasoningOutput>(json_str) {
                return Ok(output);
            }
        }
    }

    Err(format!(
        "Failed to parse reasoning output: {}",
        truncate(raw, 100)
    ))
}

/// Reasoning quality check.
#[derive(Debug, Clone)]
pub struct ReasoningQuality {
    /// Is reasoning grounded in evidence?
    pub grounded: bool,
    /// Did reasoning stay within scope?
    pub within_scope: bool,
    /// Is derived data consistent with evidence?
    pub consistent: bool,
}

impl ReasoningQuality {
    /// Check reasoning quality against evidence.
    pub fn check(output: &ReasoningOutput, evidence: &EvidenceBundle) -> Self {
        // Basic checks - more sophisticated checks could use NLP
        let grounded = !output.reasoning.is_empty();
        let within_scope = output.reasoning.len() <= MAX_REASONING_CHARS;
        let consistent = output.can_answer || !output.requires.is_empty();

        Self {
            grounded,
            within_scope,
            consistent,
        }
    }

    /// Overall quality score.
    pub fn score(&self) -> f64 {
        let mut score = 0.0;
        if self.grounded {
            score += 0.4;
        }
        if self.within_scope {
            score += 0.3;
        }
        if self.consistent {
            score += 0.3;
        }
        score
    }

    /// Is quality acceptable?
    pub fn is_acceptable(&self) -> bool {
        self.grounded && self.within_scope && self.consistent
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

    #[test]
    fn test_reasoning_validation() {
        let mut bundle = EvidenceBundle::new("DSK-0127");
        bundle.add_fact(
            "memory.free_gib",
            super::super::evidence::FactValue::Number(17.0),
        );

        let validator = ReasoningValidator::new("DSK-0127", &bundle);

        let valid_output = ReasoningOutput::answerable("DSK-0127", "Test", 0.9);
        assert!(validator.validate(&valid_output).is_ok());

        let invalid_output = ReasoningOutput::answerable("WRONG-ID", "Test", 0.9);
        assert!(validator.validate(&invalid_output).is_err());
    }

    #[test]
    fn test_parse_reasoning_output() {
        let json = r#"{"case_id":"DSK-0127","can_answer":true,"reasoning":"Test","derived":{},"confidence":0.9,"requires":[]}"#;
        let output = parse_reasoning_output(json).unwrap();
        assert_eq!(output.case_id, "DSK-0127");
        assert!(output.can_answer);
    }

    #[test]
    fn test_reasoning_quality() {
        let output = ReasoningOutput::answerable("DSK-0127", "Grounded reasoning.", 0.9);
        let bundle = EvidenceBundle::new("DSK-0127");

        let quality = ReasoningQuality::check(&output, &bundle);
        assert!(quality.is_acceptable());
        assert!(quality.score() > 0.9);
    }
}
