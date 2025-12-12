//! Structured ticket logs for learning (v0.0.406).
//!
//! Stores solved tickets in a consistent format for:
//! - Recipe generation (future)
//! - Pattern analysis
//! - LLM load measurement
//! - v0.0.407: Truthful stats with explicit state tracking
//!
//! Storage: ~/.anna/tickets/{id}.json or /var/lib/anna/tickets/{id}.json

use crate::rpc::{ProbeResult, SpecialistDomain};
use crate::ticket_state::{ErrorKind, TicketState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::debug;

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
fn truncate_output(s: &str, max: usize) -> String {
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

impl TicketLog {
    /// Create a new ticket log entry
    pub fn new(
        id: impl Into<String>,
        domain: SpecialistDomain,
        intent: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            timestamp: chrono_timestamp(),
            domain: domain.to_string().to_lowercase(),
            intent: intent.into(),
            query: query.into(),
            params: HashMap::new(),
            probes: vec![],
            docs_used: vec![],
            solver_output: SolverOutput {
                solver_type: "unknown".to_string(),
                analysis: None,
                model: None,
                tokens_used: None,
            },
            commands_run: vec![],
            answer: String::new(),
            result: TicketResult::Success,
            handled_by: "unknown".to_string(),
            duration_ms: 0,
            reliability_score: 0,
            state: Some(TicketState::Created),
            error_kind: None,
            escalated: false,
            escalation_path: None,
            llm_calls: 0,
            retry_count: 0,
        }
    }

    /// v0.0.407: Set explicit state
    pub fn with_state(mut self, state: TicketState) -> Self {
        self.state = Some(state);
        self
    }

    /// v0.0.407: Set error info
    pub fn with_error(mut self, kind: ErrorKind) -> Self {
        self.error_kind = Some(kind);
        self.state = Some(TicketState::Failed);
        self.result = TicketResult::Failed;
        self
    }

    /// v0.0.407: Set escalation info
    pub fn with_escalation(mut self, path: &str) -> Self {
        self.escalated = true;
        self.escalation_path = Some(path.to_string());
        self
    }

    /// v0.0.407: Set LLM call count
    pub fn with_llm_calls(mut self, count: u8) -> Self {
        self.llm_calls = count;
        self
    }

    /// v0.0.407: Check if this ticket represents a real success
    pub fn is_real_success(&self) -> bool {
        // Use explicit state if it's terminal (Success/Failed)
        if let Some(state) = &self.state {
            if matches!(state, TicketState::Success | TicketState::Failed) {
                return *state == TicketState::Success;
            }
        }
        // Legacy or in-progress: use result field
        self.result == TicketResult::Success && self.reliability_score >= 50
    }

    /// v0.0.407: Check if this ticket reached answered state
    pub fn reached_answered(&self) -> bool {
        if let Some(state) = &self.state {
            return matches!(
                state,
                TicketState::Answered | TicketState::CommandsRun | TicketState::Success
            );
        }
        // Legacy: has non-empty answer
        !self.answer.is_empty()
    }

    /// v0.0.407: Check if this is an LLM failure
    /// v0.0.409: Also includes ValidationFailed (LLM output invalid content)
    pub fn is_llm_failure(&self) -> bool {
        matches!(
            &self.error_kind,
            Some(ErrorKind::LlmTimeout)
                | Some(ErrorKind::LlmParseError)
                | Some(ErrorKind::ValidationFailed)
        )
    }

    /// Set parameters
    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }

    /// Add probes from probe results
    pub fn with_probes(mut self, probes: &[ProbeResult]) -> Self {
        self.probes = probes.iter().map(ProbeLog::from).collect();
        self
    }

    /// Set solver output
    pub fn with_solver(mut self, solver_type: impl Into<String>) -> Self {
        self.solver_output.solver_type = solver_type.into();
        self
    }

    /// Set LLM details
    pub fn with_llm_details(
        mut self,
        model: &str,
        analysis: Option<String>,
        tokens: Option<u32>,
    ) -> Self {
        self.solver_output.model = Some(model.to_string());
        self.solver_output.analysis = analysis;
        self.solver_output.tokens_used = tokens;
        self
    }

    /// Set answer
    pub fn with_answer(mut self, answer: impl Into<String>, result: TicketResult) -> Self {
        self.answer = answer.into();
        self.result = result;
        self
    }

    /// Set metrics
    pub fn with_metrics(mut self, duration_ms: u64, reliability: u8) -> Self {
        self.duration_ms = duration_ms;
        self.reliability_score = reliability;
        self
    }

    /// Set handler (who processed this)
    pub fn with_handler(mut self, handler: impl Into<String>) -> Self {
        self.handled_by = handler.into();
        self
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let dir = ticket_log_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        debug!("Saved ticket log: {}", path.display());
        Ok(())
    }

    /// Load from disk
    pub fn load(id: &str) -> Result<Self, std::io::Error> {
        let path = ticket_log_dir().join(format!("{}.json", id));
        let json = fs::read_to_string(&path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Get ticket log directory
pub fn ticket_log_dir() -> PathBuf {
    // Try /var/lib/anna/tickets first, fall back to ~/.anna/tickets
    let var_lib = PathBuf::from("/var/lib/anna/tickets");
    if var_lib.exists() || fs::create_dir_all(&var_lib).is_ok() {
        return var_lib;
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".anna")
        .join("tickets")
}

/// Generate ISO 8601 timestamp
fn chrono_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Load all ticket logs (for analysis)
pub fn load_all_tickets() -> Vec<TicketLog> {
    let dir = ticket_log_dir();
    let mut tickets = vec![];

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(ticket) = serde_json::from_str::<TicketLog>(&json) {
                        tickets.push(ticket);
                    }
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    tickets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    tickets
}

/// Load recent tickets (last N)
pub fn load_recent_tickets(limit: usize) -> Vec<TicketLog> {
    let mut tickets = load_all_tickets();
    tickets.truncate(limit);
    tickets
}

// Re-export stats from ticket_stats module for backward compatibility
pub use crate::ticket_stats::{calculate_stats, TicketStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_log_creation() {
        let log = TicketLog::new(
            "SVC-0001",
            SpecialistDomain::Services,
            "diagnose",
            "why is sshd failing",
        )
        .with_solver("llm:junior")
        .with_handler("llm:junior")
        .with_metrics(500, 85);

        assert_eq!(log.id, "SVC-0001");
        assert_eq!(log.domain, "services");
        assert_eq!(log.handled_by, "llm:junior");
        assert_eq!(log.reliability_score, 85);
    }

    #[test]
    fn test_probe_log_from() {
        let probe = ProbeResult {
            command: "systemctl --failed".to_string(),
            stdout: "0 failed units".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timing_ms: 50,
        };

        let log = ProbeLog::from(&probe);
        assert_eq!(log.exit_code, 0);
        assert_eq!(log.duration_ms, 50);
    }

    #[test]
    fn test_truncate_output() {
        let short = "hello";
        assert_eq!(truncate_output(short, 10), short);

        let long = "a".repeat(100);
        let truncated = truncate_output(&long, 20);
        assert!(truncated.contains("...[truncated]"));
        assert!(truncated.len() < 50);
    }
}
