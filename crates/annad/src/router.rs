//! Deterministic router - overrides LLM translator for known query classes.
//!
//! Ensures reliable routing and probe selection for common queries,
//! regardless of LLM translator behavior.

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use tracing::info;

/// Known query classes with deterministic routing
#[derive(Debug, Clone, PartialEq)]
pub enum QueryClass {
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
    /// Unknown: defer to LLM translator
    Unknown,
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

/// Classify query to a known class
pub fn classify_query(query: &str) -> QueryClass {
    let q = query.to_lowercase();

    // Help request (check first as it's specific)
    if q.trim() == "help" || q.contains("what can you do") || q.contains("how do i use") {
        return QueryClass::Help;
    }

    // System slow (multi-probe diagnostic)
    if q.contains("slow") || q.contains("sluggish") || q.contains("laggy") {
        return QueryClass::SystemSlow;
    }

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

    QueryClass::Unknown
}

/// Get deterministic route for a query
pub fn get_route(query: &str) -> DeterministicRoute {
    let class = classify_query(query);

    match class {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_cpu() {
        assert_eq!(classify_query("what cpu do i have?"), QueryClass::CpuInfo);
        assert_eq!(classify_query("show processor info"), QueryClass::CpuInfo);
        assert_eq!(classify_query("how many cores"), QueryClass::CpuInfo);
    }

    #[test]
    fn test_classify_memory() {
        assert_eq!(classify_query("how much ram"), QueryClass::RamInfo);
        assert_eq!(classify_query("show memory"), QueryClass::RamInfo);
        // Process memory should be different
        assert_eq!(
            classify_query("processes using memory"),
            QueryClass::TopMemoryProcesses
        );
    }

    #[test]
    fn test_classify_processes() {
        assert_eq!(
            classify_query("top 5 memory hogs"),
            QueryClass::TopMemoryProcesses
        );
        assert_eq!(
            classify_query("what's using cpu"),
            QueryClass::TopCpuProcesses
        );
    }

    #[test]
    fn test_classify_disk() {
        assert_eq!(classify_query("disk space free"), QueryClass::DiskSpace);
        assert_eq!(classify_query("is storage full"), QueryClass::DiskSpace);
    }

    #[test]
    fn test_classify_network() {
        assert_eq!(
            classify_query("show network interfaces"),
            QueryClass::NetworkInterfaces
        );
        assert_eq!(classify_query("wifi status"), QueryClass::NetworkInterfaces);
    }

    #[test]
    fn test_classify_help() {
        assert_eq!(classify_query("help"), QueryClass::Help);
        assert_eq!(classify_query("what can you do"), QueryClass::Help);
    }

    #[test]
    fn test_classify_slow() {
        assert_eq!(classify_query("it's slow"), QueryClass::SystemSlow);
        assert_eq!(classify_query("system is sluggish"), QueryClass::SystemSlow);
    }

    #[test]
    fn test_route_probes() {
        let route = get_route("top memory processes");
        assert_eq!(route.probes, vec!["top_memory"]);

        let route = get_route("disk space");
        assert_eq!(route.probes, vec!["disk_usage"]);

        let route = get_route("it's slow");
        assert!(route.probes.contains(&"top_cpu".to_string()));
        assert!(route.probes.contains(&"top_memory".to_string()));
    }

    #[test]
    fn test_apply_routing_overrides_llm() {
        let llm_ticket = TranslatorTicket {
            intent: QueryIntent::Request, // LLM got it wrong
            domain: SpecialistDomain::Security, // LLM got it wrong
            entities: vec![],
            needs_probes: vec!["listening_ports".to_string()], // Wrong probes
            clarification_question: None,
            confidence: 0.9,
        };

        let ticket = apply_deterministic_routing("show disk space", Some(llm_ticket));

        // Deterministic should override
        assert_eq!(ticket.domain, SpecialistDomain::Storage);
        assert_eq!(ticket.intent, QueryIntent::Question);
        assert_eq!(ticket.needs_probes, vec!["disk_usage"]);
        assert_eq!(ticket.confidence, 1.0);
    }
}
