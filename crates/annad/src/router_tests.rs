//! Unit tests for router module.
//!
//! Note: Corpus-driven tests are in tests/router_corpus_tests.rs

#[cfg(test)]
mod tests {
    use crate::router::*;
    use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};

    #[test]
    fn test_classify_cpu() {
        assert_eq!(classify_query("what cpu do i have?"), QueryClass::CpuInfo);
        assert_eq!(classify_query("show processor info"), QueryClass::CpuInfo);
        // v0.0.45: "how many cores" now routes to CpuCores for specific probe
        assert_eq!(classify_query("how many cores"), QueryClass::CpuCores);
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

        // v0.45.x: probes use command names
        let route = get_route("disk space");
        assert_eq!(route.probes, vec!["df"]);

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

        // Deterministic should override - v0.45.x uses command names
        assert_eq!(ticket.domain, SpecialistDomain::Storage);
        assert_eq!(ticket.intent, QueryIntent::Question);
        assert_eq!(ticket.needs_probes, vec!["df"]);
        assert_eq!(ticket.confidence, 1.0);
    }

    // === ROUTE Phase: New typed query classes ===

    #[test]
    fn test_classify_memory_usage() {
        assert_eq!(classify_query("memory usage"), QueryClass::MemoryUsage);
        assert_eq!(classify_query("how much memory used"), QueryClass::MemoryUsage);
        assert_eq!(classify_query("free memory"), QueryClass::MemoryUsage);
        assert_eq!(classify_query("available memory"), QueryClass::MemoryUsage);
    }

    #[test]
    fn test_classify_disk_usage() {
        assert_eq!(classify_query("disk usage"), QueryClass::DiskUsage);
        assert_eq!(classify_query("filesystem usage"), QueryClass::DiskUsage);
    }

    #[test]
    fn test_classify_service_status() {
        assert_eq!(classify_query("is nginx running"), QueryClass::ServiceStatus);
        assert_eq!(classify_query("service status"), QueryClass::ServiceStatus);
        assert_eq!(classify_query("systemd units"), QueryClass::ServiceStatus);
    }

    #[test]
    fn test_route_typed_probes() {
        let route = get_route("memory usage");
        assert_eq!(route.probes, vec!["free"]);
        assert!(route.can_answer_deterministically());

        let route = get_route("disk usage");
        assert_eq!(route.probes, vec!["df"]);
        assert!(route.can_answer_deterministically());

        let route = get_route("is nginx running");
        assert_eq!(route.probes, vec!["systemctl"]);
        assert!(route.can_answer_deterministically());
    }
}
