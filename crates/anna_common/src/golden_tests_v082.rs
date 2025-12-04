//! Golden Tests v0.0.82 - Normal Mode Clean Output
//!
//! These tests verify that the pre-router and direct handlers produce
//! clean, professional output suitable for normal (human) mode.
//!
//! Key invariants for normal mode:
//! - No raw evidence IDs (like [E1], [E2])
//! - No tool names (like hw_snapshot_summary)
//! - No internal parse errors or debug info
//! - Professional IT department dialogue style
//! - Reliability shown as percentage or brief text

use crate::pipeline_v082::{try_direct_handle, DirectHandlerResult};
use crate::pre_router::{pre_route, PreRouterIntent};

// =============================================================================
// Pre-Router Golden Tests
// =============================================================================

#[cfg(test)]
mod pre_router_tests {
    use super::*;

    #[test]
    fn golden_stats_query_variations() {
        let queries = [
            "stats",
            "show my stats",
            "what's my xp",
            "my level",
            "annactl stats",
            "anna stats",
            "show stats",
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                result.matched,
                "Query '{}' should match pre-router",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::StatsQuery,
                "Query '{}' should be StatsQuery",
                query
            );
        }
    }

    #[test]
    fn golden_memory_query_variations() {
        let queries = [
            "how much ram",
            "free memory",
            "available ram",
            "memory usage",
            "how much memory do i have",
            // Typo tolerance
            "how much free rum", // rum -> RAM typo
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                result.matched,
                "Query '{}' should match pre-router",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::MemoryQuery,
                "Query '{}' should be MemoryQuery",
                query
            );
        }
    }

    #[test]
    fn golden_disk_query_variations() {
        let queries = [
            "disk space",
            "free space",
            "storage",
            "how much disk space",
            "check disk",
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                result.matched,
                "Query '{}' should match pre-router",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::DiskQuery,
                "Query '{}' should be DiskQuery",
                query
            );
        }
    }

    #[test]
    fn golden_updates_query_variations() {
        let queries = [
            "check for updates",
            "any updates",
            "pending updates",
            "updates",
            "check updates",
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                result.matched,
                "Query '{}' should match pre-router",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::UpdatesQuery,
                "Query '{}' should be UpdatesQuery",
                query
            );
        }
    }

    #[test]
    fn golden_editor_query_variations() {
        let queries = [
            "what editor am i using",
            "which editor",
            "my editor",
            "vim",
            "nvim",
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                result.matched,
                "Query '{}' should match pre-router",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::EditorQuery,
                "Query '{}' should be EditorQuery",
                query
            );
        }
    }

    #[test]
    fn golden_debug_toggle_variations() {
        let queries = [
            "enable debug",
            "disable debug",
            "debug on",
            "debug off",
            "turn debug on",
            "turn debug off",
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                result.matched,
                "Query '{}' should match pre-router",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::DebugToggle,
                "Query '{}' should be DebugToggle",
                query
            );
        }
    }

    #[test]
    fn golden_capabilities_variations() {
        let queries = [
            "what can you do",
            "help me",
            "your capabilities",
            "what do you do",
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                result.matched,
                "Query '{}' should match pre-router",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::CapabilitiesQuery,
                "Query '{}' should be CapabilitiesQuery",
                query
            );
        }
    }

    #[test]
    fn golden_no_match_should_fallthrough() {
        let queries = [
            "restart nginx",
            "install docker",
            "tell me a joke",
            "what is kubernetes",
        ];

        for query in queries {
            let result = pre_route(query);
            assert!(
                !result.matched,
                "Query '{}' should NOT match pre-router (falls through to translator)",
                query
            );
            assert_eq!(
                result.intent,
                PreRouterIntent::NoMatch,
                "Query '{}' should be NoMatch",
                query
            );
        }
    }
}

// =============================================================================
// Direct Handler Output Golden Tests
// =============================================================================

#[cfg(test)]
mod handler_output_tests {
    use super::*;

    /// Verify output has no raw evidence IDs
    fn assert_no_evidence_ids(output: &str, context: &str) {
        // Common evidence ID patterns
        let patterns = ["[E1]", "[E2]", "[E3]", "[E4]", "[E5]", "E001", "E002"];
        for pattern in patterns {
            assert!(
                !output.contains(pattern),
                "{}: Output should not contain evidence ID pattern '{}'\nOutput: {}",
                context,
                pattern,
                output
            );
        }
    }

    /// Verify output has no tool names
    fn assert_no_tool_names(output: &str, context: &str) {
        let tool_names = [
            "hw_snapshot_summary",
            "sw_snapshot_summary",
            "memory_info",
            "mount_usage",
            "kernel_version",
            "network_status",
            "service_status",
            "audio_status",
            "boot_time",
            "journalctl",
        ];
        for tool in tool_names {
            assert!(
                !output.contains(tool),
                "{}: Output should not contain tool name '{}'\nOutput: {}",
                context,
                tool,
                output
            );
        }
    }

    /// Verify output has no parse errors or debug info
    fn assert_no_debug_info(output: &str, context: &str) {
        let debug_patterns = [
            "parse error",
            "parse failed",
            "ParseError",
            "fallback",
            "retry",
            "timeout",
            "DEBUG:",
            "TRACE:",
            "WARN:",
            "ERROR:",
        ];
        for pattern in debug_patterns {
            assert!(
                !output.to_lowercase().contains(&pattern.to_lowercase()),
                "{}: Output should not contain debug pattern '{}'\nOutput: {}",
                context,
                pattern,
                output
            );
        }
    }

