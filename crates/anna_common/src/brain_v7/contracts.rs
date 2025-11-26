//! Anna v7 Brain - Data Contracts
//!
//! Strict, serializable types for the three-phase pipeline:
//! PLAN → EXECUTE → INTERPRET
//!
//! These are the ONLY structures that cross the LLM boundary.
//! If JSON deserialization fails, the phase fails. No partial plans.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Tool Catalog (Rust → LLM, read-only)
// ============================================================================

/// Tool descriptor shown to the planner LLM.
/// The LLM sees name + description + parameters_schema.
/// The actual command is owned by Rust and never exposed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    /// Unique tool name (e.g., "mem_info", "gpu_pci")
    pub name: String,
    /// Human-readable description for the LLM
    pub description: String,
    /// Optional JSON schema for parameters (null if no params)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters_schema: Option<serde_json::Value>,
}

// ============================================================================
// Planner Phase
// ============================================================================

/// Input to the planner LLM
#[derive(Debug, Clone, Serialize)]
pub struct PlannerInput {
    /// The user's natural language query
    pub user_query: String,
    /// Available tools (name, description, parameters only)
    pub tool_catalog: Vec<ToolDescriptor>,
}

/// Output from the planner LLM (strict JSON schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerOutput {
    /// What the user is trying to achieve (one sentence)
    pub intent: String,
    /// Breakdown of subtasks needed to answer
    pub subtasks: Vec<Subtask>,
    /// Tools to call for each subtask
    pub tool_calls: Vec<ToolCall>,
    /// What evidence each tool should provide
    pub expected_evidence: Vec<ExpectedEvidence>,
    /// What this plan cannot answer
    pub limitations: Limitations,
}

/// A subtask identified by the planner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    /// Unique short ID (e.g., "st1", "st2")
    pub id: String,
    /// Human-readable description
    pub description: String,
}

/// A tool call planned by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Which subtask this serves
    pub subtask_id: String,
    /// Tool name (MUST match a ToolDescriptor.name)
    pub tool: String,
    /// Parameters for the tool (can be empty object)
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// Why this tool is needed
    pub reason: String,
}

/// Expected evidence from a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedEvidence {
    /// Which subtask this evidence is for
    pub subtask_id: String,
    /// Tool that provides this evidence
    pub tool: String,
    /// What data this tool should provide
    pub evidence_needed: String,
}

/// What the plan cannot answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limitations {
    /// Tools that would help but don't exist
    #[serde(default)]
    pub missing_tools: Vec<String>,
    /// Parts of the query that can't be answered
    #[serde(default)]
    pub unanswerable_parts: String,
}

// ============================================================================
// Execute Phase (Rust only, no LLM)
// ============================================================================

/// Result of running a single tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRun {
    /// Unique run ID (uuid or counter)
    pub id: String,
    /// Which subtask this run serves
    pub subtask_id: String,
    /// Tool name that was run
    pub tool: String,
    /// Preview of the command (e.g., "pacman -Qs steam")
    pub command_preview: String,
    /// Standard output (may be truncated)
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// When execution started
    pub started_at: DateTime<Utc>,
    /// When execution finished
    pub finished_at: DateTime<Utc>,
}

impl ToolRun {
    /// Check if the tool ran successfully
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Duration in milliseconds
    pub fn duration_ms(&self) -> i64 {
        (self.finished_at - self.started_at).num_milliseconds()
    }
}

/// The complete evidence bundle from executing all tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub runs: Vec<ToolRun>,
    pub all_succeeded: bool,
    pub collected_at: DateTime<Utc>,
}

// ============================================================================
// Interpret Phase
// ============================================================================

/// Input to the interpreter LLM
#[derive(Debug, Clone, Serialize)]
pub struct InterpreterInput {
    /// Original user query
    pub user_query: String,
    /// The plan that was executed
    pub planner: PlannerOutput,
    /// Evidence collected from tool runs
    pub evidence_bundle: Vec<ToolRun>,
}

/// Output from the interpreter LLM (strict JSON schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterOutput {
    /// Direct answer to the user's question
    pub answer: String,
    /// Which evidence was used and how
    pub evidence_used: Vec<EvidenceUsed>,
    /// How reliable is this answer
    pub reliability: Reliability,
    /// What is unknown or uncertain
    pub uncertainty: Uncertainty,
}

/// How a piece of evidence was used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceUsed {
    /// Tool that provided this evidence
    pub tool: String,
    /// What was extracted from this tool's output
    pub summary: String,
}

