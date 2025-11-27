//! LLM Protocol v0.12.0
//!
//! Strict JSON request/response protocol between annad and LLM-A / LLM-B.
//! No prose allowed in responses - only valid JSON.

use super::evidence::{AvailableProbe, ProbeEvidenceV10};
use super::scoring::ReliabilityScores;
use serde::{Deserialize, Serialize};

// ============================================================================
// Protocol Version
// ============================================================================

pub const PROTOCOL_VERSION: &str = "0.12.0";

// ============================================================================
// LLM-A: Planner/Answerer Protocol
// ============================================================================

/// Request from annad to LLM-A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmARequest {
    /// Role identifier
    pub role: String,
    /// Protocol version
    pub version: String,
    /// User's question
    pub question: String,
    /// Conversation history
    #[serde(default)]
    pub history: Vec<ConversationMessage>,
    /// Available probes in the catalog
    pub available_probes: Vec<AvailableProbe>,
    /// Evidence collected so far (may be empty on first call)
    #[serde(default)]
    pub evidence: Vec<ProbeEvidenceV10>,
}

impl LlmARequest {
    pub fn new(question: String, available_probes: Vec<AvailableProbe>) -> Self {
        Self {
            role: "planner_answer".to_string(),
            version: PROTOCOL_VERSION.to_string(),
            question,
            history: vec![],
            available_probes,
            evidence: vec![],
        }
    }

    pub fn with_history(mut self, history: Vec<ConversationMessage>) -> Self {
        self.history = history;
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<ProbeEvidenceV10>) -> Self {
        self.evidence = evidence;
        self
    }
}

/// Conversation message in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub from: String,
    pub text: String,
}

/// Response from LLM-A to annad (must be valid JSON, no prose)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAResponse {
    /// Planning phase output
    pub plan: LlmAPlan,
    /// Draft answer if ready
    #[serde(default)]
    pub draft_answer: Option<DraftAnswer>,
    /// Self-assessed scores
    #[serde(default)]
    pub self_scores: Option<ReliabilityScores>,
    /// Does this request need more probes?
    pub needs_more_probes: bool,
    /// Should we refuse to answer?
    pub refuse_to_answer: bool,
    /// Refusal reason if refusing
    #[serde(default)]
    pub refusal_reason: Option<String>,
}

/// LLM-A planning output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAPlan {
    /// Detected intent category
    pub intent: String,
    /// Requested probes with reasons
    #[serde(default)]
    pub probe_requests: Vec<ProbeRequest>,
    /// Can answer without additional probes?
    pub can_answer_without_more_probes: bool,
}

/// Probe request with reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeRequest {
    pub probe_id: String,
    pub reason: String,
}

/// Draft answer from LLM-A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftAnswer {
    /// Human-readable answer text
    pub text: String,
    /// Citations to evidence
    #[serde(default)]
    pub citations: Vec<Citation>,
}

/// Citation to probe evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub probe_id: String,
}

// ============================================================================
// LLM-B: Auditor/Skeptic Protocol
// ============================================================================

/// Request from annad to LLM-B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBRequest {
    /// Role identifier
    pub role: String,
    /// Protocol version
    pub version: String,
    /// Original user question
    pub question: String,
    /// Draft answer from LLM-A
    pub draft_answer: DraftAnswer,
    /// Evidence collected
    pub evidence: Vec<ProbeEvidenceV10>,
    /// Self-scores from LLM-A
    pub self_scores: ReliabilityScores,
}

impl LlmBRequest {
    pub fn new(
        question: String,
        draft_answer: DraftAnswer,
        evidence: Vec<ProbeEvidenceV10>,
        self_scores: ReliabilityScores,
    ) -> Self {
        Self {
            role: "auditor".to_string(),
            version: PROTOCOL_VERSION.to_string(),
            question,
            draft_answer,
            evidence,
            self_scores,
        }
    }
}

/// Response from LLM-B to annad (must be valid JSON, no prose)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBResponse {
    /// Verdict: approve, fix_and_accept, needs_more_probes, or refuse
    pub verdict: AuditVerdict,
    /// Auditor-computed scores
    pub scores: AuditScores,
    /// Additional probes requested if verdict is needs_more_probes
    #[serde(default)]
    pub probe_requests: Vec<ProbeRequest>,
    /// Problems identified with the draft answer
    #[serde(default)]
    pub problems: Vec<String>,
    /// Suggested fix if problems found
    #[serde(default)]
    pub suggested_fix: Option<String>,
    /// Corrected answer text if verdict is fix_and_accept
    #[serde(default)]
    pub fixed_answer: Option<String>,
}

/// Audit verdict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditVerdict {
    /// Answer is adequately grounded and scored
    Approve,
    /// Minor issues fixable without new probes - use fixed_answer
    FixAndAccept,
    /// Need more probes before final answer
    NeedsMoreProbes,
    /// System should refuse to answer (rare - only when no probes can help)
    Refuse,
}

