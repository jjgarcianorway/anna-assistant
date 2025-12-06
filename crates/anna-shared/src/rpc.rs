//! JSON-RPC 2.0 types for annad communication.
//! v0.0.73: Added GetDaemonInfo for version truth.

use crate::clarify_v2::ClarifyRequest;
use crate::recipe_feedback::FeedbackRequest;
use crate::reliability::ReliabilityExplanation;
use crate::trace::ExecutionTrace;
use crate::transcript::Transcript;
use crate::version::VersionInfo;
use serde::{Deserialize, Serialize};

/// RPC methods supported by annad
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RpcMethod {
    Status,
    Request,
    Reset,
    Uninstall,
    Autofix,
    Probe,
    /// Get progress events for current/last request
    Progress,
    /// Get per-team statistics (v0.0.27)
    Stats,
    /// Get comprehensive status snapshot (v0.0.29)
    StatusSnapshot,
    /// v0.0.73: Get daemon version info (for client/daemon version comparison)
    GetDaemonInfo,
    /// v0.0.95: Plan a config change (returns ChangePlan for user confirmation)
    PlanChange,
    /// v0.0.95: Apply a confirmed change plan
    ApplyChange,
    /// v0.0.95: Rollback a change using backup
    RollbackChange,
}

/// v0.0.73: Response from GetDaemonInfo RPC call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    /// Daemon version info
    pub version_info: VersionInfo,
    /// Daemon process ID
    pub pid: u32,
    /// Daemon uptime in seconds
    pub uptime_secs: u64,
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
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: String, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Parameters for the request method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestParams {
    pub prompt: String,
}

/// Parameters for the probe method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeParams {
    pub probe_type: ProbeType,
}

/// Types of probes that can be run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeType {
    TopMemory,
    TopCpu,
    DiskUsage,
    NetworkInterfaces,
}

/// v0.0.95: Parameters for PlanChange RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanChangeParams {
    /// Path to the config file
    pub config_path: String,
    /// Line to ensure exists
    pub line: String,
}

/// v0.0.95: Parameters for ApplyChange/RollbackChange RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeParams {
    /// The change plan to apply/rollback
    pub plan: crate::change::ChangePlan,
}

/// Runtime context injected into every LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContext {
    pub version: String,
    pub daemon_running: bool,
    pub capabilities: Capabilities,
    pub hardware: HardwareSummary,
    pub probes: std::collections::HashMap<String, String>,
}

/// Capability flags for the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub can_read_system_info: bool,
    pub can_run_probes: bool,
    pub can_modify_files: bool,
    pub can_install_packages: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            can_read_system_info: true,
            can_run_probes: true,
            can_modify_files: false,
            can_install_packages: false,
        }
    }
}

/// Hardware summary for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSummary {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub ram_gb: f64,
    pub gpu: Option<String>,
    pub gpu_vram_gb: Option<f64>,
}

/// Specialist domain for service desk routing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistDomain {
    #[default]
    System,
    Network,
    Storage,
    Security,
    Packages,
}

impl std::fmt::Display for SpecialistDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Network => write!(f, "network"),
            Self::Storage => write!(f, "storage"),
            Self::Security => write!(f, "security"),
            Self::Packages => write!(f, "packages"),
        }
    }
}

/// Intent classification from translator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueryIntent {
    #[default]
    Question,
    Request,
    Investigate,
}

impl std::fmt::Display for QueryIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Question => write!(f, "question"),
            Self::Request => write!(f, "request"),
            Self::Investigate => write!(f, "investigate"),
        }
    }
}

