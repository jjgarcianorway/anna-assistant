//! Ticket Intent Schema - v0.0.439.
//!
//! The canonical ticket intent schema output by translator.

use serde::{Deserialize, Serialize};

use super::types::{CanonicalIntent, Department, RiskLevel};

/// The canonical ticket intent schema output by translator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketIntentSchema {
    /// Original user query.
    pub user_query: String,
    /// Detected intent.
    pub intent: CanonicalIntent,
    /// Department that should handle this.
    pub department: Department,
    /// Required evidence (probe IDs that must succeed).
    pub required_evidence: Vec<String>,
    /// Optional evidence (nice to have).
    #[serde(default)]
    pub optional_evidence: Vec<String>,
    /// Whether clarification is needed.
    #[serde(default)]
    pub need_clarification: bool,
    /// Clarifying question if needed (max 120 chars).
    #[serde(default)]
    pub clarifying_question: Option<String>,
    /// Risk level.
    #[serde(default)]
    pub risk_level: RiskLevel,
}

impl TicketIntentSchema {
    /// Create a new schema with required fields.
    pub fn new(query: &str, intent: CanonicalIntent, department: Department) -> Self {
        Self {
            user_query: query.to_string(),
            intent,
            department,
            required_evidence: Vec::new(),
            optional_evidence: Vec::new(),
            need_clarification: false,
            clarifying_question: None,
            risk_level: RiskLevel::ReadOnly,
        }
    }

    /// Add required evidence.
    pub fn with_required_evidence(mut self, probes: Vec<&str>) -> Self {
        self.required_evidence = probes.into_iter().map(String::from).collect();
        self
    }

    /// Add optional evidence.
    pub fn with_optional_evidence(mut self, probes: Vec<&str>) -> Self {
        self.optional_evidence = probes.into_iter().map(String::from).collect();
        self
    }

    /// Set clarification needed.
    pub fn needs_clarification(mut self, question: &str) -> Self {
        self.need_clarification = true;
        // Truncate to 120 chars
        self.clarifying_question = Some(if question.len() > 120 {
            format!("{}...", &question[..117])
        } else {
            question.to_string()
        });
        self
    }

    /// Set risk level.
    pub fn with_risk(mut self, risk: RiskLevel) -> Self {
        self.risk_level = risk;
        self
    }

    /// Validate the schema.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.user_query.is_empty() {
            errors.push("user_query cannot be empty".to_string());
        }

        if self.need_clarification {
            if self.clarifying_question.is_none() {
                errors.push("need_clarification=true but no clarifying_question".to_string());
            } else if let Some(q) = &self.clarifying_question {
                if q.len() > 120 {
                    errors.push(format!(
                        "clarifying_question exceeds 120 chars: {}",
                        q.len()
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Serialize to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}
