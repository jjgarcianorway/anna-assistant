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

    /// Get sysadmin brain analysis (Beta.217b)
    /// Runs deterministic diagnostic rules and returns insights
    /// Citation: [archwiki:System_maintenance]
    BrainAnalysis,

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

    /// Get current empathy pulse (Phase 1.2)
    /// Citation: [archwiki:System_maintenance]
    EmpathyPulse,

    /// Simulate empathy evaluation for an action (Phase 1.2)
    /// Citation: [archwiki:System_maintenance]
    EmpathySimulate {
        /// Action to simulate
        action: String,
    },

    /// Get collective mind status (Phase 1.3)
    /// Citation: [archwiki:System_maintenance]
    CollectiveStatus,

    /// Get trust details for a peer (Phase 1.3)
    /// Citation: [archwiki:System_maintenance]
    CollectiveTrust {
        /// Peer ID to query
        peer_id: String,
    },

    /// Explain a consensus decision (Phase 1.3)
    /// Citation: [archwiki:System_maintenance]
    CollectiveExplain {
        /// Consensus ID to explain
        consensus_id: String,
    },

    /// Generate reflection report (Phase 1.4)
    /// Citation: [archwiki:System_maintenance]
    MirrorReflect,

    /// Get mirror audit summary (Phase 1.4)
    /// Citation: [archwiki:System_maintenance]
    MirrorAudit,

    /// Trigger bias remediation (Phase 1.4)
    /// Citation: [archwiki:System_maintenance]
    MirrorRepair,

    /// Generate temporal forecast (Phase 1.5)
    /// Citation: [archwiki:System_maintenance]
    ChronosForecast {
        /// Forecast window (hours)
        window_hours: u64,
    },

    /// Get chronos audit summary (Phase 1.5)
    /// Citation: [archwiki:System_maintenance]
    ChronosAudit,

    /// Align forecast parameters across network (Phase 1.5)
    /// Citation: [archwiki:System_maintenance]
    ChronosAlign,

    /// Run temporal forecast audit (Phase 1.6)
    /// Citation: [archwiki:System_maintenance]
    MirrorAuditForecast {
        /// Window hours for audit (default 24)
        window_hours: Option<u64>,
    },

    /// Generate temporal self-reflection with adaptive learning (Phase 1.6)
    /// Citation: [archwiki:System_maintenance]
    MirrorReflectTemporal {
        /// Window hours for reflection (default 24)
        window_hours: Option<u64>,
    },

    /// Get system profile for adaptive intelligence (Phase 3.0)
    /// Citation: [linux:proc][systemd:detect-virt][xdg:session]
    GetProfile,

    /// Get Historian 30-day system summary (Beta.53)
    GetHistorianSummary,

    /// Get telemetry snapshot for welcome reports (Beta.213)
    GetTelemetrySnapshot,

    /// Get reflection summary (6.7.0)
    /// Returns Anna's reflection on recent changes and system events
    GetReflection,

    /// Get system knowledge snapshot (6.12.0)
    /// Returns Anna's persistent memory of the system
    GetSystemKnowledge,
}

