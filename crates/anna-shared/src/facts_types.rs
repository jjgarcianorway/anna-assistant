//! Facts types: FactSource, FactValue (v0.0.41).
//!
//! Extracted from facts.rs to keep modules under 400 lines.

use serde::{Deserialize, Serialize};

/// Source of a fact (v0.0.41)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FactSource {
    /// Observed from running a probe
    ObservedProbe { probe_id: String, output_hash: String },
    /// Confirmed by user interaction
    UserConfirmed { transcript_id: String },
    /// Derived from other facts
    Derived { from: Vec<String> },
    /// Legacy string source (backwards compat)
    Legacy { source: String },
}

impl Default for FactSource {
    fn default() -> Self {
        Self::Legacy { source: "unknown".to_string() }
    }
}

impl From<String> for FactSource {
    fn from(s: String) -> Self {
        Self::Legacy { source: s }
    }
}

impl From<&str> for FactSource {
    fn from(s: &str) -> Self {
        Self::Legacy { source: s.to_string() }
    }
}

/// Value of a fact (v0.0.41)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FactValue {
    String(String),
    Number(i64),
    Bool(bool),
    List(Vec<String>),
}

impl Default for FactValue {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl From<String> for FactValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for FactValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<bool> for FactValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<i64> for FactValue {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<Vec<String>> for FactValue {
    fn from(v: Vec<String>) -> Self {
        Self::List(v)
    }
}

impl FactValue {
    /// Get as string (returns empty if not a string)
    pub fn as_str(&self) -> &str {
        match self {
            Self::String(s) => s,
            _ => "",
        }
    }

    /// Get as string (converts all types)
    pub fn to_string_value(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::List(v) => v.join(", "),
        }
    }

    /// Get as bool (false if not a bool)
    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            _ => false,
        }
    }

    /// Get as number (0 if not a number)
    pub fn as_number(&self) -> i64 {
        match self {
            Self::Number(n) => *n,
            _ => 0,
        }
    }

    /// Get as list (empty if not a list)
    pub fn as_list(&self) -> &[String] {
        match self {
            Self::List(v) => v,
            _ => &[],
        }
    }
}
