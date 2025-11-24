//! CLI tests for `annactl selftest` command (6.3.1)
//!
//! Tests the built-in capability verification system.
//! All tests are offline and deterministic.

use anna_common::selftest::run_selftests;

/// Test: All self-tests pass
///
/// Verifies that the core selftest function reports success
/// for all built-in scenarios.
#[test]
fn selftest_all_ok() {
    // Run self-tests (same function CLI uses)
    let results = run_selftests();

    // Should have at least 3 scenarios
    assert!(
        results.len() >= 3,
        "Should have at least 3 self-test scenarios"
    );

    // All should pass
    for result in &results {
        assert!(
            result.passed,
            "Scenario '{}' should pass but failed: {}",
            result.scenario,
            result.details
        );
    }

    // Verify expected scenarios are present
    let scenario_names: Vec<_> = results.iter().map(|r| r.scenario.as_str()).collect();

    assert!(
        scenario_names.contains(&"DNS / name resolution"),
        "Should test DNS scenario"
    );
    assert!(
        scenario_names.contains(&"Service failure (systemd)"),
        "Should test service failure scenario"
    );
    assert!(
        scenario_names.contains(&"Healthy system safety check"),
        "Should test healthy system safety"
    );
}

/// Test: Selftest result format
///
/// Verifies that selftest results have the expected structure
/// and can be used to generate CLI output.
#[test]
fn selftest_output_format_basic() {
    let results = run_selftests();

    for result in &results {
        // Each result should have a non-empty scenario name
        assert!(
            !result.scenario.is_empty(),
            "Scenario name should not be empty"
        );

        // Each result should have details
        assert!(
            !result.details.is_empty(),
            "Scenario details should not be empty"
        );

        // Passed results should have positive details
        if result.passed {
            // Details should describe what was tested
            assert!(
                result.details.contains("plan") || result.details.contains("no changes"),
                "Passed scenario should describe plan or safety: {}",
                result.details
            );
        }
    }
}

/// Test: DNS scenario is tested
///
/// Verifies that DNS planner behavior is included in self-tests.
#[test]
fn selftest_includes_dns() {
    let results = run_selftests();

    let dns_result = results
        .iter()
        .find(|r| r.scenario.contains("DNS"))
        .expect("Should have DNS scenario");

    assert!(dns_result.passed, "DNS scenario should pass");
    assert!(
        dns_result.details.contains("Arch Wiki"),
        "DNS details should mention Arch Wiki"
    );
}

/// Test: Service failure scenario is tested
///
/// Verifies that service failure planner behavior is included in self-tests.
#[test]
fn selftest_includes_service_failure() {
    let results = run_selftests();

    let service_result = results
        .iter()
        .find(|r| r.scenario.contains("Service"))
        .expect("Should have service failure scenario");

    assert!(service_result.passed, "Service scenario should pass");
    assert!(
        service_result.details.contains("Arch Wiki"),
        "Service details should mention Arch Wiki"
    );
}

/// Test: Healthy system safety is tested
///
/// Verifies that the planner correctly avoids proposing changes
/// when the system is healthy.
#[test]
fn selftest_includes_healthy_safety() {
    let results = run_selftests();

    let safety_result = results
        .iter()
        .find(|r| r.scenario.contains("Healthy"))
        .expect("Should have healthy system safety scenario");

    assert!(safety_result.passed, "Safety scenario should pass");
    assert!(
        safety_result.details.contains("no changes"),
        "Safety details should mention no changes proposed"
    );
}
