//! Deterministic router - overrides LLM translator for known query classes.
//!
//! Ensures reliable routing and probe selection for common queries,
//! regardless of LLM translator behavior.
//!
//! ROUTE phase: Uses typed outputs from STRUCT-lite parsers.

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use serde::{Deserialize, Serialize};
use tracing::info;

// Re-export classify_query from the patterns module
pub use crate::query_classify::classify_query;

/// Known query classes with deterministic routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryClass {
    /// System triage: "any errors", "any problems", "warnings" => fast path, no specialist
    SystemTriage,
    /// CPU info: cpu/processor => domain=system, probes=[]
    CpuInfo,
    /// CPU cores: "how many cores" => domain=system, probes=[lscpu] (v0.0.45)
    CpuCores,
    /// CPU temperature: "cpu temp" => domain=system, probes=[sensors] (v0.0.45)
    CpuTemp,
    /// RAM info: ram/memory (not process) => domain=system, probes=[]
    RamInfo,
    /// GPU info: gpu/graphics/vram => domain=system, probes=[]
    GpuInfo,
    /// Hardware audio: "sound card", "audio device" => probes=[lspci_audio] (v0.0.45)
    HardwareAudio,
    /// Memory processes: processes using memory => probes=[top_memory]
    TopMemoryProcesses,
    /// CPU processes: processes using cpu => probes=[top_cpu]
    TopCpuProcesses,
    /// Disk space: disk/storage/filesystem => probes=[disk_usage]
    DiskSpace,
    /// Network interfaces: network/ip/wifi/ethernet => probes=[network_addrs]
    NetworkInterfaces,
    /// Help request: help/usage => deterministic response
    Help,
    /// System slow: slow/sluggish => probes=[top_cpu, top_memory, disk_usage]
    SystemSlow,
    /// Memory usage (total): "how much memory" => probes=[free]
    MemoryUsage,
    /// Memory free/available: "free ram" => probes=[free] (v0.0.45 - distinct from total)
    MemoryFree,
    /// Disk usage: "disk usage" => probes=[df]
    DiskUsage,
    /// Service status: "is nginx running" => probes=[systemctl]
    ServiceStatus,
    /// System health summary: "health", "summary" => full system overview
    SystemHealthSummary,
    /// Boot time status: from knowledge store
    BootTimeStatus,
    /// Installed packages overview: from knowledge store
    InstalledPackagesOverview,
    /// Package count: "how many packages" => probes=[pacman_count] (v0.0.45)
    PackageCount,
    /// Installed tool check: "do I have nano" => probes=[command_v] (v0.0.45)
    InstalledToolCheck,
    /// App alternatives: from knowledge store
    AppAlternatives,
    /// Unknown: defer to LLM translator
    Unknown,
}

impl std::fmt::Display for QueryClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SystemTriage => "system_triage",
            Self::CpuInfo => "cpu_info",
            Self::CpuCores => "cpu_cores",
            Self::CpuTemp => "cpu_temp",
            Self::RamInfo => "ram_info",
            Self::GpuInfo => "gpu_info",
            Self::HardwareAudio => "hardware_audio",
            Self::TopMemoryProcesses => "top_memory_processes",
            Self::TopCpuProcesses => "top_cpu_processes",
            Self::DiskSpace => "disk_space",
            Self::NetworkInterfaces => "network_interfaces",
            Self::Help => "help",
            Self::SystemSlow => "system_slow",
            Self::MemoryUsage => "memory_usage",
            Self::MemoryFree => "memory_free",
            Self::DiskUsage => "disk_usage",
            Self::ServiceStatus => "service_status",
            Self::SystemHealthSummary => "system_health_summary",
            Self::BootTimeStatus => "boot_time_status",
            Self::InstalledPackagesOverview => "installed_packages_overview",
            Self::PackageCount => "package_count",
            Self::InstalledToolCheck => "installed_tool_check",
            Self::AppAlternatives => "app_alternatives",
            Self::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

