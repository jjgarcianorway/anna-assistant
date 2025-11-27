//! LLM Protocol v0.18.0
//!
//! Redesigned protocol for step-by-step orchestration:
//! - Junior proposes ONE action at a time (probe, clarification, answer, escalate)
//! - Senior only called when Junior escalates or confidence is low
//! - Clear separation between planning, execution, and validation

use serde::{Deserialize, Serialize};

// ============================================================================
// Protocol Version
// ============================================================================

pub const PROTOCOL_VERSION: &str = "0.18.0";

// ============================================================================
// Junior (LLM-A) Step Response - ONE action per iteration
// ============================================================================

/// Junior's step response - exactly ONE of these actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum JuniorStep {
    /// Request to run a single probe
    RunProbe {
        probe_id: String,
        reason: String,
    },
    /// Request to run a safe shell command (from whitelist)
    RunCommand {
        cmd: String,
        reason: String,
    },
    /// Ask user for clarification
    AskClarification {
        question: String,
    },
    /// Propose a draft answer (may be final or need Senior review)
    ProposeAnswer {
        text: String,
        citations: Vec<String>,
        scores: JuniorScores,
        ready_for_user: bool,
    },
    /// Escalate to Senior for review
    EscalateToSenior {
        summary: EscalationSummary,
    },
}

/// Junior's self-assessed scores
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JuniorScores {
    pub evidence: u8,   // 0-100
    pub reasoning: u8,  // 0-100
    pub coverage: u8,   // 0-100
    pub overall: u8,    // 0-100
}

/// Summary for Senior when escalating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationSummary {
    pub original_question: String,
    pub probes_run: Vec<ProbeResultV18>,
    pub draft_answer: Option<String>,
    pub draft_scores: Option<JuniorScores>,
    pub reason_for_escalation: String,
    pub request: EscalationRequest,
}

/// What Junior is asking Senior to do
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationRequest {
    /// Audit and correct the draft answer
    AuditAnswer,
    /// Propose a better probe or command
    ProposeBetterProbe,
}

/// Result of a probe execution (v0.18.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResultV18 {
    pub probe_id: String,
    pub raw_output: String,
    pub success: bool,
}

// ============================================================================
// Senior (LLM-B) Response
// ============================================================================

/// Senior's response after reviewing Junior's work
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SeniorResponse {
    /// Approve the answer as-is
    ApproveAnswer {
        scores: SeniorScores,
    },
    /// Correct the answer and approve
    CorrectAnswer {
        text: String,
        scores: SeniorScores,
        corrections: Vec<String>,
    },
    /// Request additional probe/command
    RequestProbe {
        probe_id: String,
        reason: String,
    },
    /// Request additional command
    RequestCommand {
        cmd: String,
        reason: String,
    },
    /// Cannot answer - refuse with reason
    Refuse {
        reason: String,
        probes_attempted: Vec<String>,
    },
}

/// Senior's assessed scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorScores {
    pub evidence: u8,   // 0-100
    pub reasoning: u8,  // 0-100
    pub coverage: u8,   // 0-100
    pub overall: u8,    // 0-100
    pub reliability_note: String,
}

impl SeniorScores {
    pub fn new(evidence: u8, reasoning: u8, coverage: u8, note: &str) -> Self {
        let overall = evidence.min(reasoning).min(coverage);
        Self {
            evidence,
            reasoning,
            coverage,
            overall,
            reliability_note: note.to_string(),
        }
    }
}

// ============================================================================
// Question Loop State
// ============================================================================

/// Tracks state across the question-handling loop
#[derive(Debug, Clone, Default)]
pub struct QuestionLoopState {
    pub question: String,
    pub probes_run: Vec<ProbeResultV18>,
    pub clarifications: Vec<Clarification>,
    pub iteration: usize,
    pub junior_drafts: Vec<JuniorDraft>,
    pub status: LoopStatus,
}

/// A clarification exchange with user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    pub question: String,
    pub answer: String,
}

/// Junior's draft answer attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorDraft {
    pub text: String,
    pub scores: JuniorScores,
    pub iteration: usize,
}

/// Current loop status
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LoopStatus {
    #[default]
    InProgress,
    AwaitingClarification,
    AwaitingSenior,
    Complete,
    Refused,
    MaxIterations,
}