/// Response data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ResponseData {
    /// Status information
    Status(StatusData),

    /// System facts
    Facts(SystemFacts),

    /// Historian 30-day summary (Beta.53)
    HistorianSummary(crate::historian::SystemSummary),

    /// Telemetry snapshot for welcome reports (Beta.213)
    TelemetrySnapshot(crate::telemetry::TelemetrySnapshot),

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

    /// Available capabilities for current state (Phase 3.0: adaptive intelligence)
    /// Citation: [archwiki:system_maintenance]
    Capabilities(CapabilitiesData),

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

    /// Sysadmin brain analysis (Beta.217b)
    /// Citation: [archwiki:System_maintenance]
    BrainAnalysis(BrainAnalysisData),

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

    /// Empathy pulse (Phase 1.2)
    /// Citation: [archwiki:System_maintenance]
    EmpathyPulse(EmpathyPulseData),

    /// Empathy simulation result (Phase 1.2)
    /// Citation: [archwiki:System_maintenance]
    EmpathySimulation(EmpathySimulationData),

    /// Collective mind status (Phase 1.3)
    /// Citation: [archwiki:System_maintenance]
    CollectiveStatus(CollectiveStatusData),

    /// Collective peer trust details (Phase 1.3)
    /// Citation: [archwiki:System_maintenance]
    CollectiveTrust(CollectiveTrustData),

    /// Collective consensus explanation (Phase 1.3)
    /// Citation: [archwiki:System_maintenance]
    CollectiveExplanation(CollectiveExplanationData),

    /// Mirror reflection report (Phase 1.4)
    /// Citation: [archwiki:System_maintenance]
    MirrorReflection(MirrorReflectionData),

    /// Mirror audit summary (Phase 1.4)
    /// Citation: [archwiki:System_maintenance]
    MirrorAudit(MirrorAuditData),

    /// Mirror repair report (Phase 1.4)
    /// Citation: [archwiki:System_maintenance]
    MirrorRepair(MirrorRepairData),

    /// Chronos forecast result (Phase 1.5)
    /// Citation: [archwiki:System_maintenance]
    ChronosForecast(ChronosForecastData),

    /// Chronos audit summary (Phase 1.5)
    /// Citation: [archwiki:System_maintenance]
    ChronosAudit(ChronosAuditData),

    /// Chronos alignment result (Phase 1.5)
    /// Citation: [archwiki:System_maintenance]
    ChronosAlign(ChronosAlignData),

    /// Mirror forecast audit result (Phase 1.6)
    /// Citation: [archwiki:System_maintenance]
    MirrorAuditForecast(MirrorAuditTemporalData),

    /// Mirror temporal reflection result (Phase 1.6)
    /// Citation: [archwiki:System_maintenance]
    MirrorReflectTemporal(MirrorReflectTemporalData),

    /// System profile for adaptive intelligence (Phase 3.0)
    /// Citation: [linux:proc][systemd:detect-virt][xdg:session]
    Profile(ProfileData),

    /// Reflection summary (6.7.0)
    /// Anna's reflection on recent changes and system events
    Reflection(ReflectionSummaryData),

    /// System knowledge snapshot (6.12.0)
    /// Anna's persistent memory of the system
    SystemKnowledge(SystemKnowledgeData),
}

/// System knowledge data for RPC (6.12.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemKnowledgeData {
    pub wm_or_de: Option<String>,
    pub wallpaper_tool: Option<String>,
    pub wallpaper_config_files: Vec<String>,
    pub wallpaper_dirs: Vec<String>,
    pub services_failed: usize,
    pub services_masked: usize,
    pub services_disabled: usize,
    pub services_active: usize,
    pub top_processes: Vec<String>,
    pub total_ram_gib: u64,
    pub cpu_cores: u64,
    pub last_updated_secs: u64,
    // 6.12.1: Hardware profile
    pub hw_cpu_model: Option<String>,
    pub hw_cpu_physical_cores: Option<u64>,
    pub hw_cpu_logical_cores: Option<u64>,
    pub hw_gpu_model: Option<String>,
    pub hw_gpu_type: Option<String>,
    pub hw_sound_devices: Vec<String>,
    pub hw_total_ram_bytes: Option<u64>,
    pub hw_machine_model: Option<String>,
}

