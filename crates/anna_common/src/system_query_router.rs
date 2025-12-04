//! System Query Router v0.0.58 - Deterministic routing to correct evidence tools.
//! Ensures disk queries use disk tools, kernel queries use kernel tools, etc.
//!
//! Targets: cpu, memory, disk_free, kernel_version, network_status, audio_status, services_status, alerts

use serde::{Deserialize, Serialize};

/// Canonical system query target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryTarget {
    Cpu,
    Memory,
    DiskFree,
    KernelVersion,
    NetworkStatus,
    AudioStatus,
    ServicesStatus,
    /// General hardware query (CPU model, GPU, etc.)
    Hardware,
    /// General software query (packages, services)
    Software,
    /// v0.0.58: Proactive alerts query ("show alerts", "why are you warning me")
    Alerts,
    /// Unknown target - needs LLM
    Unknown,
}

impl QueryTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            QueryTarget::Cpu => "cpu",
            QueryTarget::Memory => "memory",
            QueryTarget::DiskFree => "disk_free",
            QueryTarget::KernelVersion => "kernel_version",
            QueryTarget::NetworkStatus => "network_status",
            QueryTarget::AudioStatus => "audio_status",
            QueryTarget::ServicesStatus => "services_status",
            QueryTarget::Hardware => "hardware",
            QueryTarget::Software => "software",
            QueryTarget::Alerts => "alerts",
            QueryTarget::Unknown => "unknown",
        }
    }

    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cpu" | "processor" => QueryTarget::Cpu,
            "memory" | "ram" | "mem" => QueryTarget::Memory,
            "disk" | "disk_free" | "storage" | "space" => QueryTarget::DiskFree,
            "kernel" | "kernel_version" | "uname" => QueryTarget::KernelVersion,
            "network" | "network_status" | "wifi" | "ethernet" | "net" => QueryTarget::NetworkStatus,
            "audio" | "audio_status" | "sound" | "pipewire" => QueryTarget::AudioStatus,
            "service" | "services" | "services_status" => QueryTarget::ServicesStatus,
            "hardware" | "hw" => QueryTarget::Hardware,
            "software" | "sw" | "packages" => QueryTarget::Software,
            "alerts" | "warnings" | "issues" | "problems" => QueryTarget::Alerts,
            _ => QueryTarget::Unknown,
        }
    }
}

/// Tool routing for a query target
#[derive(Debug, Clone)]
pub struct ToolRouting {
    /// Primary tool(s) that MUST be used
    pub required_tools: Vec<&'static str>,
    /// Optional supplementary tools
    pub optional_tools: Vec<&'static str>,
    /// Human-readable description of expected output
    pub output_description: &'static str,
}

/// Get the tool routing for a canonical target
pub fn get_tool_routing(target: QueryTarget) -> ToolRouting {
    match target {
        QueryTarget::Cpu => ToolRouting {
            required_tools: vec!["hw_snapshot_summary"],
            optional_tools: vec![],
            output_description: "CPU model name, cores, threads, frequency",
        },
        QueryTarget::Memory => ToolRouting {
            required_tools: vec!["memory_info"],
            optional_tools: vec!["mem_summary"],
            output_description: "Total RAM in GiB, available RAM, used RAM",
        },
        QueryTarget::DiskFree => ToolRouting {
            required_tools: vec!["mount_usage"],
            optional_tools: vec!["disk_usage"],
            output_description: "Free/used space for / and /home if separate",
        },
        QueryTarget::KernelVersion => ToolRouting {
            required_tools: vec!["kernel_version"],
            optional_tools: vec!["uname_summary"],
            output_description: "Exact kernel release string (e.g., 6.x.x-arch1-1)",
        },
        QueryTarget::NetworkStatus => ToolRouting {
            required_tools: vec!["network_status"],
            optional_tools: vec!["nm_summary", "ip_route_summary", "link_state_summary"],
            output_description: "Connected interface, IPv4 presence, default route, DNS servers",
        },
        QueryTarget::AudioStatus => ToolRouting {
            required_tools: vec!["audio_status"],
            optional_tools: vec!["audio_services_summary", "pactl_summary"],
            output_description: "PipeWire/WirePlumber running, default sink present",
        },
        QueryTarget::ServicesStatus => ToolRouting {
            required_tools: vec!["service_status"],
            optional_tools: vec!["systemd_service_probe_v1"],
            output_description: "Service active/enabled state, last error if failed",
        },
        QueryTarget::Hardware => ToolRouting {
            required_tools: vec!["hw_snapshot_summary"],
            optional_tools: vec![],
            output_description: "CPU, memory, storage, GPU, network summary",
        },
        QueryTarget::Software => ToolRouting {
            required_tools: vec!["sw_snapshot_summary"],
            optional_tools: vec!["status_snapshot"],
            output_description: "Installed packages, running services",
        },
        QueryTarget::Alerts => ToolRouting {
            required_tools: vec!["proactive_alerts_summary"],
            optional_tools: vec!["failed_units_summary", "disk_pressure_summary", "thermal_status_summary"],
            output_description: "Active alerts count, top alerts with evidence IDs, recently resolved",
        },
        QueryTarget::Unknown => ToolRouting {
            required_tools: vec![],
            optional_tools: vec![],
            output_description: "Unknown target - use LLM for routing",
        },
    }
}

