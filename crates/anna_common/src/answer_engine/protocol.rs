//! LLM Protocol v0.81.0
//!
//! Strict JSON request/response protocol between annad and LLM-A / LLM-B.
//! No prose allowed in responses - only valid JSON.
//!
//! v0.81.0: Added timing fields and StructuredAnswer conversion

use super::evidence::{AvailableProbe, ProbeEvidenceV10};
use super::scoring::ReliabilityScores;
use crate::structured_answer::{DialogTrace, QaOutput, StructuredAnswer};
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
    /// v0.17.0: Senior's synthesized answer text (preferred over draft)
    #[serde(default)]
    pub text: Option<String>,
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
// Debug Trace for Development (v0.16.3)
// ============================================================================

/// A single iteration in the LLM dialog
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DebugIteration {
    /// Iteration number (1-based)
    pub iteration: usize,
    /// LLM-A prompt sent (truncated for display)
    pub llm_a_prompt: String,
    /// LLM-A raw response
    pub llm_a_response: String,
    /// LLM-A parsed intent
    pub llm_a_intent: String,
    /// Probes requested by LLM-A
    pub llm_a_probes: Vec<String>,
    /// Whether LLM-A provided a draft answer
    pub llm_a_has_draft: bool,
    /// Probes actually executed
    pub probes_executed: Vec<String>,
    /// LLM-B prompt sent (truncated for display)
    pub llm_b_prompt: Option<String>,
    /// LLM-B raw response
    pub llm_b_response: Option<String>,
    /// LLM-B verdict
    pub llm_b_verdict: Option<String>,
    /// LLM-B confidence score
    pub llm_b_confidence: Option<f64>,
}

/// Complete debug trace for a research session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DebugTrace {
    /// All iterations in the research loop
    pub iterations: Vec<DebugIteration>,
    /// Junior model used
    pub junior_model: String,
    /// Senior model used
    pub senior_model: String,
    /// Total duration in seconds
    pub duration_secs: f64,
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
    /// LLM model used for this answer (v0.15.7)
    #[serde(default)]
    pub model_used: Option<String>,
    /// v0.15.21: Clarification question to ask user (if answer needs more info)
    #[serde(default)]
    pub clarification_needed: Option<String>,
    /// v0.16.3: Debug trace showing full LLM dialog (for development)
    #[serde(default)]
    pub debug_trace: Option<DebugTrace>,
    /// v0.81.0: Junior LLM latency in milliseconds
    #[serde(default)]
    pub junior_ms: u64,
    /// v0.81.0: Senior LLM latency in milliseconds
    #[serde(default)]
    pub senior_ms: u64,
    /// v0.81.0: Probes requested by Junior
    #[serde(default)]
    pub junior_probes: Vec<String>,
    /// v0.81.0: Whether Junior provided a draft answer
    #[serde(default)]
    pub junior_had_draft: bool,
    /// v0.81.0: Senior's verdict (approve, fix_and_accept, refuse)
    #[serde(default)]
    pub senior_verdict: Option<String>,
    /// v0.84.0: Failure cause classification for reliability analysis
    /// Values: no_probe_available, probe_data_misread, llm_hallucination,
    /// timeout_or_latency, unsupported_domain, orchestration_bug, bad_command_proposal
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_cause: Option<String>,
}

impl Default for FinalAnswer {
    fn default() -> Self {
        Self {
            question: String::new(),
            answer: String::new(),
            is_refusal: false,
            citations: Vec::new(),
            scores: AuditScores::new(0.0, 0.0, 0.0),
            confidence_level: ConfidenceLevel::Red,
            problems: Vec::new(),
            loop_iterations: 0,
            model_used: None,
            clarification_needed: None,
            debug_trace: None,
            junior_ms: 0,
            senior_ms: 0,
            junior_probes: Vec::new(),
            junior_had_draft: false,
            senior_verdict: None,
            failure_cause: None,
        }
    }
}

/// Confidence level derived from overall score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    /// High confidence (>= 0.90)
    Green,
    /// Medium confidence (0.70 - 0.90)
    Yellow,
    /// Low confidence (< 0.70)
    #[default]
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

// ============================================================================
// FinalAnswer Implementation (v0.81.0)
// ============================================================================

impl FinalAnswer {
    /// Convert to StructuredAnswer format for TUI and QA output
    pub fn to_structured_answer(&self) -> StructuredAnswer {
        let reliability_label = match self.confidence_level {
            ConfidenceLevel::Green => "Green".to_string(),
            ConfidenceLevel::Yellow => "Yellow".to_string(),
            ConfidenceLevel::Red => "Red".to_string(),
        };

        // Build headline - direct answer for Green, cautious for Yellow/Red
        let headline = if self.is_refusal {
            "Unable to answer this question".to_string()
        } else {
            // Use first sentence of answer as headline
            self.answer
                .split(|c| c == '.' || c == '\n')
                .next()
                .unwrap_or(&self.answer)
                .trim()
                .to_string()
        };

        // Build details from answer (split into bullet points)
        let details: Vec<String> = if self.is_refusal {
            self.problems.clone()
        } else {
            // Split answer into lines, filter empty
            self.answer
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
                .collect()
        };

        // Build evidence summaries from citations
        let evidence: Vec<String> = self
            .citations
            .iter()
            .map(|c| {
                let summary = c.raw.as_ref().map(|r| {
                    let first_line = r.lines().next().unwrap_or("");
                    if first_line.len() > 60 {
                        format!("{}...", &first_line[..57])
                    } else {
                        first_line.to_string()
                    }
                }).unwrap_or_else(|| "no output".to_string());
                format!("{}: {}", c.probe_id, summary)
            })
            .collect();

        StructuredAnswer::new(headline, details, evidence, reliability_label)
    }

    /// Convert to QA JSON output format (v0.82.0)
    pub fn to_qa_output(&self) -> QaOutput {
        let answer = self.to_structured_answer();
        let dialog_trace = DialogTrace::new(
            self.junior_probes.clone(),
            self.junior_had_draft,
            self.senior_verdict.as_deref().unwrap_or("unknown"),
        );

        // Determine error_kind based on answer state
        let error_kind = if self.is_refusal {
            if self.answer.to_lowercase().contains("time") {
                Some("timeout".to_string())
            } else {
                Some("refused".to_string())
            }
        } else if self.scores.overall == 0.0 {
            Some("llm_parse_error".to_string())
        } else {
            None
        };

        // probes_used = unique probes from citations
        let probes_used: Vec<String> = self.citations
            .iter()
            .map(|c| c.probe_id.clone())
            .collect();

        QaOutput::new(
            answer,
            self.scores.overall,
            self.junior_ms,
            self.senior_ms,
            self.loop_iterations as u32,
            probes_used,
            error_kind,
            dialog_trace,
        )
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
        assert_eq!(
            response.fixed_answer,
            Some("Corrected answer text here".to_string())
        );
    }
}
