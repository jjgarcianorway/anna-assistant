//! Evidence Gate Types - v0.0.439.
//!
//! Core types for evidence gating system.

use std::collections::HashMap;

/// Result of a probe execution.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// Probe ID.
    pub probe_id: String,
    /// Whether the probe succeeded.
    pub success: bool,
    /// The probe output (if successful).
    pub output: Option<String>,
    /// Error message (if failed).
    pub error: Option<String>,
}

impl ProbeResult {
    /// Create a successful probe result.
    pub fn success(probe_id: &str, output: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: true,
            output: Some(output.to_string()),
            error: None,
        }
    }

    /// Create a failed probe result.
    pub fn failed(probe_id: &str, error: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: false,
            output: None,
            error: Some(error.to_string()),
        }
    }
}

/// Evidence collection status.
#[derive(Debug, Clone)]
pub struct EvidenceStatus {
    /// Collected probe results.
    pub probes: HashMap<String, ProbeResult>,
    /// Missing required probes.
    pub missing_required: Vec<String>,
    /// Failed required probes.
    pub failed_required: Vec<String>,
    /// Whether all required evidence is available.
    pub all_required_available: bool,
}

impl EvidenceStatus {
    /// Create from probe results and requirements.
    pub fn from_probes(probes: &HashMap<String, ProbeResult>, required: &[&str]) -> Self {
        let mut missing_required = Vec::new();
        let mut failed_required = Vec::new();

        for probe_id in required {
            match probes.get(*probe_id) {
                None => missing_required.push(probe_id.to_string()),
                Some(result) if !result.success => failed_required.push(probe_id.to_string()),
                _ => {}
            }
        }

        let all_required_available = missing_required.is_empty() && failed_required.is_empty();

        Self {
            probes: probes.clone(),
            missing_required,
            failed_required,
            all_required_available,
        }
    }

    /// Get successful probe output.
    pub fn get_output(&self, probe_id: &str) -> Option<&str> {
        self.probes.get(probe_id).and_then(|p| p.output.as_deref())
    }
}

/// Decision from evidence gate.
#[derive(Debug, Clone)]
pub enum GateDecision {
    /// Need more data - run these probes first.
    NeedMoreData {
        missing_probes: Vec<String>,
        reason: String,
    },
    /// Can answer directly from probes (no specialist needed).
    AnswerFromProbes { answer: DirectAnswer },
    /// Need specialist for synthesis.
    NeedSpecialist {
        evidence: EvidenceStatus,
        reason: String,
    },
    /// Intent unknown, need clarification.
    NeedClarification { question: String },
}

/// A direct answer from probe data.
#[derive(Debug, Clone)]
pub struct DirectAnswer {
    /// The factual answer.
    pub answer: String,
    /// Supporting evidence (probe outputs).
    pub evidence: Vec<(String, String)>, // (probe_id, output)
    /// Confidence (always high for direct facts).
    pub confidence: f64,
}

impl DirectAnswer {
    /// Create a new direct answer.
    pub fn new(answer: &str) -> Self {
        Self {
            answer: answer.to_string(),
            evidence: Vec::new(),
            confidence: 0.95,
        }
    }

    /// Add evidence.
    pub fn with_evidence(mut self, probe_id: &str, output: &str) -> Self {
        self.evidence
            .push((probe_id.to_string(), output.to_string()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_status_missing() {
        let probes = HashMap::new();
        let status = EvidenceStatus::from_probes(&probes, &["probe_a", "probe_b"]);

        assert!(!status.all_required_available);
        assert_eq!(status.missing_required.len(), 2);
    }

    #[test]
    fn test_evidence_status_complete() {
        let mut probes = HashMap::new();
        probes.insert(
            "probe_a".to_string(),
            ProbeResult::success("probe_a", "output"),
        );
        probes.insert(
            "probe_b".to_string(),
            ProbeResult::success("probe_b", "output"),
        );

        let status = EvidenceStatus::from_probes(&probes, &["probe_a", "probe_b"]);
        assert!(status.all_required_available);
    }
}