/// Detect the canonical target from a user request
/// Returns the detected target and confidence (0-100)
pub fn detect_target(request: &str) -> (QueryTarget, u8) {
    let request_lower = request.to_lowercase();

    // Disk/storage - highest priority patterns
    let disk_patterns = [
        ("disk space", 95), ("disk free", 95), ("free space", 90),
        ("storage space", 90), ("how much space", 95), ("space left", 90),
        ("space available", 90), ("space on /", 95), ("disk usage", 85),
        ("running out of space", 90), ("disk full", 90),
    ];
    for (pattern, confidence) in disk_patterns {
        if request_lower.contains(pattern) {
            return (QueryTarget::DiskFree, confidence);
        }
    }

    // Kernel version - specific patterns
    let kernel_patterns = [
        ("kernel version", 95), ("kernel release", 95), ("what kernel", 90),
        ("linux version", 85), ("which kernel", 90), ("running kernel", 90),
    ];
    for (pattern, confidence) in kernel_patterns {
        if request_lower.contains(pattern) {
            return (QueryTarget::KernelVersion, confidence);
        }
    }

    // Memory/RAM - be careful not to match "how much" alone
    let memory_patterns = [
        ("how much memory", 95), ("how much ram", 95), ("ram available", 90),
        ("memory available", 90), ("ram free", 90), ("memory free", 90),
        ("total memory", 90), ("total ram", 90), ("ram usage", 85),
        ("memory usage", 85), ("how much mem", 90),
    ];
    for (pattern, confidence) in memory_patterns {
        if request_lower.contains(pattern) {
            return (QueryTarget::Memory, confidence);
        }
    }

    // CPU - specific patterns
    let cpu_patterns = [
        ("what cpu", 95), ("which cpu", 90), ("cpu model", 90),
        ("processor model", 85), ("cpu info", 85), ("processor info", 85),
        ("what processor", 90), ("how many cores", 85), ("cpu cores", 85),
    ];
    for (pattern, confidence) in cpu_patterns {
        if request_lower.contains(pattern) {
            return (QueryTarget::Cpu, confidence);
        }
    }

    // Network status - specific patterns
    let network_patterns = [
        ("network status", 95), ("network connection", 90),
        ("internet connection", 90), ("am i connected", 90),
        ("am i online", 90), ("is network", 85), ("is wifi", 85),
        ("wifi status", 90), ("ethernet status", 90),
        ("connection status", 85), ("default route", 85),
    ];
    for (pattern, confidence) in network_patterns {
        if request_lower.contains(pattern) {
            return (QueryTarget::NetworkStatus, confidence);
        }
    }

    // Audio status - specific patterns
    let audio_patterns = [
        ("audio status", 95), ("sound status", 90), ("is audio", 85),
        ("is sound", 85), ("audio working", 90), ("sound working", 90),
        ("pipewire status", 90), ("pulseaudio status", 90),
        ("no sound", 85), ("no audio", 85),
    ];
    for (pattern, confidence) in audio_patterns {
        if request_lower.contains(pattern) {
            return (QueryTarget::AudioStatus, confidence);
        }
    }

    // Service status - pattern: "is X running" or "X status"
    let service_keywords = [
        "nginx", "docker", "sshd", "ssh", "apache", "mysql", "postgresql",
        "redis", "mongodb", "systemd", "networkmanager",
    ];
    for svc in service_keywords {
        if request_lower.contains(svc) {
            if request_lower.contains("running") || request_lower.contains("status")
                || request_lower.contains("started") || request_lower.contains("enabled")
            {
                return (QueryTarget::ServicesStatus, 85);
            }
        }
    }

    // General hardware query
    if request_lower.contains("hardware") || request_lower.contains("system info")
        || request_lower.contains("system specs")
    {
        return (QueryTarget::Hardware, 70);
    }

    // v0.0.58: Alert queries - "show alerts", "why are you warning me", etc.
    let alert_patterns = [
        ("show alerts", 95), ("what alerts", 95), ("any alerts", 90),
        ("any warnings", 90), ("show warnings", 90), ("any issues", 85),
        ("why are you warning", 95), ("why warning", 90), ("why the warning", 95),
        ("system alerts", 90), ("active alerts", 95), ("current alerts", 90),
        ("what's wrong", 80), ("any problems", 85),
    ];
    for (pattern, confidence) in alert_patterns {
        if request_lower.contains(pattern) {
            return (QueryTarget::Alerts, confidence);
        }
    }

    // Unknown - needs LLM
    (QueryTarget::Unknown, 0)
}

