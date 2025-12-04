//! Read-Only Tool Catalog v0.0.48
//!
//! v0.0.48: knowledge_search tool for local learned recipe retrieval
//! v0.0.47: File evidence tools for mutation support (file_stat, file_preview, file_hash, path_policy_check)
//! v0.0.46: Domain-specific evidence tools to prevent generic summary answers
//!
//! Safe, allowlisted tools that annad can execute for evidence gathering.
//! Each tool returns structured data plus a human-readable summary.
//!
//! Security model:
//! - All tools are read-only (no mutations)
//! - Strict allowlist enforcement
//! - Structured output with Evidence IDs
//! - Human-readable summaries for debug transcript

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Security classification for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolSecurity {
    /// Safe read-only operation
    ReadOnly,
    /// Reads potentially sensitive data (logs, configs)
    SensitiveRead,
}

/// Latency hint for UI/planning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyHint {
    /// < 10ms typical
    Fast,
    /// 10-100ms typical
    Medium,
    /// > 100ms typical
    Slow,
}

/// Tool definition in the catalog
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: &'static [(&'static str, &'static str, bool)], // (name, type, required)
    pub security: ToolSecurity,
    pub latency: LatencyHint,
    /// Natural language description for the transcript
    pub human_request: &'static str,
}

/// Result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool that was executed
    pub tool_name: String,
    /// Evidence ID (E1, E2, ...)
    pub evidence_id: String,
    /// Structured data (JSON-serializable)
    pub data: serde_json::Value,
    /// Human-readable one-line summary
    pub human_summary: String,
    /// Whether tool execution succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution timestamp
    pub timestamp: u64,
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// The read-only tool catalog
pub struct ToolCatalog {
    tools: HashMap<&'static str, ToolDef>,
}

impl Default for ToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCatalog {
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // status_snapshot - current daemon and system status
        tools.insert("status_snapshot", ToolDef {
            name: "status_snapshot",
            description: "Returns the current daemon status snapshot including uptime, LLM status, and system health",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check the current system and daemon status",
        });

        // sw_snapshot_summary - software overview
        tools.insert("sw_snapshot_summary", ToolDef {
            name: "sw_snapshot_summary",
            description: "Returns a summary of installed packages, commands, and services from the latest software snapshot",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get a summary of installed software, commands, and services",
        });

        // hw_snapshot_summary - hardware overview
        tools.insert("hw_snapshot_summary", ToolDef {
            name: "hw_snapshot_summary",
            description: "Returns a summary of hardware including CPU, memory, storage, GPU, and network from the latest hardware snapshot",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get a summary of the system hardware",
        });

