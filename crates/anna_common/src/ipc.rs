//! IPC protocol definitions for Anna Assistant
//!
//! Defines message types and communication protocol between daemon and client.

use crate::types::{Advice, SystemFacts};
use serde::{Deserialize, Serialize};

/// IPC Request from client to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: Method,
}

/// IPC Response from daemon to client
/// Phase 0.2b: Added version field for API versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    pub result: Result<ResponseData, String>,
    /// API version (added in Phase 0.2b)
    #[serde(default = "default_api_version")]
    pub version: String,
}

fn default_api_version() -> String {
    "1.0.0".to_string()
}

/// Request methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Method {
    /// Get daemon status
    Status,

    /// Get system facts
    GetFacts,

    /// Get recommendations
    GetAdvice,

    /// Get recommendations with user context (for multi-user systems)
    GetAdviceWithContext {
        username: String,
        desktop_env: Option<String>,
        shell: String,
        display_server: Option<String>,
    },

    /// Apply an action by advice ID
    ApplyAction {
        advice_id: String,
        dry_run: bool,
        stream: bool, // Enable live output streaming
    },

    /// Get configuration
    GetConfig,

    /// Set configuration value
    SetConfig { key: String, value: String },

    /// Refresh system facts and advice
    Refresh,

    /// Update Arch Wiki cache
    UpdateWikiCache,

    /// Ping daemon (health check)
    Ping,

    /// Check for Anna updates (delegated to daemon)
    CheckUpdate,

    /// Perform Anna update (delegated to daemon)
    /// This allows the daemon (running as root) to handle the update
    /// without requiring sudo from the user
    PerformUpdate {
        /// Skip download and just restart with current binaries
        restart_only: bool,
    },

    /// List rollbackable actions (Beta.91)
    ListRollbackable,

    /// Rollback a specific action by advice ID (Beta.91)
    RollbackAction {
        advice_id: String,
        dry_run: bool,
    },

    /// Rollback last N actions (Beta.91)
    RollbackLast {
        count: usize,
        dry_run: bool,
    },

    /// Get system state detection (Phase 0.2b)
    /// Citation: [archwiki:system_maintenance]
    GetState,

    /// Get available capabilities for current state (Phase 0.2b)
    /// Citation: [archwiki:system_maintenance]
    GetCapabilities,

    /// Health probe with version (Phase 0.2b)
    /// Citation: [archwiki:system_maintenance]
    HealthProbe,

    /// Run health probes (Phase 0.5)
    /// Citation: [archwiki:System_maintenance]
    HealthRun {
        timeout_ms: u64,
        probes: Vec<String>,
    },

    /// Get health summary from last run (Phase 0.5)
    /// Citation: [archwiki:System_maintenance]
    HealthSummary,

    /// List available recovery plans (Phase 0.5)
    /// Citation: [archwiki:General_troubleshooting]
    RecoveryPlans,
}

/// Response data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ResponseData {
    /// Status information
    Status(StatusData),

    /// System facts
    Facts(SystemFacts),

    /// List of advice
    Advice(Vec<Advice>),

    /// Action result
    ActionResult { success: bool, message: String },

    /// Configuration data
    Config(ConfigData),

    /// Simple success/pong
    Ok,

    /// Streaming chunk (for live command output)
    StreamChunk {
        chunk_type: StreamChunkType,
        data: String,
    },

    /// Stream end marker
    StreamEnd {
        success: bool,
        message: String,
    },

    /// Update check result
    UpdateCheck {
        current_version: String,
        latest_version: String,
        is_update_available: bool,
        download_url: Option<String>,
        release_notes: Option<String>,
    },

    /// Update result
    UpdateResult {
        success: bool,
        message: String,
        old_version: String,
        new_version: String,
    },

    /// List of rollbackable actions (Beta.91)
    RollbackableActions(Vec<RollbackableAction>),

    /// Rollback result (Beta.91)
    RollbackResult {
        success: bool,
        message: String,
        actions_reversed: Vec<String>, // List of advice IDs that were rolled back
    },

    /// State detection result (Phase 0.2b)
    /// Citation: [archwiki:system_maintenance]
    StateDetection(StateDetectionData),

    /// Available capabilities for current state (Phase 0.2b)
    /// Citation: [archwiki:system_maintenance]
    Capabilities(Vec<CommandCapabilityData>),

    /// Health probe result (Phase 0.2b)
    HealthProbe {
        ok: bool,
        version: String,
    },

    /// Health run results (Phase 0.5)
    /// Citation: [archwiki:System_maintenance]
    HealthRun(HealthRunData),

    /// Health summary (Phase 0.5)
    /// Citation: [archwiki:System_maintenance]
    HealthSummary(HealthSummaryData),

    /// Recovery plans (Phase 0.5)
    /// Citation: [archwiki:General_troubleshooting]
    RecoveryPlans(RecoveryPlansData),
}

