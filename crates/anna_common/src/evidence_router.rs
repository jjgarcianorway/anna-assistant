//! Evidence Router v0.0.65 - Deterministic Topic Selection
//!
//! Routes queries to the correct evidence topics:
//! - "how much disk space" → DiskFree
//! - "kernel version" → KernelVersion
//! - "network status" → NetworkStatus (multiple sub-topics)
//! - "audio working" → AudioStatus
//!
//! Prevents generic hw/sw summaries from satisfying specific questions.

use crate::evidence_topic::EvidenceTopic;
use crate::service_desk::{Ticket, TicketCategory};

// ============================================================================
// Routing Result
// ============================================================================

/// Result of evidence routing
#[derive(Debug, Clone)]
pub struct EvidenceRouting {
    /// Primary topics required to answer the question
    pub required_topics: Vec<EvidenceTopic>,
    /// Optional supplementary topics
    pub optional_topics: Vec<EvidenceTopic>,
    /// Service name if querying a specific service
    pub service_name: Option<String>,
    /// Package name if querying a specific package
    pub package_name: Option<String>,
    /// Human description of what we're checking
    pub checking_description: String,
    /// Whether this is a diagnostic (problem) vs informational query
    pub is_diagnostic: bool,
}

impl EvidenceRouting {
    /// Create a simple routing with one required topic
    pub fn single(topic: EvidenceTopic, description: &str) -> Self {
        Self {
            required_topics: vec![topic],
            optional_topics: vec![],
            service_name: None,
            package_name: None,
            checking_description: description.to_string(),
            is_diagnostic: false,
        }
    }

    /// Create routing with multiple required topics
    pub fn multiple(topics: Vec<EvidenceTopic>, description: &str) -> Self {
        Self {
            required_topics: topics,
            optional_topics: vec![],
            service_name: None,
            package_name: None,
            checking_description: description.to_string(),
            is_diagnostic: false,
        }
    }

    /// Mark as diagnostic query
    pub fn diagnostic(mut self) -> Self {
        self.is_diagnostic = true;
        self
    }

    /// Add optional topics
    pub fn with_optional(mut self, topics: Vec<EvidenceTopic>) -> Self {
        self.optional_topics = topics;
        self
    }

    /// Add service name context
    pub fn for_service(mut self, service: &str) -> Self {
        self.service_name = Some(service.to_string());
        self
    }

    /// Add package name context
    pub fn for_package(mut self, package: &str) -> Self {
        self.package_name = Some(package.to_string());
        self
    }
}

// ============================================================================
// Main Router
// ============================================================================

/// Route a query to the appropriate evidence topics
pub fn route_evidence(request: &str, ticket: Option<&Ticket>) -> EvidenceRouting {
    let lower = request.to_lowercase();

    // Check ticket category hint first
    if let Some(t) = ticket {
        if let Some(routing) = route_by_category(t.category, &lower) {
            return routing;
        }
    }

    // Disk space queries
    if is_disk_query(&lower) {
        return route_disk_query(&lower);
    }

    // Kernel version queries
    if is_kernel_query(&lower) {
        return EvidenceRouting::single(EvidenceTopic::KernelVersion, "checking kernel version");
    }

    // Memory queries
    if is_memory_query(&lower) {
        return EvidenceRouting::single(EvidenceTopic::MemoryInfo, "checking memory usage");
    }

    // CPU queries
    if is_cpu_query(&lower) {
        return EvidenceRouting::single(EvidenceTopic::CpuInfo, "checking CPU information");
    }

    // Network queries
    if is_network_query(&lower) {
        return route_network_query(&lower);
    }

    // Audio queries
    if is_audio_query(&lower) {
        return route_audio_query(&lower);
    }

    // Service queries
    if let Some(service) = extract_service_name(&lower) {
        return EvidenceRouting::single(
            EvidenceTopic::ServiceState,
            &format!("checking {} status", service),
        )
        .for_service(&service);
    }

    // Boot time queries
    if is_boot_query(&lower) {
        return EvidenceRouting::single(EvidenceTopic::BootTime, "checking boot time");
    }

    // Package queries
    if is_package_query(&lower) {
        return EvidenceRouting::single(
            EvidenceTopic::PackagesChanged,
            "checking recent package changes",
        );
    }

    // Graphics queries
    if is_graphics_query(&lower) {
        return EvidenceRouting::single(
            EvidenceTopic::GraphicsStatus,
            "checking graphics configuration",
        );
    }

    // Alert queries
    if is_alert_query(&lower) {
        return EvidenceRouting::single(EvidenceTopic::Alerts, "checking system alerts");
    }

    // Errors/warnings queries
    if is_error_query(&lower) {
        return EvidenceRouting::single(
            EvidenceTopic::RecentErrors,
            "checking recent system errors",
        );
    }

    // Default: unknown - will need LLM classification
    EvidenceRouting::single(EvidenceTopic::Unknown, "gathering system information")
}

