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
    RollbackAction { advice_id: String, dry_run: bool },

    /// Rollback last N actions (Beta.91)
    RollbackLast { count: usize, dry_run: bool },

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

    /// Repair failed probes (Phase 0.7)
    /// Citation: [archwiki:System_maintenance]
    RepairProbe {
        /// Specific probe to repair, or "all" for all failed probes
        probe: String,
        /// Dry-run mode: simulate repair without executing
        dry_run: bool,
    },

    /// Perform guided Arch Linux installation (Phase 0.8)
    /// Citation: [archwiki:Installation_guide]
    PerformInstall {
        /// Installation configuration
        config: InstallConfigData,
        /// Dry-run mode: simulate without executing
        dry_run: bool,
    },

    /// Check system health (Phase 0.9)
    /// Citation: [archwiki:System_maintenance]
    SystemHealth,

    /// Perform system update (Phase 0.9)
    /// Citation: [archwiki:System_maintenance#Upgrading_the_system]
    SystemUpdate {
        /// Dry-run mode: simulate without executing
        dry_run: bool,
    },

    /// Perform system audit (Phase 0.9)
    /// Citation: [archwiki:Security]
    SystemAudit,

    /// Get sentinel status (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    SentinelStatus,

    /// Get sentinel metrics (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    SentinelMetrics,

    /// Get sentinel configuration (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    SentinelGetConfig,

    /// Set sentinel configuration (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    SentinelSetConfig {
        /// Configuration JSON
        config: SentinelConfigData,
    },

    /// Get pending conscience actions requiring review (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceReview,

    /// Get detailed explanation for a conscience decision (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceExplain {
        /// Decision ID to explain
        decision_id: String,
    },

    /// Approve a flagged conscience action (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceApprove {
        /// Decision ID to approve
        decision_id: String,
    },

    /// Reject a flagged conscience action (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceReject {
        /// Decision ID to reject
        decision_id: String,
    },

    /// Trigger manual conscience introspection (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceIntrospect,
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
    StreamEnd { success: bool, message: String },

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
    HealthProbe { ok: bool, version: String },

    /// Health run results (Phase 0.5)
    /// Citation: [archwiki:System_maintenance]
    HealthRun(HealthRunData),

    /// Health summary (Phase 0.5)
    /// Citation: [archwiki:System_maintenance]
    HealthSummary(HealthSummaryData),

    /// Recovery plans (Phase 0.5)
    /// Citation: [archwiki:General_troubleshooting]
    RecoveryPlans(RecoveryPlansData),

    /// Repair result (Phase 0.7)
    /// Citation: [archwiki:System_maintenance]
    RepairResult(RepairResultData),

    /// Installation result (Phase 0.8)
    /// Citation: [archwiki:Installation_guide]
    InstallResult(InstallResultData),

    /// System health report (Phase 0.9)
    /// Citation: [archwiki:System_maintenance]
    HealthReport(HealthReportData),

    /// System update report (Phase 0.9)
    /// Citation: [archwiki:System_maintenance#Upgrading_the_system]
    UpdateReport(UpdateReportData),

    /// System audit report (Phase 0.9)
    /// Citation: [archwiki:Security]
    AuditReport(AuditReportData),

    /// Sentinel status (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    SentinelStatus(SentinelStatusData),

    /// Sentinel metrics (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    SentinelMetrics(SentinelMetricsData),

    /// Sentinel configuration (Phase 1.0)
    /// Citation: [archwiki:System_maintenance]
    SentinelConfig(SentinelConfigData),

    /// Conscience pending actions (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConsciencePending(ConsciencePendingData),

    /// Conscience decision explanation (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceDecision(ConscienceDecisionData),

    /// Conscience introspection report (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceIntrospection(ConscienceIntrospectionData),

    /// Conscience action result (Phase 1.1)
    /// Citation: [archwiki:System_maintenance]
    ConscienceActionResult(String),
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

/// Repair result data (Phase 0.7)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResultData {
    /// Was this a dry-run (simulation)?
    pub dry_run: bool,
    /// Current system state after repair
    pub state: String,
    /// Individual repair actions performed
    pub repairs: Vec<RepairAction>,
    /// Overall success status
    pub success: bool,
    /// Human-readable summary message
    pub message: String,
    /// Wiki citation
    pub citation: String,
}

