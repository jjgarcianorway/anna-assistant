//! Anna Brain v10.0.0 - Contracts
//!
//! Strict JSON protocol between Anna (Rust) and LLM.
//! Every field has a purpose. No ambiguity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reliability label for user-facing output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReliabilityLabel {
    /// 0.9-1.0: Strong evidence, high confidence
    High,
    /// 0.7-0.89: Good evidence, minor gaps
    Medium,
    /// 0.4-0.69: Partial evidence, some uncertainty
    Low,
    /// 0.0-0.39: Insufficient evidence, speculation
    VeryLow,
}

impl ReliabilityLabel {
    /// Convert a numeric reliability score to a label
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => Self::High,
            s if s >= 0.7 => Self::Medium,
            s if s >= 0.4 => Self::Low,
            _ => Self::VeryLow,
        }
    }

    /// Get the display string
    pub fn display(&self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
            Self::VeryLow => "VERY LOW",
        }
    }

    /// Get emoji for display
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::High => "🟢",
            Self::Medium => "🟡",
            Self::Low => "🟠",
            Self::VeryLow => "🔴",
        }
    }
}

/// A piece of evidence collected from tool output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Unique identifier (e.g., "E1", "E2")
    pub id: String,
    /// Source tool name
    pub source: String,
    /// Brief description of what was checked
    pub description: String,
    /// The actual content/output
    pub content: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
}

impl EvidenceItem {
    /// Create a new evidence item from tool result
    pub fn from_tool_result(id: &str, tool: &str, desc: &str, stdout: &str, exit_code: i32) -> Self {
        Self {
            id: id.to_string(),
            source: tool.to_string(),
            description: desc.to_string(),
            content: stdout.to_string(),
            exit_code,
        }
    }

    /// Check if this evidence represents a successful operation
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// The type of step in the brain loop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    /// Need to run a tool to gather evidence
    DecideTool,
    /// Ready to provide final answer with evidence
    FinalAnswer,
    /// Need user input to proceed
    AskUser,
}

/// A tool request from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    /// Tool name from catalog
    pub tool: String,
    /// Arguments for the tool
    #[serde(default)]
    pub arguments: HashMap<String, String>,
    /// Why this tool is needed (for transparency)
    pub why: String,
}

/// A single step in the brain's reasoning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainStep {
    /// The type of step
    pub step_type: StepType,
    /// Tool to run (when step_type = decide_tool)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_request: Option<ToolRequest>,
    /// Final answer (when step_type = final_answer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    /// Evidence references used in answer (e.g., ["E1", "E2"])
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    /// Reliability score 0.0-1.0
    pub reliability: f32,
    /// Explanation of reasoning
    pub reasoning: String,
    /// Question for user (when step_type = ask_user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_question: Option<String>,
}

impl BrainStep {
    /// Check if this is a final answer
    pub fn is_final(&self) -> bool {
        self.step_type == StepType::FinalAnswer
    }

    /// Check if user input is needed
    pub fn needs_user_input(&self) -> bool {
        self.step_type == StepType::AskUser
    }

    /// Get the reliability label
    pub fn reliability_label(&self) -> ReliabilityLabel {
        ReliabilityLabel::from_score(self.reliability)
    }
}

/// The complete state of a brain session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSession {
    /// Original user question
    pub question: String,
    /// System telemetry snapshot
    pub telemetry: serde_json::Value,
    /// Collected evidence from tools
    pub evidence: Vec<EvidenceItem>,
    /// Available tools
    pub tool_catalog: Vec<crate::brain_v10::ToolSchema>,
    /// Current iteration number
    pub iteration: usize,
}

impl BrainSession {
    /// Create a new session
    pub fn new(
        question: &str,
        telemetry: serde_json::Value,
        tool_catalog: Vec<crate::brain_v10::ToolSchema>,
    ) -> Self {
        Self {
            question: question.to_string(),
            telemetry,
            evidence: Vec::new(),
            tool_catalog,
            iteration: 0,
        }
    }