        // recent_installs - packages installed recently
        tools.insert("recent_installs", ToolDef {
            name: "recent_installs",
            description: "Returns packages installed in the last N days from pacman log",
            parameters: &[("days", "integer", false)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "check what packages were installed recently",
        });

        // journal_warnings - recent log warnings
        tools.insert("journal_warnings", ToolDef {
            name: "journal_warnings",
            description: "Returns warnings and errors from journalctl for a specific service or system-wide",
            parameters: &[
                ("service", "string", false),
                ("minutes", "integer", false),
            ],
            security: ToolSecurity::SensitiveRead,
            latency: LatencyHint::Medium,
            human_request: "check the system journal for warnings and errors",
        });

        // boot_time_trend - boot performance over time
        tools.insert("boot_time_trend", ToolDef {
            name: "boot_time_trend",
            description: "Returns boot time trends over the last N days",
            parameters: &[("days", "integer", false)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "analyze boot time trends",
        });

        // top_resource_processes - high resource usage processes
        tools.insert("top_resource_processes", ToolDef {
            name: "top_resource_processes",
            description: "Returns processes with highest CPU/memory usage in the specified time window",
            parameters: &[("window_minutes", "integer", false)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "identify processes using the most resources",
        });

        // package_info - details about a specific package
        tools.insert("package_info", ToolDef {
            name: "package_info",
            description: "Returns detailed information about a specific installed package",
            parameters: &[("name", "string", true)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "look up information about a specific package",
        });

        // service_status - status of a specific service
        tools.insert("service_status", ToolDef {
            name: "service_status",
            description: "Returns the status of a specific systemd service",
            parameters: &[("name", "string", true)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check the status of a specific service",
        });

        // disk_usage - disk space information
        tools.insert("disk_usage", ToolDef {
            name: "disk_usage",
            description: "Returns disk usage information for all mounted filesystems",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check disk space usage",
        });

        // v0.0.12: Anomaly detection tools

        // active_alerts - current alert status
        tools.insert("active_alerts", ToolDef {
            name: "active_alerts",
            description: "Returns active alerts from the anomaly detection engine with severity and evidence IDs",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check for any active system alerts or anomalies",
        });

        // what_changed - system changes correlation
        tools.insert("what_changed", ToolDef {
            name: "what_changed",
            description: "Returns packages installed/removed, services enabled/disabled, and config changes in the specified time window",
            parameters: &[("days", "integer", false)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "analyze what has changed on the system recently",
        });

        // slowness_hypotheses - slowness analysis
        tools.insert("slowness_hypotheses", ToolDef {
            name: "slowness_hypotheses",
            description: "Analyzes potential causes of system slowness by combining what_changed, anomaly signals, and resource usage into ranked hypotheses with evidence citations",
            parameters: &[("days", "integer", false)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Slow,
            human_request: "analyze potential causes of system slowness",
        });

        // v0.0.19: Knowledge search tools

        // knowledge_search - search local documentation
        tools.insert("knowledge_search", ToolDef {
            name: "knowledge_search",
            description: "Searches local knowledge packs (man pages, package docs, user notes) for relevant information. Returns excerpts with Evidence IDs for citation.",
            parameters: &[("query", "string", true), ("top_k", "integer", false)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "search local documentation for information",
        });

        // knowledge_stats - knowledge pack statistics
        tools.insert("knowledge_stats", ToolDef {
            name: "knowledge_stats",
            description: "Returns statistics about indexed knowledge packs including document counts, index size, and last update time",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check knowledge pack statistics",
        });

        // v0.0.20: Ask Me Anything tools

        // answer_context - environment context for answering
        tools.insert("answer_context", ToolDef {
            name: "answer_context",
            description: "Returns context for answering: target user, distro, relevant components, available knowledge packs",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "gather answer context (user, distro, knowledge packs)",
        });

        // source_plan - which sources will be queried
        tools.insert("source_plan", ToolDef {
            name: "source_plan",
            description: "Plans which sources to query based on question type. Returns knowledge query and system tools to use.",
            parameters: &[("request", "string", true)],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "plan source mix for this question",
        });

        // qa_stats - Q&A statistics for today
        tools.insert("qa_stats", ToolDef {
            name: "qa_stats",
            description: "Returns today's Q&A statistics: answer count, average reliability, top source types",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check today's Q&A statistics",
        });

        // v0.0.22: Reliability Engineering - self diagnostics
        tools.insert("self_diagnostics", ToolDef {
            name: "self_diagnostics",
            description: "Generates a comprehensive self-diagnostics report: install review, update state, model readiness, policy status, storage, error budgets, recent errors (redacted), and active alerts. Suitable for pasting into an issue report.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "generate self-diagnostics report",
        });

        tools.insert("metrics_summary", ToolDef {
            name: "metrics_summary",
            description: "Returns reliability metrics summary: request/tool/mutation success rates, LLM timeout rates, cache hit rates, and latency percentiles (p50/p95)",
            parameters: &[
                ("days", "number", false), // default: 1
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get reliability metrics summary",
        });

        tools.insert("error_budgets", ToolDef {
            name: "error_budgets",
            description: "Returns error budget status: current burn rate vs thresholds for requests, tools, mutations, and LLM timeouts",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check error budget status",
        });

        // v0.0.33: Case file retrieval tools
        tools.insert("last_case_summary", ToolDef {
            name: "last_case_summary",
            description: "Returns the summary of the most recent case file (what user asked, outcome, reliability score, evidence used)",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "show the last case summary",
        });

        tools.insert("last_failure_summary", ToolDef {
            name: "last_failure_summary",
            description: "Returns the summary and transcript of the most recent failed case, useful for debugging",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "show what happened in the last failure",
        });

        tools.insert("list_today_cases", ToolDef {
            name: "list_today_cases",
            description: "Lists all case files from today with timestamps, outcomes, and reliability scores",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "list today's cases",
        });

        tools.insert("list_recent_cases", ToolDef {
            name: "list_recent_cases",
            description: "Lists recent case files with timestamps, outcomes, and reliability scores",
            parameters: &[
                ("limit", "number", false), // default: 10
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "list recent cases",
        });

        // v0.0.45: Direct evidence tools for correctness

        // kernel_version - direct uname output
        tools.insert("kernel_version", ToolDef {
            name: "kernel_version",
            description: "Returns the kernel version string from uname -r and full uname -a output",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get the kernel version",
        });

        // memory_info - direct /proc/meminfo data
        tools.insert("memory_info", ToolDef {
            name: "memory_info",
            description: "Returns memory information from /proc/meminfo including total, free, available, cached, and swap",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get memory information",
        });

        // network_status - network-specific evidence
        tools.insert("network_status", ToolDef {
            name: "network_status",
            description: "Returns network status including interface states, IP addresses, default route, DNS servers, and NetworkManager/systemd-networkd status",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "get network status",
        });

        // audio_status - audio-specific evidence
        tools.insert("audio_status", ToolDef {
            name: "audio_status",
            description: "Returns audio status including pipewire/pulseaudio service state, wireplumber status, audio devices, and sinks/sources",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "get audio status",
        });

        // =====================================================================
        // v0.0.46: Domain-Specific Evidence Tools
        // These tools provide targeted evidence for specific question domains
        // to prevent generic snapshot summaries from answering domain questions
        // =====================================================================

        // --- System Domain ---

        tools.insert("uname_summary", ToolDef {
            name: "uname_summary",
            description: "Returns kernel version and architecture from uname. Required for kernel-related questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get kernel version and architecture",
        });

        tools.insert("mem_summary", ToolDef {
            name: "mem_summary",
            description: "Returns memory summary: MemTotal and MemAvailable from /proc/meminfo. Required for memory questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get memory total and available",
        });

        tools.insert("mount_usage", ToolDef {
            name: "mount_usage",
            description: "Returns disk space for / and key mounts with free/used bytes and human strings. Required for disk space questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get disk space usage for mounted filesystems",
        });

        // --- Network Domain ---

        tools.insert("nm_summary", ToolDef {
            name: "nm_summary",
            description: "Returns NetworkManager service state and active connections. Required for network status questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "get NetworkManager status and active connections",
        });

        tools.insert("ip_route_summary", ToolDef {
            name: "ip_route_summary",
            description: "Returns default route and routing table summary. Required for network connectivity questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get default route and routing information",
        });

        tools.insert("link_state_summary", ToolDef {
            name: "link_state_summary",
            description: "Returns network interface states (up/down) and carrier status. Required for network status questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get network interface link states",
        });

        // --- Audio Domain ---

        tools.insert("audio_services_summary", ToolDef {
            name: "audio_services_summary",
            description: "Returns pipewire and wireplumber service states. Required for audio status questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get audio service states (pipewire, wireplumber)",
        });

        tools.insert("pactl_summary", ToolDef {
            name: "pactl_summary",
            description: "Returns default audio sink/source names via pactl (if available). Required for audio output questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "get audio sink and source information from pactl",
        });

        // --- Boot/Logs Domain ---

        tools.insert("boot_time_summary", ToolDef {
            name: "boot_time_summary",
            description: "Returns boot time from systemd-analyze. Required for boot performance questions.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "get boot time analysis from systemd",
        });

        tools.insert("recent_errors_summary", ToolDef {
            name: "recent_errors_summary",
            description: "Returns recent journalctl warnings/errors summarized by service (bounded output). Required for error log questions.",
            parameters: &[
                ("minutes", "integer", false), // default: 30
            ],
            security: ToolSecurity::SensitiveRead,
            latency: LatencyHint::Medium,
            human_request: "get recent errors from system journal",
        });

        // =====================================================================
        // v0.0.47: File Evidence Tools for Mutation Support
        // These tools provide evidence for file mutations (append, edit, etc.)
        // =====================================================================

        tools.insert("file_stat", ToolDef {
            name: "file_stat",
            description: "Returns file metadata: owner uid/gid, mode, size, mtime, exists. Required before file mutations.",
            parameters: &[
                ("path", "string", true),
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get file metadata for mutation safety check",
        });

        tools.insert("file_preview", ToolDef {
            name: "file_preview",
            description: "Returns first N bytes of file with secrets redacted. Used for diff preview before mutations.",
            parameters: &[
                ("path", "string", true),
                ("max_bytes", "integer", false), // default: 2048
            ],
            security: ToolSecurity::SensitiveRead,
            latency: LatencyHint::Fast,
            human_request: "preview file contents for diff generation",
        });

        tools.insert("file_hash", ToolDef {
            name: "file_hash",
            description: "Returns SHA256 hash of file contents. Used for before/after verification.",
            parameters: &[
                ("path", "string", true),
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "compute file hash for integrity verification",
        });

        tools.insert("path_policy_check", ToolDef {
            name: "path_policy_check",
            description: "Returns policy decision for path: allowed/blocked, reason, evidence ID. Required before file mutations.",
            parameters: &[
                ("path", "string", true),
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check path against mutation policy",
        });

        // =====================================================================
        // v0.0.48: Knowledge Search Tool
        // Searches local learned recipes with lightweight ranking
        // =====================================================================

        tools.insert("learned_recipe_search", ToolDef {
            name: "learned_recipe_search",
            description: "Searches local learned recipes for matching patterns. Returns best matches with IDs (K1, K2...), summaries, and match reasons.",
            parameters: &[
                ("query", "string", true),
                ("limit", "integer", false), // default: 5
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "search learned recipes for similar patterns",
        });

        tools.insert("learning_stats", ToolDef {
            name: "learning_stats",
            description: "Returns learning system statistics: XP level/title, recipe count, pack count. Used for status display.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get Anna's learning progress and statistics",
        });

        // =====================================================================
        // v0.0.49: Doctor Network Evidence Tools
        // Specialized tools for NetworkingDoctor diagnosis
        // =====================================================================

        tools.insert("net_interfaces_summary", ToolDef {
            name: "net_interfaces_summary",
            description: "Returns detailed interface info: name, state, operstate, carrier, MAC, IPv4/IPv6, is_wireless. Used by NetworkingDoctor.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get detailed network interface information",
        });

        tools.insert("net_routes_summary", ToolDef {
            name: "net_routes_summary",
            description: "Returns routing info: has_default_route, default_gateway, default_interface, route_count. Used by NetworkingDoctor.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get network routing information",
        });

        tools.insert("dns_summary", ToolDef {
            name: "dns_summary",
            description: "Returns DNS config: servers, source (resolv.conf/systemd-resolved/NM), is_stub_resolver, search domains. Used by NetworkingDoctor.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get DNS configuration details",
        });

        tools.insert("iw_summary", ToolDef {
            name: "iw_summary",
            description: "Returns wireless info: connected, ssid, signal_dbm, signal_quality, frequency, bitrate. Used by NetworkingDoctor.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "get wireless connection details",
        });

        tools.insert("recent_network_errors", ToolDef {
            name: "recent_network_errors",
            description: "Returns network-specific errors from journal: error_count, warning_count, recent messages. Used by NetworkingDoctor.",
            parameters: &[
                ("minutes", "integer", false), // default: 30
            ],
            security: ToolSecurity::SensitiveRead,
            latency: LatencyHint::Medium,
            human_request: "get recent network-related errors from journal",
        });

        tools.insert("ping_check", ToolDef {
            name: "ping_check",
            description: "Pings a target (default: 1.1.1.1) with single packet. Returns success, latency_ms, error. Used by NetworkingDoctor.",
            parameters: &[
                ("target", "string", false), // default: 1.1.1.1
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Slow,
            human_request: "test network connectivity with ping",
        });

        // =====================================================================
        // v0.0.58: Proactive Alerts Tools
        // Daemon-owned alerts for high-signal issue detection
        // =====================================================================

        tools.insert("proactive_alerts_summary", ToolDef {
            name: "proactive_alerts_summary",
            description: "Returns current proactive alerts: counts by severity, active alerts with evidence IDs, recently resolved. For 'show alerts' / 'why are you warning me?' queries.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "get current proactive alerts and warnings",
        });

        tools.insert("disk_pressure_summary", ToolDef {
            name: "disk_pressure_summary",
            description: "Returns disk pressure status: mount point, free space, pressure flag. For disk space alert queries.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check disk pressure status",
        });

        tools.insert("failed_units_summary", ToolDef {
            name: "failed_units_summary",
            description: "Returns list of failed systemd units with evidence. For service failure alert queries.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check for failed systemd units",
        });

        tools.insert("thermal_status_summary", ToolDef {
            name: "thermal_status_summary",
            description: "Returns CPU temperature and throttling status. For thermal alert queries.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "check CPU temperature and thermal status",
        });

        tools.insert("journal_error_burst_summary", ToolDef {
            name: "journal_error_burst_summary",
            description: "Returns units with recent error bursts (>= 20 errors in 10 min). For journal error alert queries.",
            parameters: &[],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Medium,
            human_request: "check for journal error bursts",
        });

        // =====================================================================
        // v0.0.50: User File Mutation Tools
        // For editing user-owned config files with preview, apply, rollback
        // =====================================================================

        tools.insert("file_edit_preview_v1", ToolDef {
            name: "file_edit_preview_v1",
            description: "Preview a file edit without applying. Returns diff, would_change flag, policy check. For user files under $HOME only.",
            parameters: &[
                ("path", "string", true),
                ("mode", "string", true),      // append_line | set_key_value
                ("line", "string", false),     // for append_line
                ("key", "string", false),      // for set_key_value
                ("value", "string", false),    // for set_key_value
                ("separator", "string", false), // default: "="
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "preview file edit without applying",
        });

        tools.insert("file_edit_apply_v1", ToolDef {
            name: "file_edit_apply_v1",
            description: "Apply a file edit with backup. Creates rollback at /var/lib/anna/rollback/<case_id>/. For user files under $HOME only.",
            parameters: &[
                ("path", "string", true),
                ("mode", "string", true),      // append_line | set_key_value
                ("line", "string", false),     // for append_line
                ("key", "string", false),      // for set_key_value
                ("value", "string", false),    // for set_key_value
                ("separator", "string", false), // default: "="
                ("case_id", "string", true),   // for rollback tracking
            ],
            security: ToolSecurity::SensitiveRead, // Actually mutates, marked sensitive
            latency: LatencyHint::Medium,
            human_request: "apply file edit with backup",
        });

        tools.insert("file_edit_rollback_v1", ToolDef {
            name: "file_edit_rollback_v1",
            description: "Rollback a previous file edit by case_id. Restores from /var/lib/anna/rollback/<case_id>/backup/.",
            parameters: &[
                ("case_id", "string", true),
            ],
            security: ToolSecurity::SensitiveRead, // Actually mutates, marked sensitive
            latency: LatencyHint::Medium,
            human_request: "rollback file edit by case ID",
        });

        // v0.0.51: Systemd service action tools
        tools.insert("systemd_service_probe_v1", ToolDef {
            name: "systemd_service_probe_v1",
            description: "Probe a systemd service for its current state. Returns exists, active state, enabled state, description, last failure.",
            parameters: &[
                ("service", "string", true),  // Service name (e.g., "nginx" or "nginx.service")
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "probe systemd service state",
        });

        tools.insert("systemd_service_preview_v1", ToolDef {
            name: "systemd_service_preview_v1",
            description: "Preview a systemd service action without executing. Returns current state, expected changes, risk level, required confirmation.",
            parameters: &[
                ("service", "string", true),     // Service name
                ("operation", "string", true),   // start | stop | restart | enable | disable
            ],
            security: ToolSecurity::ReadOnly,
            latency: LatencyHint::Fast,
            human_request: "preview systemd service action",
        });

        tools.insert("systemd_service_apply_v1", ToolDef {
            name: "systemd_service_apply_v1",
            description: "Apply a systemd service action. Requires prior preview and correct confirmation phrase. Creates rollback metadata.",
            parameters: &[
                ("service", "string", true),        // Service name
                ("operation", "string", true),      // start | stop | restart | enable | disable
                ("preview_id", "string", true),     // Evidence ID from preview
                ("confirmation", "string", true),   // Confirmation phrase (e.g., "I CONFIRM (low risk)")
            ],
            security: ToolSecurity::SensitiveRead, // Actually mutates
            latency: LatencyHint::Medium,
            human_request: "apply systemd service action",
        });

        tools.insert("systemd_service_rollback_v1", ToolDef {
            name: "systemd_service_rollback_v1",
            description: "Rollback a previous systemd service action by case_id. Restores previous active and enabled states.",
            parameters: &[
                ("case_id", "string", true),
            ],
            security: ToolSecurity::SensitiveRead, // Actually mutates
            latency: LatencyHint::Medium,
            human_request: "rollback systemd service action",
        });

        Self { tools }
    }

    /// Get a tool definition by name
    pub fn get(&self, name: &str) -> Option<&ToolDef> {
        self.tools.get(name)
    }

    /// Check if a tool exists in the catalog
    pub fn exists(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }

    /// Get all tool definitions
    pub fn all_tools(&self) -> Vec<&ToolDef> {
        self.tools.values().collect()
    }

    /// Generate a natural language request description for a tool
    pub fn natural_request(&self, tool_name: &str, params: &HashMap<String, serde_json::Value>) -> String {
        if let Some(tool) = self.get(tool_name) {
            let mut request = format!("Please {}", tool.human_request);

            // Add parameter context
            if !params.is_empty() {
                let param_strs: Vec<String> = params.iter()
                    .filter_map(|(k, v)| {
                        match v {
                            serde_json::Value::Number(n) => Some(format!("{}: {}", k, n)),
                            serde_json::Value::String(s) => Some(format!("{}: {}", k, s)),
                            _ => None,
                        }
                    })
                    .collect();
                if !param_strs.is_empty() {
                    request.push_str(&format!(" ({})", param_strs.join(", ")));
                }
            }

            request.push('.');
            request
        } else {
            format!("Please execute unknown tool '{}'.", tool_name)
        }
    }
}

/// Evidence collector that assigns Evidence IDs
#[derive(Debug, Default)]
pub struct EvidenceCollector {
    counter: usize,
    evidence: Vec<ToolResult>,
}

impl EvidenceCollector {
    pub fn new() -> Self {
        Self {
            counter: 0,
            evidence: Vec::new(),
        }
    }

    /// Assign next Evidence ID
    pub fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("E{}", self.counter)
    }

    /// Add evidence and get its ID (generates a new evidence ID)
    pub fn add(&mut self, mut result: ToolResult) -> String {
        let id = self.next_id();
        result.evidence_id = id.clone();
        self.evidence.push(result);
        id
    }

    /// Push evidence that already has an ID assigned (for use with next_id())
    pub fn push(&mut self, result: ToolResult) {
        self.evidence.push(result);
    }

    /// Get all collected evidence
    pub fn all(&self) -> &[ToolResult] {
        &self.evidence
    }

    /// Get evidence by ID
    pub fn get(&self, id: &str) -> Option<&ToolResult> {
        self.evidence.iter().find(|e| e.evidence_id == id)
    }

    /// Format citations for display (e.g., "[E1, E2]")
    pub fn format_citations(&self, ids: &[&str]) -> String {
        if ids.is_empty() {
            String::new()
        } else {
            format!("[{}]", ids.join(", "))
        }
    }
}

/// Tool plan - what tools to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPlan {
    pub tools: Vec<ToolRequest>,
    /// Natural language explanation of why these tools are needed
    pub rationale: String,
}

impl ToolPlan {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            rationale: String::new(),
        }
    }

    pub fn add_tool(&mut self, name: &str, params: HashMap<String, serde_json::Value>) {
        self.tools.push(ToolRequest {
            tool_name: name.to_string(),
            parameters: params,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse tool plan from Translator LLM output
/// Split a string by commas, but only when not inside parentheses
fn split_tools(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut depth: u32 = 0;

    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    // Add the last segment
    if start < s.len() {
        result.push(&s[start..]);
    }

    result
}

pub fn parse_tool_plan(output: &str) -> Option<ToolPlan> {
    let mut plan = ToolPlan::new();

    for line in output.lines() {
        let line = line.trim();

        // Parse TOOLS: tool1, tool2(param=value), ...
        if let Some(tools_str) = line.strip_prefix("TOOLS:") {
            let tools_str = tools_str.trim();
            if tools_str.to_lowercase() == "none" {
                continue;
            }

            // Split tools by comma, respecting parentheses
            for tool_spec in split_tools(tools_str) {
                let tool_spec = tool_spec.trim();
                if tool_spec.is_empty() {
                    continue;
                }

                // Parse tool_name or tool_name(param=value, ...)
                if let Some(paren_idx) = tool_spec.find('(') {
                    let name = &tool_spec[..paren_idx];
                    let params_str = tool_spec[paren_idx+1..].trim_end_matches(')');
                    let mut params = HashMap::new();

                    for param in params_str.split(',') {
                        let param = param.trim();
                        if let Some(eq_idx) = param.find('=') {
                            let key = param[..eq_idx].trim();
                            let value = param[eq_idx+1..].trim();

                            // Try to parse as number, otherwise string
                            if let Ok(n) = value.parse::<i64>() {
                                params.insert(key.to_string(), serde_json::json!(n));
                            } else {
                                params.insert(key.to_string(), serde_json::json!(value));
                            }
                        }
                    }

                    plan.add_tool(name, params);
                } else {
                    plan.add_tool(tool_spec, HashMap::new());
                }
            }
        }

        // Parse RATIONALE: why these tools
        if let Some(rationale) = line.strip_prefix("RATIONALE:") {
            plan.rationale = rationale.trim().to_string();
        }
    }

    if plan.tools.is_empty() {
        None
    } else {
        Some(plan)
    }
}

/// Create an "unavailable" result for unimplemented tools
pub fn unavailable_result(tool_name: &str, evidence_id: &str) -> ToolResult {
    ToolResult {
        tool_name: tool_name.to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::json!({
            "status": "unavailable",
            "reason": "Tool not yet implemented"
        }),
        human_summary: format!("The {} tool is not yet available.", tool_name),
        success: false,
        error: Some("Tool not yet implemented".to_string()),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs(),
    }
}

/// Create an "unknown" result for tools not in catalog
pub fn unknown_tool_result(tool_name: &str, evidence_id: &str) -> ToolResult {
    ToolResult {
        tool_name: tool_name.to_string(),
        evidence_id: evidence_id.to_string(),
        data: serde_json::json!({
            "status": "unknown",
            "reason": "Tool not in allowlist"
        }),
        human_summary: format!("Unknown tool '{}' - not in the allowed tool catalog.", tool_name),
        success: false,
        error: Some("Tool not in allowlist".to_string()),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_catalog_exists() {
        let catalog = ToolCatalog::new();
        assert!(catalog.exists("status_snapshot"));
        assert!(catalog.exists("sw_snapshot_summary"));
        assert!(catalog.exists("hw_snapshot_summary"));
        assert!(catalog.exists("recent_installs"));
        assert!(catalog.exists("journal_warnings"));
        assert!(catalog.exists("boot_time_trend"));
        assert!(catalog.exists("top_resource_processes"));
        assert!(!catalog.exists("nonexistent_tool"));
    }

    #[test]
    fn test_parse_tool_plan_simple() {
        let output = "TOOLS: status_snapshot, hw_snapshot_summary\nRATIONALE: Need system status";
        let plan = parse_tool_plan(output).unwrap();
        assert_eq!(plan.tools.len(), 2);
        assert_eq!(plan.tools[0].tool_name, "status_snapshot");
        assert_eq!(plan.tools[1].tool_name, "hw_snapshot_summary");
        assert_eq!(plan.rationale, "Need system status");
    }

    #[test]
    fn test_parse_tool_plan_with_params() {
        let output = "TOOLS: recent_installs(days=7), journal_warnings(service=nginx, minutes=60)";
        let plan = parse_tool_plan(output).unwrap();
        assert_eq!(plan.tools.len(), 2);
        assert_eq!(plan.tools[0].tool_name, "recent_installs");
        assert_eq!(plan.tools[0].parameters.get("days"), Some(&serde_json::json!(7)));
        assert_eq!(plan.tools[1].tool_name, "journal_warnings");
        assert_eq!(plan.tools[1].parameters.get("service"), Some(&serde_json::json!("nginx")));
        assert_eq!(plan.tools[1].parameters.get("minutes"), Some(&serde_json::json!(60)));
    }

    #[test]
    fn test_parse_tool_plan_none() {
        let output = "TOOLS: none";
        let plan = parse_tool_plan(output);
        assert!(plan.is_none());
    }

    #[test]
    fn test_evidence_collector() {
        let mut collector = EvidenceCollector::new();

        let result1 = ToolResult {
            tool_name: "test".to_string(),
            evidence_id: String::new(),
            data: serde_json::json!({"key": "value"}),
            human_summary: "Test result".to_string(),
            success: true,
            error: None,
            timestamp: 0,
        };

        let id1 = collector.add(result1);
        assert_eq!(id1, "E1");

        let result2 = ToolResult {
            tool_name: "test2".to_string(),
            evidence_id: String::new(),
            data: serde_json::json!({}),
            human_summary: "Test 2".to_string(),
            success: true,
            error: None,
            timestamp: 0,
        };

        let id2 = collector.add(result2);
        assert_eq!(id2, "E2");

        assert_eq!(collector.all().len(), 2);
        assert!(collector.get("E1").is_some());
        assert!(collector.get("E2").is_some());
        assert!(collector.get("E3").is_none());
    }

    #[test]
    fn test_format_citations() {
        let collector = EvidenceCollector::new();
        assert_eq!(collector.format_citations(&[]), "");
        assert_eq!(collector.format_citations(&["E1"]), "[E1]");
        assert_eq!(collector.format_citations(&["E1", "E2", "E3"]), "[E1, E2, E3]");
    }

    #[test]
    fn test_natural_request() {
        let catalog = ToolCatalog::new();

        let request = catalog.natural_request("status_snapshot", &HashMap::new());
        assert!(request.contains("status"));

        let mut params = HashMap::new();
        params.insert("days".to_string(), serde_json::json!(7));
        let request = catalog.natural_request("recent_installs", &params);
        assert!(request.contains("days: 7"));
    }

    #[test]
    fn test_unavailable_result() {
        let result = unavailable_result("test_tool", "E1");
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.human_summary.contains("not yet available"));
    }

    #[test]
    fn test_unknown_tool_result() {
        let result = unknown_tool_result("bad_tool", "E1");
        assert!(!result.success);
        assert!(result.human_summary.contains("Unknown tool"));
    }
}