/// Individual repair action (Phase 0.7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAction {
    /// Probe that was repaired (e.g., "disk-space", "pacman-db")
    pub probe: String,
    /// Action taken or simulated
    pub action: String,
    /// Command that was executed (if applicable)
    pub command: Option<String>,
    /// Exit code (if command was executed)
    pub exit_code: Option<i32>,
    /// Success status for this specific repair
    pub success: bool,
    /// Details or error message
    pub details: String,
    /// Arch Wiki citation for this repair action
    pub citation: String,
}

/// Installation configuration data (Phase 0.8)
/// Citation: [archwiki:Installation_guide]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfigData {
    /// Disk setup mode
    pub disk_setup: DiskSetupData,
    /// Bootloader type
    pub bootloader: String,
    /// System hostname
    pub hostname: String,
    /// Username to create
    pub username: String,
    /// Timezone (e.g., "America/New_York")
    pub timezone: String,
    /// Locale (e.g., "en_US.UTF-8")
    pub locale: String,
    /// Additional packages to install
    pub extra_packages: Vec<String>,
}

/// Disk setup configuration (Phase 0.8)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum DiskSetupData {
    #[serde(rename = "manual")]
    Manual {
        root_partition: String,
        boot_partition: String,
        swap_partition: Option<String>,
    },
    #[serde(rename = "auto_btrfs")]
    AutoBtrfs {
        target_disk: String,
        create_swap: bool,
        swap_size_gb: u32,
    },
}

/// Installation result data (Phase 0.8)
/// Citation: [archwiki:Installation_guide]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResultData {
    /// Was this a dry-run (simulation)?
    pub dry_run: bool,
    /// Overall success status
    pub success: bool,
    /// Individual installation steps
    pub steps: Vec<InstallStepData>,
    /// Summary message
    pub message: String,
    /// Wiki citation
    pub citation: String,
}

/// Individual installation step (Phase 0.8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallStepData {
    /// Step name (e.g., "disk_setup", "base_system")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Success status for this step
    pub success: bool,
    /// Detailed output or error message
    pub details: String,
    /// Arch Wiki citation for this step
    pub citation: String,
}

/// System health report data (Phase 0.9)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReportData {
    /// Report timestamp (ISO 8601)
    pub timestamp: String,
    /// Overall health status (healthy, degraded, critical)
    pub overall_status: String,
    /// Service statuses
    pub services: Vec<ServiceStatusData>,
    /// Package statuses
    pub packages: Vec<PackageStatusData>,
    /// Log issues
    pub log_issues: Vec<LogIssueData>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Summary message
    pub message: String,
    /// Wiki citation
    pub citation: String,
}

/// Service status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatusData {
    pub name: String,
    pub state: String,
    pub active: bool,
    pub enabled: bool,
}

/// Package status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStatusData {
    pub name: String,
    pub status: String,
    pub version: String,
    pub update_available: bool,
}

/// Log issue data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIssueData {
    pub timestamp: String,
    pub severity: String,
    pub message: String,
    pub unit: String,
}

/// System update report data (Phase 0.9)
/// Citation: [archwiki:System_maintenance#Upgrading_the_system]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportData {
    /// Report timestamp (ISO 8601)
    pub timestamp: String,
    /// Was this a dry-run?
    pub dry_run: bool,
    /// Update success
    pub success: bool,
    /// Packages updated
    pub packages_updated: Vec<PackageUpdateData>,
    /// Services restarted
    pub services_restarted: Vec<String>,
    /// Snapshot path (if created)
    pub snapshot_path: Option<String>,
    /// Summary message
    pub message: String,
    /// Wiki citation
    pub citation: String,
}

/// Package update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpdateData {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub size_change: i64,
}

/// System audit report data (Phase 0.9)
/// Citation: [archwiki:Security]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReportData {
    /// Report timestamp (ISO 8601)
    pub timestamp: String,
    /// Overall compliance status
    pub compliant: bool,
    /// Integrity check results
    pub integrity: Vec<IntegrityStatusData>,
    /// Security findings
    pub security_findings: Vec<SecurityFindingData>,
    /// Configuration issues
    pub config_issues: Vec<ConfigIssueData>,
    /// Summary message
    pub message: String,
    /// Wiki citation
    pub citation: String,
}

