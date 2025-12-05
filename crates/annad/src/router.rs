//! Deterministic router - overrides LLM translator for known query classes.
//!
//! Ensures reliable routing and probe selection for common queries,
//! regardless of LLM translator behavior.
//!
//! ROUTE phase: Uses typed outputs from STRUCT-lite parsers.

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Known query classes with deterministic routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryClass {
    /// System triage: "any errors", "any problems", "warnings" => fast path, no specialist
    /// probes=[journal_errors, failed_units] - errors/warnings only, not full report
    SystemTriage,
    /// CPU info: cpu/processor/cores => domain=system, probes=[]
    CpuInfo,
    /// RAM info: ram/memory (not process) => domain=system, probes=[]
    RamInfo,
    /// GPU info: gpu/graphics/vram => domain=system, probes=[]
    GpuInfo,
    /// Memory processes: processes using memory => domain=system, probes=[top_memory]
    TopMemoryProcesses,
    /// CPU processes: processes using cpu => domain=system, probes=[top_cpu]
    TopCpuProcesses,
    /// Disk space: disk/storage/filesystem => domain=storage, probes=[disk_usage]
    DiskSpace,
    /// Network interfaces: network/ip/wifi/ethernet => domain=network, probes=[network_addrs]
    NetworkInterfaces,
    /// Help request: help/usage => domain=system, probes=[], deterministic response
    Help,
    /// System slow: slow/sluggish => domain=system, probes=[top_cpu, top_memory, disk_usage]
    SystemSlow,
    /// Memory usage: "how much memory" => domain=system, probes=[free]
    MemoryUsage,
    /// Disk usage: "disk usage" => domain=storage, probes=[df]
    DiskUsage,
    /// Service status: "is nginx running" => domain=system, probes=[systemctl]
    ServiceStatus,
    /// System health summary: "health", "summary" => full system overview
    SystemHealthSummary,
    /// Boot time status: from knowledge store
    BootTimeStatus,
    /// Installed packages overview: from knowledge store
    InstalledPackagesOverview,
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
            Self::RamInfo => "ram_info",
            Self::GpuInfo => "gpu_info",
            Self::TopMemoryProcesses => "top_memory_processes",
            Self::TopCpuProcesses => "top_cpu_processes",
            Self::DiskSpace => "disk_space",
            Self::NetworkInterfaces => "network_interfaces",
            Self::Help => "help",
            Self::SystemSlow => "system_slow",
            Self::MemoryUsage => "memory_usage",
            Self::DiskUsage => "disk_usage",
            Self::ServiceStatus => "service_status",
            Self::SystemHealthSummary => "system_health_summary",
            Self::BootTimeStatus => "boot_time_status",
            Self::InstalledPackagesOverview => "installed_packages_overview",
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
            "ram_info" => Some(Self::RamInfo),
            "gpu_info" => Some(Self::GpuInfo),
            "top_memory_processes" => Some(Self::TopMemoryProcesses),
            "top_cpu_processes" => Some(Self::TopCpuProcesses),
            "disk_space" => Some(Self::DiskSpace),
            "network_interfaces" => Some(Self::NetworkInterfaces),
            "help" => Some(Self::Help),
            "system_slow" => Some(Self::SystemSlow),
            "memory_usage" => Some(Self::MemoryUsage),
            "disk_usage" => Some(Self::DiskUsage),
            "service_status" => Some(Self::ServiceStatus),
            "system_health_summary" => Some(Self::SystemHealthSummary),
            "boot_time_status" => Some(Self::BootTimeStatus),
            "installed_packages_overview" => Some(Self::InstalledPackagesOverview),
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

/// Strip common greetings from query for better classification
fn strip_greetings(query: &str) -> String {
    let q = query.to_lowercase();
    // Remove common greetings and emoticons
    let patterns = [
        "hello", "hi ", "hey ", "good morning", "good afternoon", "good evening",
        "anna", ":)", ":(", ";)", ":d", ":p", "!", "?", "…", "...",
    ];
    let mut result = q;
    for p in patterns {
        result = result.replace(p, " ");
    }
    // Collapse multiple spaces
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Classify query to a known class
pub fn classify_query(query: &str) -> QueryClass {
    let q = query.to_lowercase();
    let stripped = strip_greetings(query);

    // Help request (check first as it's specific)
    if q.trim() == "help" || q.contains("what can you do") || q.contains("how do i use") {
        return QueryClass::Help;
    }

    // SystemTriage (FAST PATH): error/warning focused queries - check BEFORE SystemHealthSummary
    // These get fast probes: journal_errors, failed_units only
    // Pattern: "any X?", "is everything ok?", "how is my computer?"
    if stripped.contains("any errors")
        || stripped.contains("any problems")
        || stripped.contains("any issues")
        || stripped.contains("any warnings")
        || stripped.contains("errors so far")
        || stripped.contains("problems so far")
        || stripped.contains("what's wrong")
        || stripped.contains("whats wrong")
        || stripped.contains("is everything ok")
        || stripped.contains("is everything okay")
        || stripped.contains("how is my computer")
        || stripped.contains("how's my computer")
        || stripped.contains("computer doing")
        || q.trim() == "errors"
        || q.trim() == "warnings"
        || q.trim() == "problems"
    {
        return QueryClass::SystemTriage;
    }

    // System health summary: full system overview (health, summary, status report)
    if q.contains("health")
        || q.contains("summary")
        || q.contains("status report")
        || q.contains("overview")
        || q.contains("system status")
        || stripped.contains("how is the system")
        || stripped.contains("how's the system")
        || stripped.contains("check my system")
        || stripped.contains("check the system")
        || stripped.contains("system check")
        || stripped.contains("how am i doing")
        || stripped.contains("how are things")
        || q.trim() == "status"
        || q.trim() == "report"
    {
        return QueryClass::SystemHealthSummary;
    }

    // System slow (multi-probe diagnostic)
    if q.contains("slow") || q.contains("sluggish") || q.contains("laggy") {
        return QueryClass::SystemSlow;
    }

    // === ROUTE Phase: Typed query classes (check specific patterns first) ===

    // Memory usage (dynamic): "memory usage", "how much memory used"
    // Check before RamInfo since these are more specific
    if (q.contains("memory") && q.contains("usage"))
        || (q.contains("memory") && q.contains("used"))
        || q.contains("free memory")
        || q.contains("available memory")
    {
        return QueryClass::MemoryUsage;
    }

    // Disk usage (dynamic): specific mount or usage patterns
    // Check before DiskSpace since "disk usage" is more specific
    if q.contains("disk usage") || q.contains("filesystem usage") {
        return QueryClass::DiskUsage;
    }

    // Service status: "is X running", "status of X"
    if q.contains("running")
        || q.contains("service status")
        || q.contains("systemd")
        || (q.contains("status") && q.contains("service"))
        || (q.contains("is") && (q.contains("active") || q.contains("enabled")))
    {
        return QueryClass::ServiceStatus;
    }

    // === Legacy query classes ===

    // Top memory processes (before RAM check)
    if (q.contains("process") && (q.contains("memory") || q.contains("ram")))
        || q.contains("memory hog")
        || q.contains("top memory")
        || q.contains("most memory")
        || q.contains("what's using memory")
        || q.contains("what is using memory")
    {
        return QueryClass::TopMemoryProcesses;
    }

    // Top CPU processes
    if (q.contains("process") && q.contains("cpu"))
        || q.contains("cpu hog")
        || q.contains("top cpu")
        || q.contains("most cpu")
        || q.contains("what's using cpu")
        || q.contains("what is using cpu")
    {
        return QueryClass::TopCpuProcesses;
    }

    // Hardware snapshot queries
    if q.contains("cpu") || q.contains("processor") || q.contains("core") {
        return QueryClass::CpuInfo;
    }

    if q.contains("ram") || (q.contains("memory") && !q.contains("process")) {
        return QueryClass::RamInfo;
    }

    if q.contains("gpu") || q.contains("graphics") || q.contains("vram") {
        return QueryClass::GpuInfo;
    }

    // Disk space
    if q.contains("disk")
        || q.contains("space")
        || q.contains("storage")
        || q.contains("filesystem")
        || q.contains("mount")
        || q.contains("full")
    {
        return QueryClass::DiskSpace;
    }

    // Network interfaces
    if q.contains("network")
        || q.contains("interface")
        || q.contains("ip ")
        || q.contains("ip?")
        || q.contains("ips")
        || q.contains("wifi")
        || q.contains("ethernet")
        || q.contains("wlan")
    {
        return QueryClass::NetworkInterfaces;
    }

    // === RAG-first classes (v0.0.32R): answered from knowledge store ===

    // Boot time status
    if q.contains("boot time")
        || q.contains("bootup")
        || q.contains("startup time")
        || q.contains("how long to boot")
        || q.contains("how fast does it boot")
        || (q.contains("boot") && q.contains("seconds"))
    {
        return QueryClass::BootTimeStatus;
    }

    // Installed packages overview
    if q.contains("how many packages")
        || q.contains("packages installed")
        || q.contains("what's installed")
        || q.contains("what is installed")
        || q.contains("list packages")
        || q.contains("installed software")
        || (q.contains("packages") && q.contains("count"))
    {
        return QueryClass::InstalledPackagesOverview;
    }

    // App alternatives
    if q.contains("alternative to")
        || q.contains("alternatives to")
        || q.contains("instead of")
        || q.contains("replacement for")
        || q.contains("similar to")
        || q.contains("like")
        || (q.contains("what") && q.contains("use") && q.contains("instead"))
    {
        return QueryClass::AppAlternatives;
    }

    QueryClass::Unknown
}

/// Get deterministic route for a query
pub fn get_route(query: &str) -> DeterministicRoute {
    let class = classify_query(query);

    match class {
        // FAST PATH: SystemTriage - errors/warnings only, no specialist needed
        QueryClass::SystemTriage => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question, // Not Investigate - we don't need specialist
            probes: vec![
                "journal_errors".to_string(),   // journalctl -p 3 -b (err+)
                "journal_warnings".to_string(), // journalctl -p 4 -b (warn)
                "failed_units".to_string(),     // systemctl --failed
            ],
            can_answer_deterministically: true,
        },
        QueryClass::CpuInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![], // Use hardware snapshot
            can_answer_deterministically: true,
        },
        QueryClass::RamInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![], // Use hardware snapshot
            can_answer_deterministically: true,
        },
        QueryClass::GpuInfo => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![], // Use hardware snapshot
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
            can_answer_deterministically: false, // Needs analysis
        },
        // === ROUTE Phase: Typed query classes ===
        QueryClass::MemoryUsage => DeterministicRoute {
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
            // v0.0.32: Use health brief probes (relevant-only, not full report)
            probes: vec![
                "disk_usage".to_string(),     // df -h: Disk warnings
                "memory_info".to_string(),    // free -h: Memory pressure
                "failed_services".to_string(), // systemctl --failed: Failed services
                "top_cpu".to_string(),        // ps aux: High CPU processes
            ],
            can_answer_deterministically: true,
        },
        // === RAG-first classes (v0.0.32R): answered from knowledge store ===
        QueryClass::BootTimeStatus => DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![], // Answered from knowledge store, may trigger collector
            can_answer_deterministically: true,
        },
        QueryClass::InstalledPackagesOverview => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec![], // Answered from knowledge store, may trigger collector
            can_answer_deterministically: true,
        },
        QueryClass::AppAlternatives => DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec![], // Answered from knowledge store (cached wiki/aur data)
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
        // Use LLM ticket if available, otherwise create default
        return llm_ticket.unwrap_or_else(|| create_default_ticket(route));
    }

    // For known classes, always use deterministic routing
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
        confidence: 1.0, // Deterministic = full confidence
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
        confidence: 0.5, // Unknown class = lower confidence
    }
}

/// Check if query class can be answered deterministically
#[allow(dead_code)]
pub fn can_answer_deterministically(query: &str) -> bool {
    get_route(query).can_answer_deterministically
}