impl AuditVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditVerdict::Approve => "approve",
            AuditVerdict::FixAndAccept => "fix_and_accept",
            AuditVerdict::NeedsMoreProbes => "needs_more_probes",
            AuditVerdict::Refuse => "refuse",
        }
    }
}

/// Auditor-computed scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditScores {
    /// Evidence grounding score
    pub evidence: f64,
    /// Reasoning consistency score
    pub reasoning: f64,
    /// Question coverage score
    pub coverage: f64,
    /// Overall computed score
    pub overall: f64,
}

impl AuditScores {
    /// Compute overall score from components
    pub fn compute_overall(&self) -> f64 {
        0.4 * self.evidence + 0.3 * self.reasoning + 0.3 * self.coverage
    }

    /// Create with auto-computed overall
    pub fn new(evidence: f64, reasoning: f64, coverage: f64) -> Self {
        let overall = 0.4 * evidence + 0.3 * reasoning + 0.3 * coverage;
        Self {
            evidence,
            reasoning,
            coverage,
            overall,
        }
    }
}

// ============================================================================
// Final Answer Structure
// ============================================================================

/// Final answer to user after all processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalAnswer {
    /// Original question
    pub question: String,
    /// Answer text (or refusal message)
    pub answer: String,
    /// Was this a refusal?
    pub is_refusal: bool,
    /// Evidence citations
    pub citations: Vec<ProbeEvidenceV10>,
    /// Final scores
    pub scores: AuditScores,
    /// Confidence level
    pub confidence_level: ConfidenceLevel,
    /// Problems (if any, for low confidence)
    #[serde(default)]
    pub problems: Vec<String>,
    /// Number of LLM-A/LLM-B loop iterations used
    #[serde(default)]
    pub loop_iterations: usize,
}

/// Confidence level derived from overall score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    /// High confidence (>= 0.90)
    Green,
    /// Medium confidence (0.70 - 0.90)
    Yellow,
    /// Low confidence (< 0.70)
    Red,
}

impl ConfidenceLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.90 {
            ConfidenceLevel::Green
        } else if score >= 0.70 {
            ConfidenceLevel::Yellow
        } else {
            ConfidenceLevel::Red
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ConfidenceLevel::Green => "GREEN",
            ConfidenceLevel::Yellow => "YELLOW",
            ConfidenceLevel::Red => "RED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "0.12.0");
    }

    #[test]
    fn test_llm_a_request() {
        let req = LlmARequest::new("How much RAM?".to_string(), vec![]);
        assert_eq!(req.role, "planner_answer");
        assert_eq!(req.version, PROTOCOL_VERSION);
    }

    #[test]
    fn test_llm_b_request() {
        let draft = DraftAnswer {
            text: "You have 16GB".to_string(),
            citations: vec![Citation {
                probe_id: "mem.info".to_string(),
            }],
        };
        let scores = ReliabilityScores::default();
        let req = LlmBRequest::new("How much RAM?".to_string(), draft, vec![], scores);
        assert_eq!(req.role, "auditor");
    }

    #[test]
    fn test_audit_scores_compute() {
        let scores = AuditScores::new(0.95, 0.90, 0.85);
        // 0.4 * 0.95 + 0.3 * 0.90 + 0.3 * 0.85 = 0.38 + 0.27 + 0.255 = 0.905
        assert!((scores.overall - 0.905).abs() < 0.001);
    }

    #[test]
    fn test_confidence_level_from_score() {
        assert_eq!(ConfidenceLevel::from_score(0.95), ConfidenceLevel::Green);
        assert_eq!(ConfidenceLevel::from_score(0.90), ConfidenceLevel::Green);
        assert_eq!(ConfidenceLevel::from_score(0.85), ConfidenceLevel::Yellow);
        assert_eq!(ConfidenceLevel::from_score(0.70), ConfidenceLevel::Yellow);
        assert_eq!(ConfidenceLevel::from_score(0.65), ConfidenceLevel::Red);
    }

    #[test]
    fn test_audit_verdict() {
        assert_eq!(AuditVerdict::Approve.as_str(), "approve");
        assert_eq!(AuditVerdict::FixAndAccept.as_str(), "fix_and_accept");
        assert_eq!(AuditVerdict::NeedsMoreProbes.as_str(), "needs_more_probes");
        assert_eq!(AuditVerdict::Refuse.as_str(), "refuse");
    }

    #[test]
    fn test_llm_b_response_fixed_answer() {
        let json = r#"{
            "verdict": "fix_and_accept",
            "scores": {"evidence": 0.85, "reasoning": 0.90, "coverage": 0.80, "overall": 0.80},
            "probe_requests": [],
            "problems": ["minor wording issue"],
            "suggested_fix": null,
            "fixed_answer": "Corrected answer text here"
        }"#;
        let response: LlmBResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.verdict, AuditVerdict::FixAndAccept);
        assert_eq!(response.fixed_answer, Some("Corrected answer text here".to_string()));
    }
}
