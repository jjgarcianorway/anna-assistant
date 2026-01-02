//! Primitive library tests.
//!
//! Tests for probe primitives, library coverage, and selection logic.

#[cfg(test)]
mod tests {
    use crate::evidence_first::{
        primitives::{Domain, PrimitiveLibrary},
        probe_plan::ProbePlan,
    };

    /// Test 2: CPU temperature check.
    #[test]
    fn test_cpu_temperature_check() {
        let library = PrimitiveLibrary::new();

        // Verify temperature probe exists
        let temp_probe = library.get("hw.cpu.temp");
        assert!(temp_probe.is_some(), "hw.cpu.temp should exist");

        // Verify it's in hardware domain
        let hw_probes = library.for_domain(Domain::Hardware);
        assert!(
            hw_probes.iter().any(|p| p.id == "hw.cpu.temp"),
            "Should be in Hardware domain"
        );

        // Test keyword search
        let temp_probes = library.find_by_keyword("temperature");
        assert!(!temp_probes.is_empty(), "Should find temperature probes");
    }

    /// Test 6: Primitive library coverage.
    #[test]
    fn test_primitive_library_coverage() {
        let library = PrimitiveLibrary::new();

        // Should have probes for all major domains
        let domains = [
            Domain::Boot,
            Domain::Services,
            Domain::Logs,
            Domain::Memory,
            Domain::Disk,
            Domain::Network,
            Domain::Hardware,
        ];

        for domain in domains {
            let probes = library.for_domain(domain);
            assert!(!probes.is_empty(), "Should have probes for {:?}", domain);
        }

        // Verify key probes exist
        let key_probes = [
            "sys.boot.analyze",
            "sys.boot.blame",
            "sys.services.failed",
            "sys.logs.errors",
            "sys.mem.free",
            "sys.disk.df",
            "net.ip.addr",
        ];

        for probe_id in key_probes {
            assert!(library.get(probe_id).is_some(), "Should have {}", probe_id);
        }
    }

    /// Test 10: Probe selection by domain and keywords.
    #[test]
    fn test_probe_selection() {
        let library = PrimitiveLibrary::new();
        let mut plan = ProbePlan::new("test");

        // Select by domain
        plan.select_for_domain(Domain::Boot, &library);
        assert!(!plan.is_empty(), "Should select boot probes");

        // Select by keywords
        let mut plan2 = ProbePlan::new("test2");
        plan2.select_from_keywords(&["memory", "ram"], &library);
        assert!(!plan2.is_empty(), "Should select memory probes");
    }
}