/// Reliability assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reliability {
    /// Score from 0.0 (no confidence) to 1.0 (certain)
    pub score: f32,
    /// Level: "LOW" (0.0-0.4), "MEDIUM" (0.4-0.8), "HIGH" (0.8-1.0)
    pub level: String,
    /// Why this score was given
    pub reason: String,
}

impl Reliability {
    pub fn high(reason: &str) -> Self {
        Self {
            score: 0.95,
            level: "HIGH".to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn medium(score: f32, reason: &str) -> Self {
        Self {
            score: score.clamp(0.4, 0.79),
            level: "MEDIUM".to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn low(score: f32, reason: &str) -> Self {
        Self {
            score: score.clamp(0.0, 0.39),
            level: "LOW".to_string(),
            reason: reason.to_string(),
        }
    }
}

/// What is unknown or uncertain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uncertainty {
    /// Whether there are unknowns in the answer
    pub has_unknowns: bool,
    /// Details about what is unknown
    #[serde(default)]
    pub details: String,
}

// ============================================================================
// Final Result
// ============================================================================

/// Final result returned to the user
#[derive(Debug, Clone)]
pub struct BrainResult {
    /// The answer (possibly with uncertainty banner)
    pub answer: String,
    /// Reliability score (0.0-1.0)
    pub reliability_score: f32,
    /// Reliability level (LOW/MEDIUM/HIGH)
    pub reliability_level: String,
    /// Number of retries used
    pub retries_used: usize,
    /// Whether the query was successfully answered
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl BrainResult {
    pub fn success(answer: String, reliability: &Reliability) -> Self {
        Self {
            answer,
            reliability_score: reliability.score,
            reliability_level: reliability.level.clone(),
            retries_used: 0,
            success: true,
            error: None,
        }
    }

    pub fn low_reliability(answer: String, reliability: &Reliability, retries: usize) -> Self {
        let banner = format!(
            "⚠️  Answer reliability: {:.0}% ({})\nReason: {}\n\n{}",
            reliability.score * 100.0,
            reliability.level,
            reliability.reason,
            answer
        );
        Self {
            answer: banner,
            reliability_score: reliability.score,
            reliability_level: reliability.level.clone(),
            retries_used: retries,
            success: true,
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            answer: format!("❌  {}", message),
            reliability_score: 0.0,
            reliability_level: "LOW".to_string(),
            retries_used: 0,
            success: false,
            error: Some(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_output_serialization() {
        let output = PlannerOutput {
            intent: "Check RAM".to_string(),
            subtasks: vec![Subtask {
                id: "st1".to_string(),
                description: "Get memory info".to_string(),
            }],
            tool_calls: vec![ToolCall {
                subtask_id: "st1".to_string(),
                tool: "mem_info".to_string(),
                parameters: serde_json::json!({}),
                reason: "Need /proc/meminfo".to_string(),
            }],
            expected_evidence: vec![ExpectedEvidence {
                subtask_id: "st1".to_string(),
                tool: "mem_info".to_string(),
                evidence_needed: "MemTotal line".to_string(),
            }],
            limitations: Limitations {
                missing_tools: vec![],
                unanswerable_parts: String::new(),
            },
        };

        let json = serde_json::to_string(&output).unwrap();
        let parsed: PlannerOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.intent, "Check RAM");
        assert_eq!(parsed.tool_calls.len(), 1);
    }

    #[test]
    fn test_interpreter_output_serialization() {
        let output = InterpreterOutput {
            answer: "You have 32 GB RAM".to_string(),
            evidence_used: vec![EvidenceUsed {
                tool: "mem_info".to_string(),
                summary: "MemTotal: 32791612 kB".to_string(),
            }],
            reliability: Reliability::high("Direct from /proc/meminfo"),
            uncertainty: Uncertainty {
                has_unknowns: false,
                details: String::new(),
            },
        };

        let json = serde_json::to_string(&output).unwrap();
        let parsed: InterpreterOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.reliability.level, "HIGH");
    }

    #[test]
    fn test_tool_run_duration() {
        let run = ToolRun {
            id: "1".to_string(),
            subtask_id: "st1".to_string(),
            tool: "test".to_string(),
            command_preview: "echo test".to_string(),
            stdout: "test".to_string(),
            stderr: String::new(),
            exit_code: 0,
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };
        assert!(run.success());
        assert!(run.duration_ms() >= 0);
    }
}