impl QueryClass {
    /// Parse from string (for corpus tests)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "system_triage" => Some(Self::SystemTriage),
            "cpu_info" => Some(Self::CpuInfo),
            "cpu_cores" => Some(Self::CpuCores),
            "cpu_temp" => Some(Self::CpuTemp),
            "ram_info" => Some(Self::RamInfo),
            "gpu_info" => Some(Self::GpuInfo),
            "hardware_audio" => Some(Self::HardwareAudio),
            "top_memory_processes" => Some(Self::TopMemoryProcesses),
            "top_cpu_processes" => Some(Self::TopCpuProcesses),
            "disk_space" => Some(Self::DiskSpace),
            "network_interfaces" => Some(Self::NetworkInterfaces),
            "help" => Some(Self::Help),
            "system_slow" => Some(Self::SystemSlow),
            "memory_usage" => Some(Self::MemoryUsage),
            "memory_free" => Some(Self::MemoryFree),
            "disk_usage" => Some(Self::DiskUsage),
            "service_status" => Some(Self::ServiceStatus),
            "system_health_summary" => Some(Self::SystemHealthSummary),
            "boot_time_status" => Some(Self::BootTimeStatus),
            "installed_packages_overview" => Some(Self::InstalledPackagesOverview),
            "package_count" => Some(Self::PackageCount),
            "installed_tool_check" => Some(Self::InstalledToolCheck),
            "app_alternatives" => Some(Self::AppAlternatives),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Check if this class is RAG-first (answered from knowledge store)
    pub fn is_rag_first(&self) -> bool {
        matches!(self, Self::BootTimeStatus | Self::InstalledPackagesOverview | Self::AppAlternatives)
    }

    /// Check if this class is a fast-path (skip translator, no specialist)
    pub fn is_fast_path(&self) -> bool {
        matches!(self, Self::SystemTriage | Self::Help)
    }
}

/// Route result from deterministic router
#[derive(Debug, Clone)]
pub struct DeterministicRoute {
    pub class: QueryClass,
    pub domain: SpecialistDomain,
    pub intent: QueryIntent,
    pub probes: Vec<String>,
    pub can_answer_deterministically: bool,
}

/// Get deterministic route for a query
pub fn get_route(query: &str) -> DeterministicRoute {
    let class = classify_query(query);

    match class {
        // FAST PATH: SystemTriage - errors/warnings only, no specialist needed
        QueryClass::SystemTriage => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![
                "journal_errors".to_string(),
                "journal_warnings".to_string(),
                "failed_units".to_string(),
                "boot_time".to_string(),
            ],
            can_answer_deterministically: true,
        },
        QueryClass::CpuInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: true,
        },
        QueryClass::CpuCores => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lscpu".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::CpuTemp => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["sensors".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::RamInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: true,
        },
        QueryClass::GpuInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: true,
        },
        QueryClass::HardwareAudio => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["lspci_audio".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::TopMemoryProcesses => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["top_memory".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::TopCpuProcesses => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["top_cpu".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::DiskSpace => DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["disk_usage".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::NetworkInterfaces => DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["network_addrs".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::Help => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: true,
        },
        QueryClass::SystemSlow => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Investigate,
            probes: vec![
                "top_cpu".to_string(),
                "top_memory".to_string(),
                "disk_usage".to_string(),
            ],
            can_answer_deterministically: false,
        },
        QueryClass::MemoryUsage => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["free".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::MemoryFree => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["free".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::DiskUsage => DeterministicRoute {
            class,
            domain: SpecialistDomain::Storage,
            intent: QueryIntent::Question,
            probes: vec!["df".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::ServiceStatus => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["systemctl".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::SystemHealthSummary => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![
                "disk_usage".to_string(),
                "memory_info".to_string(),
                "failed_services".to_string(),
                "top_cpu".to_string(),
            ],
            can_answer_deterministically: true,
        },
        QueryClass::BootTimeStatus => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: true,
        },
        QueryClass::InstalledPackagesOverview => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: true,
        },
        QueryClass::PackageCount => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec!["pacman_count".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::InstalledToolCheck => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["command_v".to_string()],
            can_answer_deterministically: true,
        },
        QueryClass::AppAlternatives => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: true,
        },
        QueryClass::Unknown => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![],
            can_answer_deterministically: false,
        },
    }
}

/// Apply deterministic router, overriding LLM ticket for known classes
pub fn apply_deterministic_routing(query: &str, llm_ticket: Option<TranslatorTicket>) -> TranslatorTicket {
    let route = get_route(query);

    if route.class == QueryClass::Unknown {
        return llm_ticket.unwrap_or_else(|| create_default_ticket(route));
    }

    info!(
        "Deterministic router: class={:?}, domain={}, probes={:?}",
        route.class, route.domain, route.probes
    );

    TranslatorTicket {
        intent: route.intent,
        domain: route.domain,
        entities: vec![],
        needs_probes: route.probes,
        clarification_question: None,
        confidence: 1.0,
    }
}

/// Create default ticket from route
fn create_default_ticket(route: DeterministicRoute) -> TranslatorTicket {
    TranslatorTicket {
        intent: route.intent,
        domain: route.domain,
        entities: vec![],
        needs_probes: route.probes,
        clarification_question: None,
        confidence: 0.5,
    }
}

/// Check if query class can be answered deterministically
#[allow(dead_code)]
pub fn can_answer_deterministically(query: &str) -> bool {
    get_route(query).can_answer_deterministically
}