/// Reflection summary data for RPC (6.7.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionSummaryData {
    pub items: Vec<ReflectionItemData>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Reflection item data for RPC (6.7.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionItemData {
    pub severity: ReflectionSeverity,
    pub category: String,
    pub title: String,
    pub details: String,
    pub since_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Reflection severity for RPC (6.7.0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectionSeverity {
    Info,
    Notice,
    Warning,
    Critical,
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
    /// 6.20.0: Daemon health state (Healthy, Degraded, BrokenRecoverable, SafeMode)
    #[serde(default)]
    pub health_state: Option<String>,
    /// 6.20.0: Reason for degraded/safe mode state
    #[serde(default)]
    pub health_reason: Option<String>,
    /// 6.22.0: Anna's operational mode (NORMAL or SAFE)
    #[serde(default)]
    pub anna_mode: Option<String>,
    /// 6.22.0: Reason for safe mode if applicable
    #[serde(default)]
    pub anna_mode_reason: Option<String>,
    /// 6.22.0: Update status string
    #[serde(default)]
    pub update_status: Option<String>,
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

/// Capabilities response with adaptive intelligence (Phase 3.0)
/// Citation: [archwiki:system_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesData {
    /// Available commands for current state
    pub commands: Vec<CommandCapabilityData>,
    /// Monitoring mode (full/light/minimal)
    pub monitoring_mode: String,
    /// Rationale for monitoring mode selection
    pub monitoring_rationale: String,
    /// Whether system is resource-constrained
    pub is_constrained: bool,
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

/// Sysadmin brain analysis data (Beta.217b, Beta.271)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainAnalysisData {
    /// Report timestamp (ISO 8601)
    pub timestamp: String,
    /// List of diagnostic insights
    pub insights: Vec<DiagnosticInsightData>,
    /// Formatted output (canonical [SUMMARY]/[DETAILS]/[COMMANDS])
    pub formatted_output: String,
    /// Total critical issues count
    pub critical_count: usize,
    /// Total warning issues count
    pub warning_count: usize,
    /// Proactive correlated issues (Beta.271)
    #[serde(default)]
    pub proactive_issues: Vec<ProactiveIssueSummaryData>,
    /// Proactive health score 0-100 (Beta.273)
    #[serde(default = "default_health_score")]
    pub proactive_health_score: u8,
}

/// Default health score for backward compatibility
fn default_health_score() -> u8 {
    100
}

/// Proactive issue summary data (Beta.271)
/// User-safe representation of correlated issues from proactive engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveIssueSummaryData {
    /// Root cause type (user-safe string)
    pub root_cause: String,
    /// Severity: "critical", "warning", "info", "trend"
    pub severity: String,
    /// Human-readable summary
    pub summary: String,
    /// Optional rule ID for remediation mapping
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    /// Confidence (0.7-1.0)
    pub confidence: f32,
    /// First seen timestamp (ISO 8601)
    pub first_seen: String,
    /// Last seen timestamp (ISO 8601)
    pub last_seen: String,
    /// 6.2.0: Suggested fix based on Arch Wiki (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<SuggestedFixData>,
}

/// 6.2.0: Suggested fix with Arch Wiki sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFixData {
    pub description: String,
    pub steps: Vec<SuggestedStepData>,
    pub knowledge_sources: Vec<KnowledgeSourceData>,
}

/// Step in a suggested fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedStepData {
    pub kind: String, // "inspect" or "change"
    pub command: String,
    pub requires_confirmation: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_command: Option<String>,
}

/// Knowledge source reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSourceData {
    pub url: String,
    pub kind: String, // "ArchWiki" or "OfficialProjectDoc"
}

/// Diagnostic insight data (Beta.217b)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInsightData {
    /// Rule identifier
    pub rule_id: String,
    /// Severity: "info", "warning", "critical"
    pub severity: String,
    /// One-sentence summary
    pub summary: String,
    /// Detailed explanation
    pub details: String,
    /// Diagnostic/fix commands
    pub commands: Vec<String>,
    /// Documentation citations
    pub citations: Vec<String>,
    /// Evidence from telemetry
    pub evidence: String,
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

/// Empathy pulse data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyPulseData {
    /// Timestamp
    pub timestamp: String,
    /// Empathy index (0.0-1.0)
    pub empathy_index: f64,
    /// Strain index (0.0-1.0)
    pub strain_index: f64,
    /// Resonance map
    pub resonance_map: ResonanceMapData,
    /// Context summary
    pub context_summary: String,
    /// Recent perceptions
    pub recent_perceptions: Vec<PerceptionRecordData>,
}

