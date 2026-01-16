//! Simplified RPC types for daemon communication.

mod types;

pub use types::*;

use serde::{Deserialize, Serialize};

/// RPC methods supported by the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcMethod {
    /// Send a question to be answered (non-streaming, returns full result)
    Ask,
    /// Send a question with streaming response
    AskStreaming,
    /// Get daemon status
    Status,
    /// Reset all statistics and learning data
    Reset,
    /// Diagnose WiFi issues (Phase 43)
    DiagnoseWifi,
}

/// Streaming response types (JSON lines format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamingResponse {
    /// A dialogue step (user question, command, output, etc.)
    #[serde(rename = "step")]
    Step { step: DialogueStep },
    /// A token from the LLM (for streaming final answer)
    #[serde(rename = "token")]
    Token { token: String },
    /// v0.3.44: Internal comms dialogue line (fly-on-the-wall)
    #[serde(rename = "dialogue")]
    Dialogue {
        speaker: String,
        recipient: Option<String>,
        message: String,
        offset_ms: u64,
    },
    /// Validation warning (v0.0.889) - issues detected during streaming
    #[serde(rename = "validation")]
    Validation { warning: ValidationWarning },
    /// Final result with complete answer
    #[serde(rename = "done")]
    Done { result: AskResult },
    /// Error occurred
    #[serde(rename = "error")]
    Error { message: String },
}

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: RpcMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: String,
}

impl RpcRequest {
    pub fn new(method: RpcMethod, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: String,
}

impl RpcResponse {
    pub fn success(id: &str, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: id.to_string(),
        }
    }

    pub fn error(id: &str, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message: message.to_string(),
            }),
            id: id.to_string(),
        }
    }
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// v0.3.20: Reset modes per spec
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResetMode {
    /// Reset memory only (experiences, patterns, clusters)
    Memory,
    /// Reset config only (settings back to defaults)
    Config,
    /// Reset model preferences
    Models,
    /// Reset helper packages tracking
    Helpers,
    /// Reset everything (full factory reset)
    #[default]
    Everything,
}

impl ResetMode {
    /// Get description of what this mode resets
    pub fn description(&self) -> &'static str {
        match self {
            ResetMode::Memory => "memory (experiences, patterns, learned behaviors)",
            ResetMode::Config => "configuration (settings back to defaults)",
            ResetMode::Models => "model preferences (will re-detect on next start)",
            ResetMode::Helpers => "helper tracking (does not uninstall packages)",
            ResetMode::Everything => "everything (full factory reset)",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "memory" | "mem" => Some(ResetMode::Memory),
            "config" | "cfg" => Some(ResetMode::Config),
            "models" | "model" => Some(ResetMode::Models),
            "helpers" | "helper" | "deps" => Some(ResetMode::Helpers),
            "everything" | "all" | "full" => Some(ResetMode::Everything),
            _ => None,
        }
    }
}

/// v0.3.21: Parameters for reset command
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResetParams {
    /// Reset mode (defaults to Everything if not specified)
    #[serde(default)]
    pub mode: ResetMode,
    /// Skip backup (dangerous, default false)
    #[serde(default)]
    pub skip_backup: bool,
    /// Dry run - show what would be reset without actually resetting
    #[serde(default)]
    pub dry_run: bool,
}

impl ResetParams {
    /// Create params for a specific mode
    pub fn with_mode(mode: ResetMode) -> Self {
        Self {
            mode,
            skip_backup: false,
            dry_run: false,
        }
    }
}

// =============================================================================
// Assisted Operation Types (Phase 43)
// =============================================================================
//
// These types are wire-format equivalents of annad::assisted_ops types.
// They are used to serialize assisted operations over RPC.

/// An assisted operation returned from diagnosis (Phase 43).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistedOperationResult {
    /// Unique identifier for this operation
    pub operation_id: String,
    /// What problem was detected
    pub detected_problem: String,
    /// Human-readable explanation of the problem and proposed fix
    pub explanation: String,
    /// Ordered list of steps for the human to execute
    pub proposed_steps: Vec<ProposedStepResult>,
    /// How risky are these changes
    pub risk_level: RiskLevel,
    /// Sources of information (Arch Wiki URLs, man pages, etc.)
    pub sources: Vec<SourceResult>,
    /// Whether a reboot is required after all steps
    pub requires_reboot: bool,
    /// Diagnosis summary - what was observed
    pub diagnosis_summary: String,
}

/// A single proposed step (Phase 43).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedStepResult {
    /// Step number in sequence
    pub step_number: u32,
    /// Human-readable description of what this step does
    pub description: String,
    /// The exact command to run (verbatim, for copy/paste)
    pub exact_command: String,
    /// Why this step is necessary
    pub why: String,
    /// Whether this step can be undone
    pub reversible: bool,
    /// Command to reverse this step (if reversible)
    pub reverse_command: Option<String>,
    /// Execution safety classification
    pub safety: CommandSafety,
}

/// Safety classification for a proposed command (Phase 43).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommandSafety {
    /// Safe to run automatically - in HumanExecutionAdapter allowlist
    SafeAutomatic,
    /// Must be run manually by human - requires sudo or has risks
    #[default]
    ManualOnly,
}

/// Risk level for an assisted operation (Phase 43).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Changes are purely diagnostic or configuration
    Low,
    /// Changes affect system behavior but are reversible
    Medium,
    /// Changes may cause issues if done incorrectly
    High,
    /// Changes could break the system
    Critical,
}

/// A source of information (Phase 43).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResult {
    /// Type of source
    pub source_type: SourceType,
    /// Title or description
    pub title: String,
    /// URL or reference
    pub reference: String,
}

/// Type of information source (Phase 43).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Arch Wiki article
    ArchWiki,
    /// Man page
    ManPage,
    /// Upstream documentation
    Upstream,
    /// Kernel documentation
    Kernel,
    /// Forum or community post
    Community,
}
