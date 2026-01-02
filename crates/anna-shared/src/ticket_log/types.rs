//! Type definitions for ticket logs (v0.0.406+).

use crate::rpc::ProbeResult;
use crate::ticket_state::{ErrorKind, TicketState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A structured ticket log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketLog {
    /// Ticket ID (e.g., "SVC-0023")
    pub id: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Domain (e.g., "packages", "services")
    pub domain: String,
    /// Intent (e.g., "diagnose", "configure")
    pub intent: String,
    /// Original user query
    pub query: String,
    /// Translator output parameters
    pub params: HashMap<String, String>,
    /// Probes that were run
    pub probes: Vec<ProbeLog>,
    /// Documentation snippets used (if any)
    pub docs_used: Vec<DocSnippet>,
    /// Solver output details
    pub solver_output: SolverOutput,
    /// Commands that were executed
    pub commands_run: Vec<CommandLog>,
    /// Final rendered answer
    pub answer: String,
    /// Result status (legacy, use state for new tickets)
    pub result: TicketResult,
    /// Who/what handled it
    pub handled_by: String,
    /// Processing duration in milliseconds
    pub duration_ms: u64,
    /// Reliability score (0-100)
    pub reliability_score: u8,
    /// v0.0.407: Explicit ticket state
    #[serde(default)]
    pub state: Option<TicketState>,
    /// v0.0.407: Error kind if failed
    #[serde(default)]
    pub error_kind: Option<ErrorKind>,
    /// v0.0.407: Whether ticket was escalated
    #[serde(default)]
    pub escalated: bool,
    /// v0.0.407: Escalation path (e.g., "recipe→llm", "junior→senior")
    #[serde(default)]
    pub escalation_path: Option<String>,
    /// v0.0.407: LLM call count
    #[serde(default)]
    pub llm_calls: u8,
    /// v0.0.407: Retry count
    #[serde(default)]
    pub retry_count: u8,
}

/// Probe execution log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeLog {
    /// Probe ID
    pub id: String,
    /// Command that was run
    pub command: String,
    /// Output (truncated if too long)
    pub output: String,
    /// Exit code
    pub exit_code: i32,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl From<&ProbeResult> for ProbeLog {
    fn from(p: &ProbeResult) -> Self {
        Self {
            id: extract_probe_id(&p.command),
            command: p.command.clone(),
            output: truncate_output(&p.stdout, 2000),
            exit_code: p.exit_code,
            duration_ms: p.timing_ms,
        }
    }
}

/// Extract probe ID from command (best effort)
fn extract_probe_id(cmd: &str) -> String {
    // Try to extract a meaningful ID from the command
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if !parts.is_empty() {
        parts[0].to_string()
    } else {
        "unknown".to_string()
    }
}

/// Truncate output to max length
pub(crate) fn truncate_output(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...[truncated]", &s[..max])
    }
}

/// Documentation snippet used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSnippet {
    /// Source (e.g., "arch_wiki", "man_page", "builtin")
    pub source: String,
    /// Title or identifier
    pub title: String,
    /// Relevant excerpt
    pub excerpt: String,
    /// Confidence in relevance
    pub confidence: u8,
}

/// Solver output details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverOutput {
    /// Type of solver (e.g., "recipe:check_failed", "llm:junior", "llm:senior")
    pub solver_type: String,
    /// Analysis or reasoning (if from LLM)
    pub analysis: Option<String>,
    /// Model used (if LLM)
    pub model: Option<String>,
    /// Number of tokens used (if LLM)
    pub tokens_used: Option<u32>,
}

/// Command execution log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLog {
    /// Command that was run
    pub cmd: String,
    /// Exit code
    pub exit_code: i32,
    /// Output summary
    pub output_summary: Option<String>,
}

/// Ticket result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketResult {
    /// Successfully answered
    Success,
    /// Partial answer (low confidence)
    Partial,
    /// Failed to answer
    Failed,
    /// Needs clarification from user
    NeedsClarification,
    /// User cancelled or timeout
    Cancelled,
}
