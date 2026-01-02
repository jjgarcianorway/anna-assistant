//! Evidence and fallback answer types (v0.0.433).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Evidence collected from a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    /// Probe name.
    pub name: String,
    /// Raw output.
    pub raw_output: String,
    /// Parsed values (if applicable).
    pub parsed: HashMap<String, String>,
    /// Whether the probe succeeded.
    pub success: bool,
    /// Duration in ms.
    pub duration_ms: u64,
}

impl ProbeEvidence {
    /// Create new evidence.
    pub fn new(name: &str, raw_output: &str, success: bool) -> Self {
        Self {
            name: name.to_string(),
            raw_output: raw_output.to_string(),
            parsed: HashMap::new(),
            success,
            duration_ms: 0,
        }
    }

    /// Add a parsed value.
    pub fn with_parsed(mut self, key: &str, value: &str) -> Self {
        self.parsed.insert(key.to_string(), value.to_string());
        self
    }

    /// Set duration.
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Fallback answer generated from evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackAnswer {
    /// Summary based on evidence.
    pub summary: String,
    /// Raw data snippet.
    pub raw_data: String,
    /// Confidence (lower than LLM answer).
    pub confidence: f32,
    /// Evidence sources used.
    pub sources: Vec<String>,
    /// What failed (LLM stage).
    pub failure_reason: String,
}

/// Truncate string to max length.
pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