    /// Add evidence from a tool result
    pub fn add_evidence(&mut self, tool: &str, desc: &str, stdout: &str, exit_code: i32) {
        let id = format!("E{}", self.evidence.len() + 1);
        self.evidence.push(EvidenceItem::from_tool_result(
            &id, tool, desc, stdout, exit_code,
        ));
    }

    /// Get evidence by ID
    pub fn get_evidence(&self, id: &str) -> Option<&EvidenceItem> {
        self.evidence.iter().find(|e| e.id == id)
    }

    /// Increment iteration counter
    pub fn next_iteration(&mut self) {
        self.iteration += 1;
    }
}

/// Result from a brain session
#[derive(Debug, Clone)]
pub enum SessionResult {
    /// Final answer with evidence
    Answer {
        text: String,
        reliability: f32,
        label: ReliabilityLabel,
        evidence_refs: Vec<String>,
    },
    /// Need user input to continue
    NeedsUserInput {
        question: String,
    },
    /// Max iterations reached without confident answer
    MaxIterationsReached {
        best_answer: String,
        reliability: f32,
    },
}

impl SessionResult {
    /// Format the result for user display
    pub fn format_for_user(&self) -> String {
        match self {
            Self::Answer { text, reliability, label, evidence_refs } => {
                let mut output = text.clone();

                // Add reliability footer if not HIGH
                if *label != ReliabilityLabel::High {
                    output.push_str(&format!(
                        "\n\n{}  Confidence: {} ({:.0}%)",
                        label.emoji(),
                        label.display(),
                        reliability * 100.0
                    ));
                }

                // Add evidence summary if any
                if !evidence_refs.is_empty() {
                    output.push_str(&format!(
                        "\n📊  Evidence: {}",
                        evidence_refs.join(", ")
                    ));
                }

                output
            }
            Self::NeedsUserInput { question } => {
                format!("❓  {}", question)
            }
            Self::MaxIterationsReached { best_answer, reliability } => {
                let label = ReliabilityLabel::from_score(*reliability);
                format!(
                    "{}\n\n{}  Confidence: {} ({:.0}%) - reached max iterations",
                    best_answer,
                    label.emoji(),
                    label.display(),
                    reliability * 100.0
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_labels() {
        assert_eq!(ReliabilityLabel::from_score(0.95), ReliabilityLabel::High);
        assert_eq!(ReliabilityLabel::from_score(0.75), ReliabilityLabel::Medium);
        assert_eq!(ReliabilityLabel::from_score(0.5), ReliabilityLabel::Low);
        assert_eq!(ReliabilityLabel::from_score(0.2), ReliabilityLabel::VeryLow);
    }

    #[test]
    fn test_evidence_item() {
        let evidence = EvidenceItem::from_tool_result(
            "E1",
            "run_shell",
            "Check if Steam is installed",
            "local/steam 1.0.0.85-1",
            0,
        );
        assert!(evidence.is_success());
        assert_eq!(evidence.id, "E1");
    }

    #[test]
    fn test_session_add_evidence() {
        let mut session = BrainSession::new(
            "Is Steam installed?",
            serde_json::json!({}),
            vec![],
        );
        session.add_evidence("run_shell", "pacman -Qs steam", "local/steam 1.0.0", 0);

        assert_eq!(session.evidence.len(), 1);
        assert_eq!(session.evidence[0].id, "E1");
    }

    #[test]
    fn test_step_type_serialization() {
        let step = BrainStep {
            step_type: StepType::FinalAnswer,
            tool_request: None,
            answer: Some("Yes, Steam is installed.".to_string()),
            evidence_refs: vec!["E1".to_string()],
            reliability: 0.95,
            reasoning: "Found Steam package in pacman query".to_string(),
            user_question: None,
        };

        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"step_type\":\"final_answer\""));
    }
}