/// Integrity check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityStatusData {
    pub component: String,
    pub check_type: String,
    pub passed: bool,
    pub details: String,
}

/// Security finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFindingData {
    pub severity: String,
    pub description: String,
    pub recommendation: String,
    pub reference: String,
}

/// Configuration issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigIssueData {
    pub file: String,
    pub issue: String,
    pub expected: String,
    pub actual: String,
}

/// Sentinel status data (Phase 1.0)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelStatusData {
    /// Sentinel enabled
    pub enabled: bool,
    /// Autonomous mode active
    pub autonomous_mode: bool,
    /// Uptime (seconds)
    pub uptime_seconds: u64,
    /// Current system state
    pub system_state: String,
    /// Last health status
    pub last_health_status: String,
    /// Last health check timestamp
    pub last_health_check: Option<String>,
    /// Last update scan timestamp
    pub last_update_scan: Option<String>,
    /// Last audit timestamp
    pub last_audit: Option<String>,
    /// Error rate (errors per hour)
    pub error_rate: f64,
    /// System drift index (0.0-1.0)
    pub drift_index: f64,
}

/// Sentinel metrics data (Phase 1.0)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelMetricsData {
    /// Uptime (seconds)
    pub uptime_seconds: u64,
    /// Total events processed
    pub total_events: u64,
    /// Automated actions taken
    pub automated_actions: u64,
    /// Manual commands received
    pub manual_commands: u64,
    /// Health checks performed
    pub health_checks: u64,
    /// Update scans performed
    pub update_scans: u64,
    /// Audits performed
    pub audits: u64,
    /// Current health status
    pub current_health: String,
    /// Error rate (errors per hour)
    pub error_rate: f64,
    /// System drift index
    pub drift_index: f64,
}

/// Sentinel configuration data (Phase 1.0)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelConfigData {
    /// Enable autonomous operations
    pub autonomous_mode: bool,
    /// Health check interval (seconds)
    pub health_check_interval: u64,
    /// Update scan interval (seconds)
    pub update_scan_interval: u64,
    /// Audit interval (seconds)
    pub audit_interval: u64,
    /// Auto-repair failed services
    pub auto_repair_services: bool,
    /// Auto-update packages
    pub auto_update: bool,
    /// Maximum packages to auto-update without confirmation
    pub auto_update_threshold: u32,
    /// Enable adaptive scheduling
    pub adaptive_scheduling: bool,
}

/// Conscience pending actions data (Phase 1.1)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciencePendingData {
    /// Pending actions requiring review
    pub pending_actions: Vec<PendingActionData>,
}

/// Pending action data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingActionData {
    /// Action ID
    pub id: String,
    /// Timestamp
    pub timestamp: String,
    /// Action description
    pub action: String,
    /// Flag reason
    pub flag_reason: String,
    /// Uncertainty score
    pub uncertainty: f64,
    /// Ethical score
    pub ethical_score: f64,
    /// Weakest dimension
    pub weakest_dimension: String,
}

/// Conscience decision data (Phase 1.1)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConscienceDecisionData {
    /// Decision ID
    pub id: String,
    /// Timestamp
    pub timestamp: String,
    /// Action description
    pub action: String,
    /// Outcome
    pub outcome: String,
    /// Ethical score
    pub ethical_score: f64,
    /// Safety score
    pub safety: f64,
    /// Privacy score
    pub privacy: f64,
    /// Integrity score
    pub integrity: f64,
    /// Autonomy score
    pub autonomy: f64,
    /// Confidence
    pub confidence: f64,
    /// Reasoning tree (formatted text)
    pub reasoning: String,
    /// Rollback plan available
    pub has_rollback_plan: bool,
}

/// Conscience introspection data (Phase 1.1)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConscienceIntrospectionData {
    /// Report timestamp
    pub timestamp: String,
    /// Period analyzed
    pub period: String,
    /// Decisions reviewed
    pub decisions_reviewed: u64,
    /// Approved count
    pub approved_count: u64,
    /// Rejected count
    pub rejected_count: u64,
    /// Flagged count
    pub flagged_count: u64,
    /// Average ethical score
    pub avg_ethical_score: f64,
    /// Average confidence
    pub avg_confidence: f64,
    /// Violations detected
    pub violations_count: u64,
    /// Recommendations
    pub recommendations: Vec<String>,
}