/// Resonance map data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceMapData {
    /// User resonance (0.0-1.0)
    pub user_resonance: f64,
    /// System resonance (0.0-1.0)
    pub system_resonance: f64,
    /// Environment resonance (0.0-1.0)
    pub environment_resonance: f64,
    /// Recent adjustments
    pub recent_adjustments: Vec<ResonanceAdjustmentData>,
}

/// Resonance adjustment data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceAdjustmentData {
    /// Timestamp
    pub timestamp: String,
    /// Stakeholder
    pub stakeholder: String,
    /// Delta
    pub delta: f64,
    /// Reason
    pub reason: String,
}

/// Perception record data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionRecordData {
    /// Timestamp
    pub timestamp: String,
    /// Action
    pub action: String,
    /// Stakeholder impacts
    pub stakeholder_impacts: StakeholderImpactsData,
    /// Context factors
    pub context_factors: Vec<String>,
    /// Adaptation
    pub adaptation: Option<String>,
}

/// Stakeholder impacts data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderImpactsData {
    /// User impact
    pub user: StakeholderImpactData,
    /// System impact
    pub system: StakeholderImpactData,
    /// Environment impact
    pub environment: StakeholderImpactData,
}

/// Stakeholder impact data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderImpactData {
    /// Impact score (0.0-1.0)
    pub score: f64,
    /// Impact type
    pub impact_type: String,
    /// Reasoning
    pub reasoning: String,
}

/// Empathy simulation data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathySimulationData {
    /// Action simulated
    pub action: String,
    /// Evaluation
    pub evaluation: EmpathyEvaluationData,
    /// Reasoning explanation
    pub reasoning: String,
    /// Would action proceed?
    pub would_proceed: bool,
}

/// Empathy evaluation data (Phase 1.2)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathyEvaluationData {
    /// Should defer?
    pub should_defer: bool,
    /// Deferral reason
    pub deferral_reason: Option<String>,
    /// Stakeholder impacts
    pub stakeholder_impacts: StakeholderImpactsData,
    /// Context factors
    pub context_factors: Vec<String>,
    /// Recommended delay (seconds)
    pub recommended_delay: u64,
    /// Tone adaptation
    pub tone_adaptation: Option<String>,
}

/// Collective mind status data (Phase 1.3)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveStatusData {
    /// Is collective mind enabled?
    pub enabled: bool,
    /// This node's ID
    pub node_id: String,
    /// Number of connected peers
    pub connected_peers: usize,
    /// Total known peers
    pub total_peers: usize,
    /// Average network empathy (0.0-1.0)
    pub avg_network_empathy: f64,
    /// Average network strain (0.0-1.0)
    pub avg_network_strain: f64,
    /// Number of recent consensus decisions
    pub recent_decisions: usize,
    /// Overall network health (0.0-1.0)
    pub network_health: f64,
}

/// Collective peer trust data (Phase 1.3)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveTrustData {
    /// Peer ID
    pub peer_id: String,
    /// Peer name
    pub peer_name: String,
    /// Peer address
    pub peer_address: String,
    /// Overall trust score (0.0-1.0)
    pub overall_trust: f64,
    /// Honesty score (0.0-1.0)
    pub honesty: f64,
    /// Reliability score (0.0-1.0)
    pub reliability: f64,
    /// Ethical alignment score (0.0-1.0)
    pub ethical_alignment: f64,
    /// Total messages received
    pub messages_received: u64,
    /// Messages validated
    pub messages_validated: u64,
    /// Last interaction timestamp
    pub last_interaction: String,
    /// Connected?
    pub connected: bool,
}

/// Collective consensus explanation data (Phase 1.3)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveExplanationData {
    /// Consensus ID
    pub consensus_id: String,
    /// Action being decided
    pub action: String,
    /// Decision outcome
    pub decision: String,
    /// Timestamp
    pub timestamp: String,
    /// Vote breakdown
    pub votes: Vec<ConsensusVoteData>,
    /// Total participants
    pub total_participants: usize,
    /// Approval percentage
    pub approval_percentage: f64,
    /// Weighted approval percentage
    pub weighted_approval: f64,
    /// Reasoning
    pub reasoning: String,
}