    #[test]
    fn golden_memory_output_clean() {
        let result = try_direct_handle("how much ram");
        assert!(result.handled);

        assert_no_evidence_ids(&result.response, "memory query");
        assert_no_tool_names(&result.response, "memory query");
        assert_no_debug_info(&result.response, "memory query");

        // Should contain human-readable info
        assert!(
            result.response.contains("Memory")
                || result.response.contains("GB")
                || result.response.contains("RAM"),
            "Memory output should contain human-readable memory info"
        );
    }

    #[test]
    fn golden_disk_output_clean() {
        let result = try_direct_handle("disk space");
        assert!(result.handled);

        assert_no_evidence_ids(&result.response, "disk query");
        assert_no_tool_names(&result.response, "disk query");
        assert_no_debug_info(&result.response, "disk query");

        // Should contain human-readable info
        assert!(
            result.response.contains("Disk") || result.response.contains("Usage"),
            "Disk output should contain human-readable disk info"
        );
    }

    #[test]
    fn golden_capabilities_output_clean() {
        let result = try_direct_handle("what can you do");
        assert!(result.handled);

        assert_no_evidence_ids(&result.response, "capabilities");
        assert_no_tool_names(&result.response, "capabilities");
        assert_no_debug_info(&result.response, "capabilities");

        // Should contain helpful info
        assert!(
            result.response.contains("help")
                || result.response.contains("Anna")
                || result.response.contains("SYSTEM"),
            "Capabilities output should contain helpful information"
        );
    }

    #[test]
    fn golden_debug_toggle_enable_clean() {
        let result = try_direct_handle("enable debug");
        assert!(result.handled);

        assert_no_evidence_ids(&result.response, "debug enable");
        // Allow "debug" as it's the feature name
        assert_no_debug_info(&result.response, "debug enable");

        // Should confirm enable
        assert!(
            result.response.to_lowercase().contains("enable")
                || result.response.to_lowercase().contains("debug"),
            "Debug enable output should confirm the action"
        );
    }

    #[test]
    fn golden_debug_toggle_disable_clean() {
        let result = try_direct_handle("disable debug");
        assert!(result.handled);

        assert_no_evidence_ids(&result.response, "debug disable");
        assert_no_debug_info(&result.response, "debug disable");

        // Should confirm disable
        assert!(
            result.response.to_lowercase().contains("disable")
                || result.response.to_lowercase().contains("debug")
                || result.response.to_lowercase().contains("clean"),
            "Debug disable output should confirm the action"
        );
    }

    #[test]
    fn golden_cpu_output_clean() {
        let result = try_direct_handle("what cpu");
        assert!(result.handled);

        assert_no_evidence_ids(&result.response, "cpu query");
        assert_no_tool_names(&result.response, "cpu query");
        assert_no_debug_info(&result.response, "cpu query");
    }

    #[test]
    fn golden_kernel_output_clean() {
        let result = try_direct_handle("kernel version");
        assert!(result.handled);

        assert_no_evidence_ids(&result.response, "kernel query");
        assert_no_tool_names(&result.response, "kernel query");
        assert_no_debug_info(&result.response, "kernel query");

        // Should contain version info
        assert!(
            result.response.contains("Kernel") || result.response.contains("Release"),
            "Kernel output should contain version info"
        );
    }

    #[test]
    fn golden_typo_tolerance_rum_to_ram() {
        // "rum" should be handled as "RAM"
        let result = try_direct_handle("how much free rum");
        assert!(result.handled);
        assert_eq!(result.intent, "memory_query");

        // Should produce memory info, not an error
        assert!(
            result.response.contains("Memory") || result.response.contains("GB"),
            "Typo 'rum' should be understood as RAM query"
        );
    }
}

// =============================================================================
// Reliability Score Tests
// =============================================================================

#[cfg(test)]
mod reliability_tests {
    use super::*;

    #[test]
    fn golden_deterministic_handlers_have_high_reliability() {
        let queries = [
            ("how much ram", 100),
            ("disk space", 100),
            ("what can you do", 100),
            ("enable debug", 100),
            ("what cpu", 100),
            ("kernel version", 100),
        ];

        for (query, expected_reliability) in queries {
            let result = try_direct_handle(query);
            assert!(result.handled);
            assert_eq!(
                result.reliability, expected_reliability,
                "Query '{}' should have reliability {}",
                query, expected_reliability
            );
        }
    }
}

// =============================================================================
// Integration Test: Full Pipeline Verification
// =============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn golden_integration_normal_mode_workflow() {
        // Simulate a user session in normal mode

        // 1. User asks about capabilities
        let result = try_direct_handle("what can you do");
        assert!(result.handled);
        assert!(!result.response.is_empty());

        // 2. User checks memory
        let result = try_direct_handle("how much ram");
        assert!(result.handled);
        assert!(result.reliability == 100);

        // 3. User enables debug
        let result = try_direct_handle("enable debug");
        assert!(result.handled);

        // 4. User disables debug
        let result = try_direct_handle("disable debug");
        assert!(result.handled);

        // 5. User checks updates
        let result = try_direct_handle("any updates");
        assert!(result.handled);

        // 6. User asks something that needs translator
        let result = try_direct_handle("install nginx");
        assert!(!result.handled, "install nginx should go to translator");
    }
}
