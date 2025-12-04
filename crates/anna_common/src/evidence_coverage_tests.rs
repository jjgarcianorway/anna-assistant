//! Integration Tests for Evidence Coverage v0.0.57
//!
//! Tests that the pipeline correctly routes queries to matching evidence tools
//! and that Junior properly penalizes mismatched/missing evidence.

#[cfg(test)]
mod tests {
    use crate::evidence_coverage::*;
    use crate::junior_rubric::*;
    use crate::system_query_router::*;
    use crate::tools::ToolResult;
    use std::time::{SystemTime, UNIX_EPOCH};

    // ========================================================================
    // Test Helpers
    // ========================================================================

    fn make_evidence(tool: &str, summary: &str, id: &str) -> ToolResult {
        ToolResult {
            tool_name: tool.to_string(),
            evidence_id: id.to_string(),
            data: serde_json::json!({}),
            human_summary: summary.to_string(),
            success: true,
            error: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    // ========================================================================
    // Disk Query Tests (the main bug fix)
    // ========================================================================

    #[test]
    fn test_disk_query_with_cpu_evidence_fails() {
        // This is THE BUG we're fixing: disk query routed to CPU evidence
        let (target, confidence) = detect_target("how much disk space is free");
        assert_eq!(target, QueryTarget::DiskFree);
        assert!(confidence >= 90);

        // Wrong evidence: CPU info instead of disk info
        let evidence = vec![
            make_evidence("hw_snapshot_cpu", "CPU: AMD Ryzen 7 5800X, 8 cores, 16 threads", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(!coverage.is_sufficient);
        assert!(coverage.coverage_percent < 50);

        let answer = "You have an AMD Ryzen 7 5800X [E1] with 8 cores.";
        let result = verify_answer(target, answer, &evidence);

        assert!(!result.ship_it);
        assert!(result.has_mismatch);
        assert!(result.reliability_score <= 20, "Wrong evidence should cap at 20%, got {}%", result.reliability_score);
    }

    #[test]
    fn test_disk_query_with_correct_evidence_passes() {
        let (target, _) = detect_target("how much disk space is free");
        assert_eq!(target, QueryTarget::DiskFree);

        // Correct evidence: disk usage info
        let evidence = vec![
            make_evidence("mount_usage", "Disk /: 433 GiB free of 500 GiB total (13% used)", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(coverage.is_sufficient, "Coverage should be sufficient: {:?}", coverage);

        let answer = "You have 433 GiB free on / [E1], out of 500 GiB total.";
        let result = verify_answer(target, answer, &evidence);

        assert!(result.ship_it, "Should ship: {:?}", result);
        assert!(result.reliability_score >= 75, "Reliability should be >= 75%, got {}%", result.reliability_score);
    }

    // ========================================================================
    // Memory Query Tests
    // ========================================================================

    #[test]
    fn test_memory_query_with_cpu_evidence_fails() {
        let (target, confidence) = detect_target("how much memory do i have");
        assert_eq!(target, QueryTarget::Memory);
        assert!(confidence >= 90);

        // Wrong evidence
        let evidence = vec![
            make_evidence("hw_snapshot_cpu", "CPU: AMD Ryzen 5 3600, 6 cores", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(!coverage.is_sufficient);

        let answer = "You have an AMD Ryzen 5 [E1] processor.";
        let result = verify_answer(target, answer, &evidence);

        assert!(!result.ship_it);
    }

    #[test]
    fn test_memory_query_with_correct_evidence_passes() {
        let (target, _) = detect_target("how much ram do i have");
        assert_eq!(target, QueryTarget::Memory);

        let evidence = vec![
            make_evidence("memory_info", "Memory: Total 32 GiB, Available 24.5 GiB, Used 7.5 GiB", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(coverage.is_sufficient);

        let answer = "You have 32 GiB of RAM [E1], with 24.5 GiB currently available.";
        let result = verify_answer(target, answer, &evidence);

        assert!(result.ship_it);
        assert!(result.reliability_score >= 85);
    }

    // ========================================================================
    // Kernel Query Tests
    // ========================================================================

    #[test]
    fn test_kernel_query_with_cpu_evidence_fails() {
        let (target, confidence) = detect_target("what kernel version am i using");
        assert_eq!(target, QueryTarget::KernelVersion);
        assert!(confidence >= 90);

        let evidence = vec![
            make_evidence("hw_snapshot_cpu", "CPU: Intel Core i7-12700K", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(!coverage.is_sufficient);

        let result = verify_answer(target, "You have an Intel Core i7 [E1].", &evidence);
        assert!(!result.ship_it);
    }

    #[test]
    fn test_kernel_query_with_correct_evidence_passes() {
        let (target, _) = detect_target("what linux kernel version");
        assert_eq!(target, QueryTarget::KernelVersion);

        let evidence = vec![
            make_evidence("kernel_version", "Linux 6.7.1-arch1-1 x86_64", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(coverage.is_sufficient);

        let answer = "You are running Linux kernel 6.7.1-arch1-1 [E1].";
        let result = verify_answer(target, answer, &evidence);

        assert!(result.ship_it);
        assert!(result.reliability_score >= 85);
    }

    // ========================================================================
    // Network Status Tests
    // ========================================================================

    #[test]
    fn test_network_query_with_cpu_evidence_fails() {
        let (target, confidence) = detect_target("what is my network status");
        assert_eq!(target, QueryTarget::NetworkStatus);
        assert!(confidence >= 90);

        let evidence = vec![
            make_evidence("hw_snapshot_cpu", "CPU: AMD Ryzen 9 5900X", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(!coverage.is_sufficient);

        let result = verify_answer(target, "CPU info [E1]", &evidence);
        assert!(!result.ship_it);
    }

    #[test]
    fn test_network_query_with_correct_evidence_passes() {
        let (target, _) = detect_target("am i connected to the network");

        let evidence = vec![
            make_evidence("network_status", "eth0: UP, IP 192.168.1.100, gateway 192.168.1.1, DNS 8.8.8.8", "E1"),
        ];

        let coverage = analyze_coverage(target, &evidence);
        assert!(coverage.is_sufficient, "Network coverage should be sufficient: {:?}", coverage);

        let answer = "You are connected via eth0 [E1] with IP 192.168.1.100. Default gateway is 192.168.1.1.";
        let result = verify_answer(target, answer, &evidence);

        assert!(result.ship_it, "Network answer should ship: {:?}", result);
    }

    // ========================================================================
    // Coverage Gap Detection Tests
    // ========================================================================

    #[test]
    fn test_gap_filling_suggests_correct_tools() {
        let evidence = vec![
            make_evidence("hw_snapshot_cpu", "CPU info only", "E1"),
        ];

        let coverage = analyze_coverage(QueryTarget::DiskFree, &evidence);
        let tools = get_gap_filling_tools(&coverage);

        assert!(!tools.is_empty(), "Should suggest tools for disk_free");
        assert!(
            tools.contains(&"mount_usage") || tools.contains(&"disk_usage"),
            "Should suggest mount_usage or disk_usage: {:?}",
            tools
        );
    }

    #[test]
    fn test_no_gap_filling_when_sufficient() {
        let evidence = vec![
            make_evidence("mount_usage", "Disk /: 433 GiB free of 500 GiB total", "E1"),
        ];

        let coverage = analyze_coverage(QueryTarget::DiskFree, &evidence);
        let tools = get_gap_filling_tools(&coverage);

        assert!(tools.is_empty(), "Should not suggest tools when coverage is sufficient");
    }

    // ========================================================================
    // Junior Scoring Tests
    // ========================================================================

    #[test]
    fn test_junior_caps_at_20_for_wrong_evidence() {
        let evidence = vec![
            make_evidence("hw_snapshot_cpu", "CPU: AMD Ryzen", "E1"),
        ];

        let result = verify_answer(QueryTarget::DiskFree, "AMD Ryzen [E1]", &evidence);
        assert!(result.reliability_score <= 20);
        assert!(result.has_mismatch);
    }

    #[test]
    fn test_junior_caps_at_50_for_missing_evidence() {
        // Some evidence but missing key fields
        let evidence = vec![
            make_evidence("status_snapshot", "System up for 5 days", "E1"),
        ];

        let coverage = analyze_coverage(QueryTarget::DiskFree, &evidence);
        assert!(coverage.coverage_percent < 50);

        let result = verify_answer(QueryTarget::DiskFree, "System status [E1]", &evidence);
        assert!(result.reliability_score <= 50);
    }

    #[test]
    fn test_junior_rewards_high_coverage() {
        let evidence = vec![
            make_evidence("mount_usage", "Disk /: 433 GiB free of 500 GiB total (13% used)", "E1"),
        ];

        let answer = "You have 433 GiB free on / [E1], out of 500 GiB total (13% used).";
        let result = verify_answer(QueryTarget::DiskFree, answer, &evidence);

        assert!(result.reliability_score >= 85, "High coverage should yield high score: {}%", result.reliability_score);
    }

    // ========================================================================
    // Tool Routing Tests
    // ========================================================================

    #[test]
    fn test_disk_query_routes_to_mount_usage() {
        let (target, _) = detect_target("how much space is free");
        let routing = get_tool_routing(target);

        assert!(
            routing.required_tools.contains(&"mount_usage"),
            "Disk queries should route to mount_usage: {:?}",
            routing.required_tools
        );
    }

    #[test]
    fn test_memory_query_routes_to_memory_info() {
        let (target, _) = detect_target("how much memory do i have");
        let routing = get_tool_routing(target);

        assert!(
            routing.required_tools.contains(&"memory_info"),
            "Memory queries should route to memory_info: {:?}",
            routing.required_tools
        );
    }

    #[test]
    fn test_kernel_query_routes_to_kernel_version() {
        let (target, _) = detect_target("what kernel version");
        let routing = get_tool_routing(target);

        assert!(
            routing.required_tools.contains(&"kernel_version"),
            "Kernel queries should route to kernel_version: {:?}",
            routing.required_tools
        );
    }

    #[test]
    fn test_network_query_routes_to_network_status() {
        let (target, _) = detect_target("what is my network status");
        let routing = get_tool_routing(target);

        assert!(
            routing.required_tools.contains(&"network_status"),
            "Network queries should route to network_status: {:?}",
            routing.required_tools
        );
    }

    // ========================================================================
    // Answer Validation Tests
    // ========================================================================

    #[test]
    fn test_answer_must_match_target() {
        // Disk query with CPU answer
        let (valid, critique) = validate_answer_for_target(
            QueryTarget::DiskFree,
            "CPU: AMD Ryzen 7, 8 cores, 16 threads"
        );
        assert!(!valid);
        assert!(critique.contains("CPU"));

        // Kernel query with CPU answer
        let (valid, critique) = validate_answer_for_target(
            QueryTarget::KernelVersion,
            "CPU: Intel i9, 12 cores"
        );
        assert!(!valid);
        assert!(critique.contains("kernel"));

        // Memory query with correct answer
        let (valid, _) = validate_answer_for_target(
            QueryTarget::Memory,
            "You have 32 GiB RAM available"
        );
        assert!(valid);
    }

    #[test]
    fn test_uncited_claims_penalized() {
        let evidence = vec![
            make_evidence("memory_info", "Memory: 32 GiB total", "E1"),
        ];

        // Answer without citation
        let result = verify_answer(QueryTarget::Memory, "You have 32 GiB of RAM.", &evidence);

        // Should have penalty for uncited claim
        assert!(
            result.penalties.iter().any(|p| p.reason.contains("citation")),
            "Should penalize uncited claims: {:?}",
            result.penalties
        );
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_evidence_fails() {
        let evidence: Vec<ToolResult> = vec![];

        let coverage = analyze_coverage(QueryTarget::DiskFree, &evidence);
        assert!(!coverage.is_sufficient);
        assert_eq!(coverage.coverage_percent, 0);

        let result = verify_answer(QueryTarget::DiskFree, "Some answer", &evidence);
        assert!(!result.ship_it);
        assert!(result.reliability_score < 50);
    }

    #[test]
    fn test_unknown_target_lenient() {
        let evidence = vec![
            make_evidence("status_snapshot", "System status", "E1"),
        ];

        let coverage = analyze_coverage(QueryTarget::Unknown, &evidence);
        // Unknown targets have no requirements, so 100% coverage
        assert_eq!(coverage.coverage_percent, 100);
    }

    #[test]
    fn test_multiple_evidence_sources() {
        let evidence = vec![
            make_evidence("link_state_summary", "eth0: UP, carrier present", "E1"),
            make_evidence("ip_route_summary", "default via 192.168.1.1", "E2"),
            make_evidence("dns_summary", "nameserver 8.8.8.8", "E3"),
        ];

        let coverage = analyze_coverage(QueryTarget::NetworkStatus, &evidence);
        // Should have good coverage from multiple sources
        assert!(coverage.coverage_percent >= 50);
    }
}