// ============================================================================
// Final Answer (what goes to user)
// ============================================================================

/// Final answer delivered to user via annactl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalAnswerV18 {
    /// The answer text
    pub text: String,
    /// Reliability score (0-100)
    pub reliability: u8,
    /// Brief reliability note
    pub reliability_note: String,
    /// Citations (probe_ids or commands used)
    pub citations: Vec<String>,
    /// Was this a refusal?
    pub is_refusal: bool,
    /// Number of iterations used
    pub iterations: usize,
    /// Was Senior involved?
    pub senior_reviewed: bool,
}

impl FinalAnswerV18 {
    /// Format for display to user
    pub fn format_for_display(&self) -> String {
        let mut output = String::new();

        // Answer text
        output.push_str(&self.text);
        output.push_str("\n\n");

        // Reliability line
        let reliability_icon = match self.reliability {
            90..=100 => "HIGH",
            70..=89 => "MEDIUM",
            _ => "LOW",
        };
        output.push_str(&format!(
            "Reliability: {} / 100 ({})",
            self.reliability, reliability_icon
        ));

        // Citations if present
        if !self.citations.is_empty() {
            output.push_str("\nSources: ");
            output.push_str(&self.citations.join(", "));
        }

        output
    }
}

// ============================================================================
// Request/Response wrappers for JSON communication
// ============================================================================

/// Request to Junior (what annad sends)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorRequest {
    pub protocol: String,
    pub question: String,
    pub available_probes: Vec<String>,
    pub available_commands: Vec<String>,
    pub history: Vec<HistoryEntry>,
    pub iteration: usize,
}

/// History entry for context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HistoryEntry {
    ProbeResult {
        probe_id: String,
        output: String,
        success: bool,
    },
    CommandResult {
        cmd: String,
        output: String,
        success: bool,
    },
    Clarification {
        question: String,
        answer: String,
    },
}

/// Request to Senior (what annad sends when Junior escalates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorRequest {
    pub protocol: String,
    pub original_question: String,
    pub history: Vec<HistoryEntry>,
    pub junior_draft: Option<String>,
    pub junior_scores: Option<JuniorScores>,
    pub escalation_reason: String,
    pub request_type: String, // "audit_answer" or "propose_better_probe"
}

// ============================================================================
// Constants
// ============================================================================

/// Maximum iterations before giving up
pub const MAX_ITERATIONS: usize = 10;

/// Minimum score to deliver answer without Senior review
pub const MIN_SCORE_WITHOUT_SENIOR: u8 = 85;

/// Minimum score for green (high confidence)
pub const SCORE_GREEN: u8 = 90;

/// Minimum score for yellow (medium confidence)
pub const SCORE_YELLOW: u8 = 70;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "0.18.0");
    }

    #[test]
    fn test_junior_step_run_probe() {
        let step = JuniorStep::RunProbe {
            probe_id: "cpu.info".to_string(),
            reason: "need CPU model".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("run_probe"));
        assert!(json.contains("cpu.info"));
    }

    #[test]
    fn test_junior_step_propose_answer() {
        let step = JuniorStep::ProposeAnswer {
            text: "You have 32 GB RAM".to_string(),
            citations: vec!["mem.info".to_string()],
            scores: JuniorScores {
                evidence: 95,
                reasoning: 90,
                coverage: 100,
                overall: 90,
            },
            ready_for_user: true,
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("propose_answer"));
        assert!(json.contains("32 GB RAM"));
    }

    #[test]
    fn test_senior_scores_overall() {
        let scores = SeniorScores::new(90, 85, 95, "strong evidence from mem.info");
        assert_eq!(scores.overall, 85); // min of all three
    }

    #[test]
    fn test_final_answer_display() {
        let answer = FinalAnswerV18 {
            text: "You have an Intel Core i9-14900HX CPU.".to_string(),
            reliability: 92,
            reliability_note: "strong evidence from cpu.info".to_string(),
            citations: vec!["cpu.info".to_string()],
            is_refusal: false,
            iterations: 2,
            senior_reviewed: false,
        };
        let display = answer.format_for_display();
        assert!(display.contains("Intel Core i9"));
        assert!(display.contains("92 / 100"));
        assert!(display.contains("HIGH"));
    }
}