/// Validate that tool results contain expected data for target. Returns (is_valid, critique).
pub fn validate_answer_for_target(target: QueryTarget, answer: &str) -> (bool, String) {
    let a = answer.to_lowercase();
    let cpu_noise = a.contains("cpu:") || a.contains("processor:") || a.contains("cores");

    match target {
        QueryTarget::DiskFree => {
            let has = a.contains("free") || a.contains("used") || a.contains("gib")
                || a.contains("/") || a.contains("disk") || a.contains("mount");
            if cpu_noise && !has { (false, "Answer contains CPU info but not disk info".into()) }
            else if !has { (false, "Answer missing disk free space information".into()) }
            else { (true, String::new()) }
        }
        QueryTarget::KernelVersion => {
            let has = a.contains("kernel") || a.contains("linux") || answer.contains("6.") || answer.contains("5.");
            if cpu_noise && !has { (false, "Answer contains CPU info but not kernel version".into()) }
            else if !has { (false, "Answer missing kernel version string".into()) }
            else { (true, String::new()) }
        }
        QueryTarget::Memory => {
            let has = a.contains("memory") || a.contains("ram") || a.contains("gib") || a.contains("available");
            if !has { (false, "Answer missing memory/RAM information".into()) } else { (true, String::new()) }
        }
        QueryTarget::Cpu => {
            let has = a.contains("cpu") || a.contains("processor") || a.contains("cores")
                || a.contains("amd") || a.contains("intel") || a.contains("ryzen");
            if !has { (false, "Answer missing CPU information".into()) } else { (true, String::new()) }
        }
        QueryTarget::NetworkStatus => {
            let has = a.contains("network") || a.contains("interface") || a.contains("connected")
                || a.contains("ip") || a.contains("route") || a.contains("wifi");
            if !has { (false, "Answer missing network status information".into()) } else { (true, String::new()) }
        }
        QueryTarget::AudioStatus => {
            let has = a.contains("audio") || a.contains("sound") || a.contains("pipewire")
                || a.contains("pulse") || a.contains("sink");
            if !has { (false, "Answer missing audio status information".into()) } else { (true, String::new()) }
        }
        QueryTarget::Alerts => {
            let has = a.contains("alert") || a.contains("warning") || a.contains("critical")
                || a.contains("no active") || a.contains("issue") || a.contains("problem");
            if !has { (false, "Answer missing alerts/warnings information".into()) } else { (true, String::new()) }
        }
        _ => (true, String::new()),
    }
}

/// Map old-style targets (from translator) to canonical targets
pub fn map_translator_targets(targets: &[String]) -> Vec<QueryTarget> {
    targets.iter()
        .map(|t| QueryTarget::parse(t))
        .filter(|t| *t != QueryTarget::Unknown)
        .collect()
}

/// Get the required tools for a list of canonical targets
/// Returns deduplicated list of tool names
pub fn get_required_tools(targets: &[QueryTarget]) -> Vec<String> {
    let mut tools: Vec<String> = targets.iter()
        .flat_map(|t| get_tool_routing(*t).required_tools)
        .map(|s| s.to_string())
        .collect();
    tools.sort();
    tools.dedup();
    tools
}

