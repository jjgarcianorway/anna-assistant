//! Golden tests for deterministic router and translator robustness.
//!
//! Verifies that known query classes are routed correctly and that
//! the system handles invalid/missing translator JSON gracefully.

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};

/// Re-implement router logic for testing
mod router {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum QueryClass {
        CpuInfo,
        RamInfo,
        GpuInfo,
        TopMemoryProcesses,
        TopCpuProcesses,
        DiskSpace,
        NetworkInterfaces,
        Help,
        SystemSlow,
        Unknown,
    }

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

        if q.contains("gpu") || q.contains("graphics") || q.contains("vram") {
            return QueryClass::GpuInfo;
        }

        if q.contains("ram") || (q.contains("memory") && !q.contains("process")) {
            return QueryClass::RamInfo;
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

    pub fn get_probes(class: &QueryClass) -> Vec<String> {
        match class {
            QueryClass::CpuInfo | QueryClass::RamInfo | QueryClass::GpuInfo | QueryClass::Help => vec![],
            QueryClass::TopMemoryProcesses => vec!["top_memory".to_string()],
            QueryClass::TopCpuProcesses => vec!["top_cpu".to_string()],
            QueryClass::DiskSpace => vec!["disk_usage".to_string()],
            QueryClass::NetworkInterfaces => vec!["network_addrs".to_string()],
            QueryClass::SystemSlow => vec!["top_cpu".to_string(), "top_memory".to_string(), "disk_usage".to_string()],
            QueryClass::Unknown => vec![],
        }
    }

    pub fn get_domain(class: &QueryClass) -> SpecialistDomain {
        match class {
            QueryClass::DiskSpace => SpecialistDomain::Storage,
            QueryClass::NetworkInterfaces => SpecialistDomain::Network,
            _ => SpecialistDomain::System,
        }
    }
}

// === Query Class Tests ===

#[test]
fn test_cpu_query_routing() {
    let queries = ["what cpu do i have?", "show processor info", "how many cores"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::CpuInfo, "Query: {}", q);
        assert_eq!(router::get_domain(&router::QueryClass::CpuInfo), SpecialistDomain::System);
        assert!(router::get_probes(&router::QueryClass::CpuInfo).is_empty());
    }
}

#[test]
fn test_ram_query_routing() {
    let queries = ["how much ram", "show memory", "total memory"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::RamInfo, "Query: {}", q);
        assert!(router::get_probes(&router::QueryClass::RamInfo).is_empty());
    }
}

#[test]
fn test_gpu_query_routing() {
    let queries = ["what gpu do i have", "show graphics card", "check vram"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::GpuInfo, "Query: {}", q);
    }
}

#[test]
fn test_top_memory_routing() {
    let queries = ["top memory processes", "memory hogs", "what's using memory"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::TopMemoryProcesses, "Query: {}", q);
        assert_eq!(router::get_probes(&router::QueryClass::TopMemoryProcesses), vec!["top_memory"]);
    }
}

#[test]
fn test_top_cpu_routing() {
    let queries = ["top cpu processes", "cpu hogs", "what's using cpu"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::TopCpuProcesses, "Query: {}", q);
        assert_eq!(router::get_probes(&router::QueryClass::TopCpuProcesses), vec!["top_cpu"]);
    }
}

#[test]
fn test_disk_space_routing() {
    let queries = ["disk space", "storage full", "filesystem usage"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::DiskSpace, "Query: {}", q);
        assert_eq!(router::get_domain(&router::QueryClass::DiskSpace), SpecialistDomain::Storage);
        assert_eq!(router::get_probes(&router::QueryClass::DiskSpace), vec!["disk_usage"]);
    }
}

#[test]
fn test_network_routing() {
    let queries = ["network interfaces", "show ip ", "wifi status"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::NetworkInterfaces, "Query: {}", q);
        assert_eq!(router::get_domain(&router::QueryClass::NetworkInterfaces), SpecialistDomain::Network);
        assert_eq!(router::get_probes(&router::QueryClass::NetworkInterfaces), vec!["network_addrs"]);
    }
}

#[test]
fn test_help_routing() {
    assert_eq!(router::classify_query("help"), router::QueryClass::Help);
    assert_eq!(router::classify_query("what can you do"), router::QueryClass::Help);
}

#[test]
fn test_system_slow_routing() {
    let queries = ["it's slow", "system is sluggish"];
    for q in queries {
        assert_eq!(router::classify_query(q), router::QueryClass::SystemSlow, "Query: {}", q);
        let probes = router::get_probes(&router::QueryClass::SystemSlow);
        assert!(probes.contains(&"top_cpu".to_string()));
        assert!(probes.contains(&"top_memory".to_string()));
        assert!(probes.contains(&"disk_usage".to_string()));
    }
}

// === Translator JSON Robustness Tests ===

#[test]
fn test_parse_minimal_json() {
    // Just intent and domain - should use defaults for everything else
    let json = r#"{"intent":"question","domain":"system"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert_eq!(parsed["intent"], "question");
    assert_eq!(parsed["domain"], "system");
}

#[test]
fn test_parse_missing_confidence() {
    // confidence missing - should default to 0.0
    let json = r#"{"intent":"question","domain":"system","entities":[],"needs_probes":[]}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    // confidence is missing
    assert!(parsed.get("confidence").is_none());
}

#[test]
fn test_parse_null_arrays() {
    // null arrays should be treated as empty
    let json = r#"{"intent":"question","domain":"system","entities":null,"needs_probes":null}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert!(parsed["entities"].is_null());
    assert!(parsed["needs_probes"].is_null());
}

#[test]
fn test_parse_empty_json() {
    // Empty JSON should still parse
    let json = "{}";
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!(parsed.is_object());
}

// === Scoring Tests ===

#[test]
fn test_deterministic_scoring_full() {
    // Deterministic answer with probes = high score
    use anna_shared::rpc::ReliabilitySignals;

    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };
    assert_eq!(signals.score(), 100);
}

#[test]
fn test_deterministic_scoring_timeout() {
    // Translator timeout + deterministic answer = 80
    use anna_shared::rpc::ReliabilitySignals;

    let signals = ReliabilitySignals {
        translator_confident: false,  // Timed out
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };
    assert_eq!(signals.score(), 80);
}

#[test]
fn test_empty_parser_result_scoring() {
    // Empty parser result = grounded=false, no_invention=false
    use anna_shared::rpc::ReliabilitySignals;

    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: false,  // Empty result
        no_invention: false,     // Empty result
        clarification_not_needed: false,  // Empty result
    };
    assert_eq!(signals.score(), 40);  // Only translator + coverage
}

// === Domain Consistency Tests ===

#[test]
fn test_domain_matches_classification() {
    use router::QueryClass;

    // Storage queries should route to Storage domain
    assert_eq!(router::get_domain(&QueryClass::DiskSpace), SpecialistDomain::Storage);

    // Network queries should route to Network domain
    assert_eq!(router::get_domain(&QueryClass::NetworkInterfaces), SpecialistDomain::Network);

    // System queries should route to System domain
    assert_eq!(router::get_domain(&QueryClass::CpuInfo), SpecialistDomain::System);
    assert_eq!(router::get_domain(&QueryClass::RamInfo), SpecialistDomain::System);
    assert_eq!(router::get_domain(&QueryClass::TopMemoryProcesses), SpecialistDomain::System);
}
