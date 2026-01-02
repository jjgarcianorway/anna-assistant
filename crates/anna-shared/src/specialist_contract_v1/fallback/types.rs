//! Fallback types - v0.0.440.
//!
//! Core types for fallback responses and context.

use serde::{Deserialize, Serialize};

/// Fallback response when specialist fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackResponse {
    /// Case ID.
    pub case_id: String,
    /// Short factual answer OR "insufficient evidence".
    pub answer: String,
    /// Confidence (typically lower than specialist).
    pub confidence: f64,
    /// Missing evidence that would have helped.
    #[serde(default)]
    pub missing_evidence: Vec<String>,
    /// Next probes to run if more evidence needed.
    #[serde(default)]
    pub next_probe: Vec<String>,
}

impl FallbackResponse {
    /// Create a new fallback response.
    pub fn new(case_id: &str, answer: &str, confidence: f64) -> Self {
        Self {
            case_id: case_id.to_string(),
            answer: answer.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            missing_evidence: Vec::new(),
            next_probe: Vec::new(),
        }
    }

    /// Create an "insufficient evidence" response.
    pub fn insufficient_evidence(case_id: &str, missing: Vec<&str>) -> Self {
        Self {
            case_id: case_id.to_string(),
            answer: "Insufficient evidence to answer.".to_string(),
            confidence: 0.0,
            missing_evidence: missing.into_iter().map(String::from).collect(),
            next_probe: Vec::new(),
        }
    }

    /// Add missing evidence.
    pub fn with_missing(mut self, probe_id: &str) -> Self {
        self.missing_evidence.push(probe_id.to_string());
        self
    }

    /// Add next probe.
    pub fn with_next_probe(mut self, probe_id: &str) -> Self {
        self.next_probe.push(probe_id.to_string());
        self
    }

    /// Check if this is an insufficient evidence response.
    pub fn is_insufficient(&self) -> bool {
        self.answer.contains("Insufficient evidence") || self.confidence == 0.0
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

/// Probe evidence for fallback summarizer.
#[derive(Debug, Clone)]
pub struct ProbeEvidence {
    /// Probe ID.
    pub probe_id: String,
    /// Probe output.
    pub output: String,
    /// Whether probe succeeded.
    pub success: bool,
}

impl ProbeEvidence {
    /// Create successful evidence.
    pub fn success(probe_id: &str, output: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            output: output.to_string(),
            success: true,
        }
    }

    /// Create failed evidence.
    pub fn failed(probe_id: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            output: String::new(),
            success: false,
        }
    }
}

/// Context for fallback decision.
#[derive(Debug, Clone)]
pub struct FallbackContext {
    /// Case ID.
    pub case_id: String,
    /// Why fallback was triggered.
    pub reason: FallbackReason,
    /// Number of specialist attempts.
    pub attempts: usize,
    /// Total time spent on specialist calls.
    pub total_time_ms: u64,
}

/// Why fallback was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackReason {
    /// Specialist timed out.
    Timeout,
    /// Specialist response invalid.
    InvalidResponse,
    /// Max retries exhausted.
    RetriesExhausted,
    /// Specialist not available.
    Unavailable,
}

impl FallbackReason {
    /// Get label for logging.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::InvalidResponse => "invalid_response",
            Self::RetriesExhausted => "retries_exhausted",
            Self::Unavailable => "unavailable",
        }
    }
}