/// Consensus vote data (Phase 1.3)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVoteData {
    /// Peer ID
    pub peer_id: String,
    /// Vote type
    pub vote: String,
    /// Vote weight
    pub weight: f64,
    /// Ethical score
    pub ethical_score: f64,
    /// Reasoning
    pub reasoning: String,
    /// Trust score at time of vote
    pub trust_score: f64,
}

/// Mirror reflection data (Phase 1.4)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorReflectionData {
    /// Reflection ID
    pub reflection_id: String,
    /// Timestamp
    pub timestamp: String,
    /// Period start
    pub period_start: String,
    /// Period end
    pub period_end: String,
    /// Self-coherence score (0.0-1.0)
    pub self_coherence: f64,
    /// Ethical decisions count
    pub ethical_decisions_count: usize,
    /// Conscience actions count
    pub conscience_actions_count: usize,
    /// Average empathy index
    pub avg_empathy_index: f64,
    /// Average strain index
    pub avg_strain_index: f64,
    /// Empathy trend
    pub empathy_trend: String,
    /// Adaptations count
    pub adaptations_count: usize,
    /// Self-identified biases
    pub self_identified_biases: Vec<String>,
}

/// Mirror audit data (Phase 1.4)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorAuditData {
    /// Is enabled?
    pub enabled: bool,
    /// Current coherence
    pub current_coherence: f64,
    /// Last reflection time
    pub last_reflection: Option<String>,
    /// Last consensus time
    pub last_consensus: Option<String>,
    /// Recent reflections count
    pub recent_reflections_count: usize,
    /// Received critiques count
    pub received_critiques_count: usize,
    /// Active remediations count
    pub active_remediations_count: usize,
    /// Network coherence (if in consensus)
    pub network_coherence: Option<f64>,
    /// Recent critiques summary
    pub recent_critiques: Vec<CritiqueSummary>,
}

/// Critique summary (Phase 1.4)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueSummary {
    /// Critic ID
    pub critic_id: String,
    /// Coherence assessment
    pub coherence_assessment: f64,
    /// Inconsistencies count
    pub inconsistencies_count: usize,
    /// Biases count
    pub biases_count: usize,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Mirror repair data (Phase 1.4)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorRepairData {
    /// Timestamp
    pub timestamp: String,
    /// Total remediations
    pub total_remediations: usize,
    /// Successful remediations
    pub successful_remediations: usize,
    /// Failed remediations
    pub failed_remediations: usize,
    /// Summary
    pub summary: String,
    /// Applied remediations
    pub applied_remediations: Vec<RemediationSummary>,
}

/// Remediation summary (Phase 1.4)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSummary {
    /// Description
    pub description: String,
    /// Type
    pub remediation_type: String,
    /// Expected impact
    pub expected_impact: String,
    /// Parameter adjustments
    pub parameter_adjustments: std::collections::HashMap<String, f64>,
}

/// Chronos forecast data (Phase 1.5)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronosForecastData {
    /// Forecast ID
    pub forecast_id: String,
    /// Generated timestamp
    pub generated_at: String,
    /// Forecast horizon (hours)
    pub horizon_hours: u64,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Final projected health score
    pub final_health: f64,
    /// Final projected empathy index
    pub final_empathy: f64,
    /// Final projected strain index
    pub final_strain: f64,
    /// Final projected network coherence
    pub final_coherence: f64,
    /// Temporal empathy index
    pub temporal_empathy_index: f64,
    /// Moral cost
    pub moral_cost: f64,
    /// Ethical trajectory
    pub ethical_trajectory: String,
    /// Stakeholder impacts
    pub stakeholder_impacts: std::collections::HashMap<String, f64>,
    /// Divergence warnings
    pub divergence_warnings: Vec<String>,
    /// Intervention recommendations
    pub recommendations: Vec<String>,
    /// Archive hash
    pub archive_hash: String,
}