/// Translator ticket - structured output from LLM translator
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranslatorTicket {
    /// Query intent classification
    #[serde(default)]
    pub intent: QueryIntent,
    /// Target specialist domain
    #[serde(default)]
    pub domain: SpecialistDomain,
    /// Extracted entities (processes, services, mounts, etc.)
    #[serde(default)]
    pub entities: Vec<String>,
    /// Probe IDs needed from allowlist
    #[serde(default)]
    pub needs_probes: Vec<String>,
    /// Clarification question if query is ambiguous
    #[serde(default)]
    pub clarification_question: Option<String>,
    /// Translator confidence 0.0-1.0
    #[serde(default)]
    pub confidence: f32,
    /// v0.0.74: Answer contract defining what the answer should contain
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_contract: Option<crate::answer_contract::AnswerContract>,
}

/// Structured probe result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Command that was run
    pub command: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// First N lines of stdout
    pub stdout: String,
    /// First N lines of stderr
    pub stderr: String,
    /// Execution time in milliseconds
    pub timing_ms: u64,
}

/// Evidence block showing what data was used
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceBlock {
    /// Hardware snapshot fields used
    #[serde(default)]
    pub hardware_fields: Vec<String>,
    /// Probes that were executed
    #[serde(default)]
    pub probes_executed: Vec<ProbeResult>,
    /// Translator ticket that routed this query
    #[serde(default)]
    pub translator_ticket: TranslatorTicket,
    /// Last error if any (e.g., "timeout at translator")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_error: Option<String>,
}

/// Reliability scoring signals (all boolean for deterministic calculation)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReliabilitySignals {
    /// Translator confidence >= 0.7
    #[serde(default)]
    pub translator_confident: bool,
    /// All requested probes succeeded
    #[serde(default)]
    pub probe_coverage: bool,
    /// Answer references probe/hardware data
    #[serde(default)]
    pub answer_grounded: bool,
    /// No invented facts detected
    #[serde(default)]
    pub no_invention: bool,
    /// No clarification needed
    #[serde(default)]
    pub clarification_not_needed: bool,
}

impl ReliabilitySignals {
    /// Calculate score: 20 points per signal, max 100
    pub fn score(&self) -> u8 {
        let mut score: u8 = 0;
        if self.translator_confident {
            score += 20;
        }
        if self.probe_coverage {
            score += 20;
        }
        if self.answer_grounded {
            score += 20;
        }
        if self.no_invention {
            score += 20;
        }
        if self.clarification_not_needed {
            score += 20;
        }
        score
    }
}

/// Unified response from service desk pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeskResult {
    /// Unique request ID for tracking
    pub request_id: String,
    /// The LLM's answer text
    pub answer: String,
    /// Reliability score 0-100 (deterministic from signals)
    pub reliability_score: u8,
    /// Reliability scoring signals
    pub reliability_signals: ReliabilitySignals,
    /// TRUST: Structured explanation when score < 80 (None otherwise)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reliability_explanation: Option<ReliabilityExplanation>,
    /// Which specialist handled this
    pub domain: SpecialistDomain,
    /// Evidence block showing data sources
    pub evidence: EvidenceBlock,
    /// Whether clarification is needed
    pub needs_clarification: bool,
    /// Question to ask if clarification needed (legacy)
    pub clarification_question: Option<String>,
    /// Full clarification request with options (v0.0.47+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarification_request: Option<ClarifyRequest>,
    /// Full transcript of pipeline events
    pub transcript: Transcript,
    /// TRACE: Execution trace showing stages and paths (v0.0.23+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_trace: Option<ExecutionTrace>,
    /// v0.0.96: Proposed config change requiring user confirmation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposed_change: Option<crate::change::ChangePlan>,
    /// v0.0.103: Anna asks for feedback when uncertain about recipe answer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_request: Option<FeedbackRequest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_serialization() {
        let req = RpcRequest::new(RpcMethod::Status, None);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"method\":\"status\""));
    }

    #[test]
    fn test_rpc_response_success() {
        let resp = RpcResponse::success("test-id".to_string(), serde_json::json!({"status": "ok"}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_rpc_response_error() {
        let resp = RpcResponse::error("test-id".to_string(), -32600, "Invalid request".to_string());
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }
}