/// Route based on ticket category
fn route_by_category(category: TicketCategory, request: &str) -> Option<EvidenceRouting> {
    match category {
        TicketCategory::Networking => Some(route_network_query(request)),
        TicketCategory::Storage => Some(route_disk_query(request)),
        TicketCategory::Audio => Some(route_audio_query(request)),
        _ => None, // Let pattern matching handle it
    }
}

// ============================================================================
// Query Detection
// ============================================================================

fn is_disk_query(request: &str) -> bool {
    let patterns = [
        "disk space",
        "disk free",
        "free space",
        "storage space",
        "how much space",
        "space left",
        "space available",
        "disk usage",
        "running out of space",
        "disk full",
        "storage left",
        "how full",
        "root partition",
        "disk",
        "filesystem",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_kernel_query(request: &str) -> bool {
    let patterns = [
        "kernel version",
        "kernel release",
        "what kernel",
        "linux version",
        "which kernel",
        "running kernel",
        "uname",
        "kernel am i",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_memory_query(request: &str) -> bool {
    let patterns = [
        "how much memory",
        "how much ram",
        "ram available",
        "memory available",
        "ram free",
        "memory free",
        "total memory",
        "total ram",
        "ram usage",
        "memory usage",
        "how much mem",
        "ram do i have",
        "memory do i have",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_cpu_query(request: &str) -> bool {
    let patterns = [
        "what cpu",
        "which cpu",
        "cpu model",
        "processor model",
        "cpu info",
        "processor info",
        "what processor",
        "how many cores",
        "cpu cores",
        "cpu do i have",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_network_query(request: &str) -> bool {
    let patterns = [
        "network status",
        "network connection",
        "internet connection",
        "am i connected",
        "am i online",
        "is network",
        "is wifi",
        "wifi status",
        "ethernet status",
        "wifi connected",
        "wifi working",
        "connection status",
        "default route",
        "network info",
        "dns",
        "ip address",
        "my ip",
        "ping",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_audio_query(request: &str) -> bool {
    let patterns = [
        "audio status",
        "sound status",
        "is audio",
        "is sound",
        "audio working",
        "sound working",
        "pipewire",
        "pulseaudio",
        "no sound",
        "no audio",
        "speakers",
        "audio device",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_boot_query(request: &str) -> bool {
    let patterns = [
        "boot time",
        "startup time",
        "how long to boot",
        "boot slow",
        "slow boot",
        "boot analysis",
        "uptime",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_package_query(request: &str) -> bool {
    let patterns = [
        "what changed",
        "recently installed",
        "installed recently",
        "packages changed",
        "new packages",
        "updated packages",
        "package version",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_graphics_query(request: &str) -> bool {
    let patterns = [
        "gpu",
        "graphics card",
        "video card",
        "graphics driver",
        "nvidia",
        "amd gpu",
        "intel graphics",
        "display",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_alert_query(request: &str) -> bool {
    let patterns = [
        "show alerts",
        "what alerts",
        "any alerts",
        "any warnings",
        "show warnings",
        "any issues",
        "system alerts",
        "active alerts",
    ];
    patterns.iter().any(|p| request.contains(p))
}

fn is_error_query(request: &str) -> bool {
    let patterns = ["recent error", "show error", "what error", "journal error"];
    patterns.iter().any(|p| request.contains(p))
}

// ============================================================================
// Specialized Routers
// ============================================================================

/// Route disk queries - might need multiple topics
fn route_disk_query(request: &str) -> EvidenceRouting {
    let is_problem =
        request.contains("full") || request.contains("running out") || request.contains("no space");

    let mut routing = EvidenceRouting::single(
        EvidenceTopic::DiskFree,
        "checking disk space on filesystems",
    );

    if is_problem {
        routing = routing.diagnostic();
    }

    routing
}

/// Route network queries - might need multiple sub-checks
fn route_network_query(request: &str) -> EvidenceRouting {
    let is_problem = request.contains("not working")
        || request.contains("disconnect")
        || request.contains("slow")
        || request.contains("can't connect");

    let mut routing = EvidenceRouting::single(
        EvidenceTopic::NetworkStatus,
        "checking network connectivity",
    );

    if is_problem {
        routing = routing.diagnostic();
    }

    routing
}

/// Route audio queries
fn route_audio_query(request: &str) -> EvidenceRouting {
    let is_problem = request.contains("not working")
        || request.contains("no sound")
        || request.contains("no audio")
        || request.contains("broken");

    let mut routing =
        EvidenceRouting::single(EvidenceTopic::AudioStatus, "checking audio stack status");

    if is_problem {
        routing = routing.diagnostic();
    }

    routing
}

/// Extract service name from query
fn extract_service_name(request: &str) -> Option<String> {
    let service_keywords = [
        ("nginx", "nginx"),
        ("docker", "docker"),
        ("sshd", "sshd"),
        ("ssh", "sshd"),
        ("apache", "apache2"),
        ("httpd", "httpd"),
        ("mysql", "mysql"),
        ("mariadb", "mariadb"),
        ("postgresql", "postgresql"),
        ("redis", "redis"),
        ("mongodb", "mongodb"),
        ("systemd", "systemd"),
        ("networkmanager", "NetworkManager"),
        ("bluetooth", "bluetooth"),
    ];

    for (keyword, service) in service_keywords {
        if request.contains(keyword) {
            if request.contains("running")
                || request.contains("status")
                || request.contains("started")
                || request.contains("enabled")
                || request.contains("active")
                || request.contains("is ")
            {
                return Some(service.to_string());
            }
        }
    }
    None
}

// ============================================================================
// Blacklist: Tools that cannot satisfy specific topics
// ============================================================================

/// Check if a tool can satisfy a specific topic
/// This prevents hw/sw_snapshot_summary from satisfying specific queries
pub fn tool_satisfies_topic(tool_name: &str, topic: EvidenceTopic) -> bool {
    // Generic summaries can ONLY satisfy Unknown or broad queries
    let generic_tools = [
        "hw_snapshot_summary",
        "sw_snapshot_summary",
        "status_snapshot",
    ];

    if generic_tools.contains(&tool_name) {
        // Generic tools can only satisfy GraphicsStatus (for GPU info) or Unknown
        return matches!(
            topic,
            EvidenceTopic::GraphicsStatus | EvidenceTopic::Unknown
        );
    }

    // Specific tools must match their topics
    match topic {
        EvidenceTopic::DiskFree => tool_name == "mount_usage" || tool_name == "disk_usage",
        EvidenceTopic::KernelVersion => tool_name == "kernel_version",
        EvidenceTopic::MemoryInfo => tool_name == "memory_info",
        EvidenceTopic::CpuInfo => tool_name == "cpu_info" || tool_name == "hw_snapshot_summary",
        EvidenceTopic::NetworkStatus => {
            tool_name == "network_status" || tool_name == "net_interfaces_summary"
        }
        EvidenceTopic::AudioStatus => tool_name == "audio_status",
        EvidenceTopic::ServiceState => {
            tool_name == "service_status" || tool_name == "systemd_service_probe_v1"
        }
        EvidenceTopic::BootTime => tool_name == "boot_time_trend",
        EvidenceTopic::PackagesChanged => {
            tool_name == "recent_installs" || tool_name == "what_changed"
        }
        EvidenceTopic::GraphicsStatus => true, // hw_snapshot_summary is OK for GPU
        EvidenceTopic::Alerts => tool_name == "proactive_alerts_summary",
        EvidenceTopic::RecentErrors => tool_name == "journal_warnings",
        EvidenceTopic::Unknown => true, // Any tool is fine for unknown
    }
}

/// Get the correct tool for a topic
pub fn get_tool_for_topic(topic: EvidenceTopic) -> &'static str {
    match topic {
        EvidenceTopic::DiskFree => "mount_usage",
        EvidenceTopic::KernelVersion => "kernel_version",
        EvidenceTopic::MemoryInfo => "memory_info",
        EvidenceTopic::CpuInfo => "cpu_info",
        EvidenceTopic::NetworkStatus => "network_status",
        EvidenceTopic::AudioStatus => "audio_status",
        EvidenceTopic::ServiceState => "service_status",
        EvidenceTopic::BootTime => "boot_time_trend",
        EvidenceTopic::PackagesChanged => "recent_installs",
        EvidenceTopic::GraphicsStatus => "hw_snapshot_summary",
        EvidenceTopic::Alerts => "proactive_alerts_summary",
        EvidenceTopic::RecentErrors => "journal_warnings",
        EvidenceTopic::Unknown => "status_snapshot",
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_routing() {
        let routing = route_evidence("how much disk space is free", None);
        assert!(routing.required_topics.contains(&EvidenceTopic::DiskFree));
        assert!(!routing.is_diagnostic);
    }

    #[test]
    fn test_kernel_routing() {
        let routing = route_evidence("what kernel version am I running", None);
        assert!(routing
            .required_topics
            .contains(&EvidenceTopic::KernelVersion));
    }

    #[test]
    fn test_memory_routing() {
        let routing = route_evidence("how much memory do I have", None);
        assert!(routing.required_topics.contains(&EvidenceTopic::MemoryInfo));
    }

    #[test]
    fn test_network_routing() {
        let routing = route_evidence("what is my network status", None);
        assert!(routing
            .required_topics
            .contains(&EvidenceTopic::NetworkStatus));
    }

    #[test]
    fn test_audio_routing() {
        let routing = route_evidence("is my audio working", None);
        assert!(routing
            .required_topics
            .contains(&EvidenceTopic::AudioStatus));
    }

    #[test]
    fn test_diagnostic_detection() {
        let routing = route_evidence("my disk is full", None);
        assert!(routing.is_diagnostic);

        let routing = route_evidence("how much disk space do I have", None);
        assert!(!routing.is_diagnostic);
    }

    #[test]
    fn test_tool_blacklist() {
        // hw_snapshot_summary should NOT satisfy DiskFree
        assert!(!tool_satisfies_topic(
            "hw_snapshot_summary",
            EvidenceTopic::DiskFree
        ));
        assert!(!tool_satisfies_topic(
            "sw_snapshot_summary",
            EvidenceTopic::KernelVersion
        ));

        // Correct tools should satisfy their topics
        assert!(tool_satisfies_topic("mount_usage", EvidenceTopic::DiskFree));
        assert!(tool_satisfies_topic(
            "kernel_version",
            EvidenceTopic::KernelVersion
        ));
        assert!(tool_satisfies_topic(
            "memory_info",
            EvidenceTopic::MemoryInfo
        ));
    }

    #[test]
    fn test_service_extraction() {
        assert_eq!(
            extract_service_name("is docker running"),
            Some("docker".to_string())
        );
        assert_eq!(
            extract_service_name("is nginx active"),
            Some("nginx".to_string())
        );
        assert_eq!(
            extract_service_name("status of sshd"),
            Some("sshd".to_string())
        );
        assert_eq!(extract_service_name("hello world"), None);
    }
}