/// Type of streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamChunkType {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// Status update
    Status,
}

/// Rollbackable action information (Beta.91)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackableAction {
    /// Advice ID that was applied
    pub advice_id: String,
    /// Human-readable title
    pub title: String,
    /// When it was executed (ISO 8601 timestamp)
    pub executed_at: String,
    /// Original command that was executed
    pub command: String,
    /// Generated rollback command
    pub rollback_command: Option<String>,
    /// Whether this action can be rolled back
    pub can_rollback: bool,
    /// Reason why rollback is not available (if applicable)
    pub rollback_unavailable_reason: Option<String>,
}

/// Daemon status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    pub version: String,
    pub uptime_seconds: u64,
    pub last_telemetry_check: String,
    pub pending_recommendations: usize,
}

/// Configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub autonomy_tier: u8,
    pub auto_update_check: bool,
    pub wiki_cache_path: String,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            autonomy_tier: 0, // AdviseOnly
            auto_update_check: true,
            wiki_cache_path: "~/.local/share/anna/wiki".to_string(),
        }
    }
}

/// State detection result (Phase 0.2b)
/// Citation: [archwiki:system_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDetectionData {
    /// Detected state (iso_live, recovery_candidate, post_install_minimal, configured, degraded, unknown)
    pub state: String,
    /// When detection occurred (ISO 8601 timestamp)
    pub detected_at: String,
    /// Additional detection metadata
    pub details: StateDetailsData,
    /// Wiki citation for detection logic
    pub citation: String,
}

/// State detection metadata (Phase 0.2b)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDetailsData {
    /// Running under UEFI (vs BIOS)
    pub uefi: bool,
    /// Detected block devices
    pub disks: Vec<String>,
    /// Network connectivity status
    pub network: NetworkStatusData,
    /// Anna state file present
    pub state_file_present: bool,
    /// Health check passed (if applicable)
    pub health_ok: Option<bool>,
}

/// Network connectivity metadata (Phase 0.2b)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatusData {
    /// Has active network interface
    pub has_interface: bool,
    /// Has default route
    pub has_route: bool,
    /// Can resolve DNS
    pub can_resolve: bool,
}

/// Command capability definition (Phase 0.2b)
/// Citation: [archwiki:system_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCapabilityData {
    /// Command name (e.g., "install", "update")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Version when command was introduced
    pub since: String,
    /// Arch Wiki citation for this command
    pub citation: String,
    /// Whether command requires root privileges
    pub requires_root: bool,
}

/// Health run result (Phase 0.5)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRunData {
    /// Current system state
    pub state: String,
    /// Summary counts
    pub summary: HealthSummaryCount,
    /// Individual probe results
    pub results: Vec<HealthProbeResult>,
    /// Wiki citation
    pub citation: String,
}

/// Health summary count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummaryCount {
    pub ok: usize,
    pub warn: usize,
    pub fail: usize,
}

/// Individual health probe result (Phase 0.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthProbeResult {
    /// Probe name
    pub probe: String,
    /// Status: ok, warn, fail
    pub status: String,
    /// Detailed probe data
    pub details: serde_json::Value,
    /// Arch Wiki citation for this probe
    pub citation: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp (ISO 8601)
    pub ts: String,
}

/// Health summary (Phase 0.5)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummaryData {
    /// Current system state
    pub state: String,
    /// Summary counts
    pub summary: HealthSummaryCount,
    /// Last run timestamp (ISO 8601)
    pub last_run_ts: String,
    /// List of probes with alerts
    pub alerts: Vec<String>,
    /// Wiki citation
    pub citation: String,
}

/// Recovery plans list (Phase 0.5)
/// Citation: [archwiki:General_troubleshooting]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlansData {
    /// Available recovery plans
    pub plans: Vec<RecoveryPlanItem>,
    /// Wiki citation
    pub citation: String,
}

/// Individual recovery plan (Phase 0.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlanItem {
    /// Plan ID (e.g., "bootloader", "initramfs")
    pub id: String,
    /// Human-readable description
    pub desc: String,
    /// Arch Wiki citation for this plan
    pub citation: String,
}
