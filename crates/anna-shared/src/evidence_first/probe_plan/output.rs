//! Probe output structures.
//!
//! Structures for representing probe execution results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of executing a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutput {
    /// Primitive ID.
    pub primitive_id: String,
    /// Raw command output.
    pub raw_output: String,
    /// Parsed/structured output (if parser succeeded).
    pub parsed: Option<ParsedOutput>,
    /// Exit code.
    pub exit_code: Option<i32>,
    /// Execution time in ms.
    pub execution_time_ms: u64,
    /// Any error message.
    pub error: Option<String>,
}

impl ProbeOutput {
    /// Check if probe succeeded.
    pub fn success(&self) -> bool {
        self.exit_code == Some(0) && self.error.is_none()
    }

    /// Get summary for citation.
    pub fn summary(&self, max_len: usize) -> String {
        if let Some(parsed) = &self.parsed {
            parsed.summary.clone()
        } else if self.raw_output.len() > max_len {
            format!("{}...", &self.raw_output[..max_len])
        } else {
            self.raw_output.clone()
        }
    }
}

/// Parsed output from a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOutput {
    /// Type of parsed data.
    pub kind: ParsedKind,
    /// Human-readable summary.
    pub summary: String,
    /// Key-value pairs extracted.
    pub fields: HashMap<String, String>,
}

/// Kind of parsed output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedKind {
    /// Time measurement (e.g., boot time).
    TimeMeasurement,
    /// List of items (e.g., failed services).
    ItemList,
    /// Generic key-value.
    KeyValue,
    /// Unparsed raw text.
    Raw,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_output_success() {
        let output = ProbeOutput {
            primitive_id: "test".to_string(),
            raw_output: "output".to_string(),
            parsed: None,
            exit_code: Some(0),
            execution_time_ms: 100,
            error: None,
        };
        assert!(output.success());

        let failed = ProbeOutput {
            primitive_id: "test".to_string(),
            raw_output: "".to_string(),
            parsed: None,
            exit_code: Some(1),
            execution_time_ms: 100,
            error: Some("failed".to_string()),
        };
        assert!(!failed.success());
    }
}