/// Chronos audit data (Phase 1.5)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronosAuditData {
    /// Total archived forecasts
    pub total_archived: usize,
    /// Recent forecasts
    pub recent_forecasts: Vec<ForecastSummary>,
}

/// Forecast summary (Phase 1.5)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastSummary {
    /// Forecast ID
    pub forecast_id: String,
    /// Generated timestamp
    pub generated_at: String,
    /// Horizon (hours)
    pub horizon_hours: u64,
    /// Confidence
    pub confidence: f64,
    /// Warnings count
    pub warnings_count: usize,
    /// Moral cost
    pub moral_cost: f64,
}

/// Chronos alignment data (Phase 1.5)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronosAlignData {
    /// Alignment status
    pub status: String,
    /// Parameters aligned
    pub parameters_aligned: usize,
    /// Parameter changes
    pub parameter_changes: std::collections::HashMap<String, String>,
}

/// Mirror audit temporal data (Phase 1.6)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorAuditTemporalData {
    /// Total audits performed
    pub total_audits: usize,
    /// Last audit timestamp
    pub last_audit_at: Option<String>,
    /// Average temporal integrity score
    pub average_temporal_integrity: Option<f64>,
    /// Active bias findings
    pub active_biases: Vec<BiasFindingData>,
    /// Pending adjustment plans
    pub pending_adjustments: Vec<AdjustmentPlanData>,
}

/// Bias finding data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasFindingData {
    /// Type of bias
    pub kind: String,
    /// Confidence [0.0..1.0]
    pub confidence: f64,
    /// Supporting evidence
    pub evidence: String,
    /// Magnitude of bias
    pub magnitude: f64,
    /// Sample size
    pub sample_size: usize,
}

/// Adjustment plan data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentPlanData {
    /// Plan ID
    pub plan_id: String,
    /// Created timestamp
    pub created_at: String,
    /// Target subsystem
    pub target: String,
    /// Parameter adjustments
    pub adjustments: Vec<ParameterAdjustmentData>,
    /// Expected improvement [0.0..1.0]
    pub expected_improvement: f64,
    /// Rationale
    pub rationale: String,
}

/// Parameter adjustment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterAdjustmentData {
    /// Parameter name
    pub parameter: String,
    /// Current value (if known)
    pub current_value: Option<f64>,
    /// Recommended value
    pub recommended_value: f64,
    /// Justification
    pub reason: String,
}

/// Mirror reflect temporal data (Phase 1.6)
/// Citation: [archwiki:System_maintenance]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorReflectTemporalData {
    /// Reflection ID
    pub reflection_id: String,
    /// Generated timestamp
    pub generated_at: String,
    /// Window hours analyzed
    pub window_hours: u64,
    /// Temporal integrity score
    pub temporal_integrity_score: f64,
    /// Detected biases
    pub biases_detected: Vec<BiasFindingData>,
    /// Recommended adjustments
    pub recommended_adjustments: Option<AdjustmentPlanData>,
    /// Summary
    pub summary: String,
}

/// System profile data for adaptive intelligence (Phase 3.0)
/// Citation: [linux:proc][systemd:detect-virt][xdg:session]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    /// Total system RAM in MB
    pub total_memory_mb: u64,
    /// Available RAM in MB
    pub available_memory_mb: u64,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total disk space in GB
    pub total_disk_gb: u64,
    /// Available disk space in GB
    pub available_disk_gb: u64,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Virtualization type (none, vm, container, unknown)
    pub virtualization: String,
    /// Session type (desktop, headless, ssh, console, unknown)
    pub session_type: String,
    /// GPU present
    pub gpu_present: bool,
    /// GPU vendor (if present)
    pub gpu_vendor: Option<String>,
    /// GPU model (if present)
    pub gpu_model: Option<String>,
    /// Recommended monitoring mode (minimal, light, full)
    pub recommended_monitoring_mode: String,
    /// Human-readable rationale for monitoring mode selection
    pub monitoring_rationale: String,
    /// Whether system is resource-constrained
    pub is_constrained: bool,
    /// Timestamp when profile was collected
    pub timestamp: String,
}
