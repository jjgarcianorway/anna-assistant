//! Answer field types and values - v0.0.437.
//!
//! Defines the structure of individual answer fields and their value types.

use super::intent::Units;
use super::formatters::{format_bytes, format_duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A field in the answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerField {
    /// Field name.
    pub name: String,
    /// Field value.
    pub value: AnswerValue,
    /// Evidence ID that supports this field.
    pub evidence_id: Option<String>,
}

impl AnswerField {
    /// Create a new answer field.
    pub fn new(name: &str, value: AnswerValue) -> Self {
        Self {
            name: name.to_string(),
            value,
            evidence_id: None,
        }
    }

    /// Attach evidence to this field.
    pub fn with_evidence(mut self, evidence_id: &str) -> Self {
        self.evidence_id = Some(evidence_id.to_string());
        self
    }
}

/// Value types for answer fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnswerValue {
    /// String value.
    String(String),
    /// Numeric value.
    Number(f64),
    /// Boolean value.
    Boolean(bool),
    /// List of strings.
    StringList(Vec<String>),
    /// List of key-value pairs.
    ObjectList(Vec<HashMap<String, String>>),
}

impl AnswerValue {
    /// Format value with units.
    pub fn format_with_units(&self, units: Units) -> String {
        match (self, units) {
            (AnswerValue::Number(n), Units::Bytes) => format_bytes(*n as u64),
            (AnswerValue::Number(n), Units::Percent) => format!("{:.1}%", n),
            (AnswerValue::Number(n), Units::Seconds) => format_duration(*n),
            (AnswerValue::Number(n), Units::Human) => format!("{}", n),
            (AnswerValue::String(s), _) => s.clone(),
            (AnswerValue::Boolean(b), _) => if *b { "yes" } else { "no" }.to_string(),
            (AnswerValue::StringList(list), _) => list.join(", "),
            (AnswerValue::ObjectList(_), _) => "[complex data]".to_string(),
        }
    }
}