#[cfg(test)]
mod tests {
    use super::*;

    // Core detection tests - the main bug fixes for v0.0.52
    #[test]
    fn test_detect_disk_query() {
        let (target, confidence) = detect_target("how much disk space is free");
        assert_eq!(target, QueryTarget::DiskFree);
        assert!(confidence >= 90);
    }

    #[test]
    fn test_detect_kernel_query() {
        let (target, confidence) = detect_target("what kernel version am I using");
        assert_eq!(target, QueryTarget::KernelVersion);
        assert!(confidence >= 90);
    }

    #[test]
    fn test_detect_memory_query() {
        let (target, confidence) = detect_target("how much memory do I have");
        assert_eq!(target, QueryTarget::Memory);
        assert!(confidence >= 90);
    }

    #[test]
    fn test_detect_cpu_query() {
        let (target, confidence) = detect_target("what cpu do I have");
        assert_eq!(target, QueryTarget::Cpu);
        assert!(confidence >= 90);
    }

    // Critical: answers must match query target (the bug we're fixing)
    #[test]
    fn test_validate_disk_answer_wrong_cpu() {
        let (valid, critique) = validate_answer_for_target(
            QueryTarget::DiskFree,
            "CPU: AMD Ryzen 5 3600, 6 cores, 12 threads"
        );
        assert!(!valid);
        assert!(critique.contains("CPU"));
    }

    #[test]
    fn test_validate_kernel_answer_wrong_cpu() {
        let (valid, critique) = validate_answer_for_target(
            QueryTarget::KernelVersion,
            "CPU: AMD Ryzen 7 5800X, 8 cores"
        );
        assert!(!valid);
        assert!(critique.contains("kernel"));
    }

    #[test]
    fn test_validate_disk_answer_correct() {
        let (valid, _) = validate_answer_for_target(
            QueryTarget::DiskFree,
            "Disk free: 433.7 GiB, used: 45% on /"
        );
        assert!(valid);
    }

    // Tool routing tests
    #[test]
    fn test_tool_routing() {
        assert!(get_tool_routing(QueryTarget::DiskFree).required_tools.contains(&"mount_usage"));
        assert!(get_tool_routing(QueryTarget::KernelVersion).required_tools.contains(&"kernel_version"));
        assert!(get_tool_routing(QueryTarget::Memory).required_tools.contains(&"memory_info"));
        assert!(get_tool_routing(QueryTarget::NetworkStatus).required_tools.contains(&"network_status"));
        assert!(get_tool_routing(QueryTarget::AudioStatus).required_tools.contains(&"audio_status"));
    }

    // Memory vs disk disambiguation (common confusion)
    #[test]
    fn test_detect_memory_not_disk() {
        let (target, _) = detect_target("how much memory do I have");
        assert_eq!(target, QueryTarget::Memory);
        assert_ne!(target, QueryTarget::DiskFree);
    }

    #[test]
    fn test_detect_unknown_for_ambiguous() {
        let (target, confidence) = detect_target("what time is it");
        assert_eq!(target, QueryTarget::Unknown);
        assert_eq!(confidence, 0);
    }

    #[test]
    fn test_map_translator_targets() {
        let targets = vec!["cpu".to_string(), "memory".to_string(), "bogus".to_string()];
        let mapped = map_translator_targets(&targets);
        assert_eq!(mapped.len(), 2);
        assert!(mapped.contains(&QueryTarget::Cpu));
        assert!(mapped.contains(&QueryTarget::Memory));
    }

    // v0.0.58: Alert routing tests
    #[test]
    fn test_detect_alert_query() {
        let (target, confidence) = detect_target("show me any alerts");
        assert_eq!(target, QueryTarget::Alerts);
        assert!(confidence >= 85);
    }

    #[test]
    fn test_detect_why_warning_query() {
        let (target, confidence) = detect_target("why are you warning me?");
        assert_eq!(target, QueryTarget::Alerts);
        assert!(confidence >= 90);
    }

    #[test]
    fn test_alert_tool_routing() {
        let routing = get_tool_routing(QueryTarget::Alerts);
        assert!(routing.required_tools.contains(&"proactive_alerts_summary"));
    }

    #[test]
    fn test_validate_alert_answer() {
        let (valid, _) = validate_answer_for_target(
            QueryTarget::Alerts,
            "No active alerts. System is healthy."
        );
        assert!(valid);
    }
}
