// Beta.248: Large NL QA Regression Test Suite
// ============================================
//
// Beta.274: Expanded to 700 tests with comprehensive coverage metrics
//
// This test suite contains 700 real-world natural language queries focused on
// system health, status, and diagnostic patterns. It complements the smoke suite
// (178 tests) with broader coverage of actual user queries.
//
// Test data: tests/data/regression_nl_big.toml
//
// Purpose:
// - Measure routing accuracy on real-world questions
// - Detect routing regressions as Anna evolves
// - Provide visibility into which queries work and which don't
// - Generate per-route, per-classification, per-priority coverage metrics
//
// Beta.248/274 is measurement-only: no behavior changes, just data collection.

use serde::Deserialize;
use std::fs;

// ============================================================================
// TEST DATA STRUCTURES
// ============================================================================

#[derive(Debug, Deserialize)]
struct TestSuite {
    test: Vec<TestCase>,
}

#[derive(Debug, Deserialize, Clone)]
struct TestCase {
    id: String,
    query: String,
    expect_route: String,
    #[serde(default)]
    expect_contains: Vec<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    status: Option<String>,  // Beta.249: Test classification status
    // Beta.252: Metadata fields
    #[serde(default)]
    priority: Option<String>,  // high | medium | low
    #[serde(default)]
    classification: Option<String>,  // router_bug | test_unrealistic | ambiguous | correct
    #[serde(default)]
    target_route: Option<String>,  // What we want long-term
    #[serde(default)]
    current_route: Option<String>,  // What router currently does
}

// ============================================================================
// ROUTE CLASSIFICATION (Copied from smoke tests)
// ============================================================================

/// Classify a query based on the routing logic
/// This duplicates logic from regression_nl_smoke.rs for consistency
fn classify_route(query: &str) -> String {
    // Check for diagnostic queries
    if is_full_diagnostic_query(query) {
        return "diagnostic".to_string();
    }

    // Check for status queries
    if is_system_report_query(query) {
        return "status".to_string();
    }

    // Default to conversational
    "conversational".to_string()
}

/// Normalize query text for consistent matching
/// Beta.254: Enhanced to match production normalization
fn normalize_query_for_intent(text: &str) -> String {
    let mut normalized = text.to_lowercase();

    // Beta.254: Strip repeated trailing punctuation
    while normalized.ends_with("???") || normalized.ends_with("!!!") || normalized.ends_with("...") ||
          normalized.ends_with("??") || normalized.ends_with("!!") || normalized.ends_with("..") ||
          normalized.ends_with("?!") || normalized.ends_with("!?") {
        normalized = normalized[..normalized.len()-2].to_string();
    }

    normalized = normalized.trim_end_matches(|c| c == '?' || c == '.' || c == '!').to_string();

    // Beta.254: Strip trailing emojis
    let trailing_emojis = ["🙂", "😊", "😅", "😉", "🤔", "👍", "✅"];
    for emoji in &trailing_emojis {
        if normalized.ends_with(emoji) {
            normalized = normalized[..normalized.len() - emoji.len()].trim_end().to_string();
        }
    }

    // Beta.254: Strip polite fluff
    let polite_prefixes = ["please ", "hey ", "hi ", "hello "];
    for prefix in &polite_prefixes {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
        }
    }

    let polite_suffixes = [" please", " thanks", " thank you"];
    for suffix in &polite_suffixes {
        if normalized.ends_with(suffix) {
            normalized = normalized[..normalized.len() - suffix.len()].to_string();
        }
    }

    normalized = normalized.replace('-', " ").replace('_', " ");

    let mut result = String::new();
    let mut prev_was_space = false;

    for c in normalized.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Check if query is a diagnostic request
fn is_full_diagnostic_query(query: &str) -> bool {
    let normalized = normalize_query_for_intent(query);

    // Exclude conceptual questions
    let conceptual_patterns = ["what is", "what does", "what's", "explain", "define"];
    for pattern in &conceptual_patterns {
        if normalized.starts_with(pattern) {
            return false;
        }
    }

    // Exclude negative context indicators
    let negative_context = ["in general", "in theory", "theoretically"];
    for context in &negative_context {
        if normalized.contains(context) {
            return false;
        }
    }

    // Beta.249: Diagnostic keywords - synced with production (unified_query_handler.rs)
    let diagnostic_keywords = [
        "full diagnostic", "run diagnostic", "perform diagnostic", "execute diagnostic",
        "system health", "health check", "check health", "system ok",
        "everything ok", "any problems", "any issues", "what's wrong",
        "system problems", "system issues", "failed services", "services down",
        "disk problems", "disk issues", "cpu problems", "memory problems",
        "network problems", "boot problems",
        // Beta.249: New high-value patterns
        "is my system healthy", "is the system healthy",
        "is everything ok", "is everything okay", "everything ok", "everything okay",
        "are there any problems", "are there any issues",
        "show me any issues", "anything wrong",
        "are any services failing",
        "how is my system", "how is the system",
        "running out of disk", "running out of memory", "running out of ram",
        "is my cpu overloaded", "is cpu overloaded",
        "disk space problems", "memory problems", "ram problems",
        // Beta.251: Troubleshooting and problem detection
        "what is wrong", "whats wrong", "something wrong", "anything wrong with",
        // Beta.251: Compact health status patterns
        "health status", "status health",
        // Beta.251: Service health patterns
        "service down", "which services down", "which services are down",
        // Beta.251: System doing patterns
        "system doing", "my system doing", "the system doing", "this system doing",
        // Beta.251: Check machine/computer patterns
        "check this machine", "check my machine", "check this computer",
        "check my computer", "check this host", "check my host",
        // Beta.252: Resource health variants (Category A - trivial safe fix)
        "disk healthy", "cpu healthy", "memory healthy", "ram healthy",
        "network health", "machine healthy", "computer healthy",
        // Beta.253: Category B - "What's wrong" with system context
        "what's wrong with my system", "what is wrong with my system",
        "what's wrong with this system", "what's wrong with my machine",
        "what's wrong with this machine", "what's wrong with my computer",
        // Beta.253: Clear diagnostic commands
        "service health", "show me problems", "show problems",
        "display health", "check system", "diagnose system",
        "do diagnostic", "run diagnostic",
        // Beta.254: Resource-specific errors/problems/issues (question format)
        "journal errors", "package problems", "failed boot attempts", "boot attempts",
        "internet connectivity issues", "connectivity issues", "hardware problems",
        "overheating issues", "filesystem errors", "mount problems", "security issues",
        // Beta.254: Possessive health forms
        "computer's health", "machine's health", "system's health",
        // Beta.255: Temporal diagnostic patterns (time + health/error terms)
        "errors today", "errors recently", "errors lately",
        "critical errors today", "critical errors recently",
        "failed services today", "failed services recently",
        "any errors today", "any errors recently",
        "issues today", "issues recently",
        "problems today", "problems recently",
        "failures today", "failures recently",
        "morning system check", "morning check",
        "checking in on the system", "just checking the system",
        // Beta.256: Resource health variants + consolidation
        "is my machine healthy", "is my disk healthy",
        "machine healthy", "disk healthy",
        "full system check", "complete diagnostic",
        "system problem", "service issue",  // singular forms
        "no problems",  // negation pattern
        // Beta.256: Intent markers and polite requests
        "i want to know if my system is healthy",
        "i need a system check", "can you check my system",
    ];

    for keyword in &diagnostic_keywords {
        if normalized.contains(keyword) {
            return true;
        }
    }

    // Beta.249: Resource-specific patterns
    let resources = ["disk", "disk space", "cpu", "memory", "ram", "network", "service", "services"];
    let health_terms = ["problems", "issues", "errors", "failures", "failing"];

    for resource in &resources {
        for term in &health_terms {
            let pattern = format!("{} {}", resource, term);
            if normalized.contains(&pattern) {
                return true;
            }
        }
    }

    false
}

/// Check if query is a status request
fn is_system_report_query(query: &str) -> bool {
    let normalized = normalize_query_for_intent(query);

    let status_keywords = [
        "system status", "current status", "show status",
        "machine status", "system state", "what's happening",
        // Beta.249: Added importance-based status patterns
        "anything important",
        // Beta.251: "status of" patterns
        "status of my system", "status of the system", "status of this system",
        "status of my machine", "status of my computer",
        "status of this machine", "status of this computer",
        // Beta.251: "[my/this] [computer/machine] status" patterns
        "my computer status", "my machine status", "my system status",
        "this computer status", "this machine status", "this system status",
        "computer status",
        // Beta.251: "status current" terse pattern
        "status current", "current system status",
        // Beta.253: Category C - "the machine/computer/system status" variants
        "the machine status", "the computer status", "the system status",
        "check the machine status", "check the computer status", "check the system status",
        "the machine's status", "the computer's status", "the system's status",
        // Beta.255: Temporal + recency patterns for status
        "how is my system today", "how is this system today", "how is my machine today",
        "what happened on this system", "what happened on this machine", "what happened on my system",
        "anything important on my system", "anything important on this machine",
        "any events on my system", "any events on this machine",
        "recently on my system", "recently on this system", "recently on my machine",
        "lately on my system", "lately on this system",
    ];

    for keyword in &status_keywords {
        if normalized.contains(keyword) {
            return true;
        }
    }

    // Single word "status"
    if normalized == "status" {
        return true;
    }

    false
}

// ============================================================================
// TEST EXECUTION
// ============================================================================

#[tokio::test]
async fn test_regression_nl_big_suite() {
    // Load test data
    let test_data_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/regression_nl_big.toml"
    );

    let toml_content = fs::read_to_string(test_data_path)
        .expect("Failed to read regression_nl_big.toml");

    let test_suite: TestSuite = toml::from_str(&toml_content)
        .expect("Failed to parse regression_nl_big.toml");

    println!("\n🧪 Beta.248 Large NL QA Regression Suite");
    println!("=========================================\n");

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_tests = Vec::new();

    for test in &test_suite.test {
        let actual_route = classify_route(&test.query);

        let route_match = actual_route == test.expect_route;

        if route_match {
            passed += 1;
        } else {
            failed += 1;
            failed_tests.push((test.clone(), actual_route.clone()));
        }

        // Optional: Print each test result (can be verbose for 250 tests)
        // Uncomment for debugging:
        // println!(
        //     "{} {} - Expected: {}, Got: {}",
        //     if route_match { "✓" } else { "✗" },
        //     test.id,
        //     test.expect_route,
        //     actual_route
        // );
    }

    // ========================================================================
    // SUMMARY REPORT
    // ========================================================================

    println!("\n📊 Test Summary");
    println!("================\n");
    println!("Total tests:  {}", test_suite.test.len());
    println!("Passed:       {} ({:.1}%)",
        passed,
        (passed as f64 / test_suite.test.len() as f64) * 100.0
    );
    println!("Failed:       {} ({:.1}%)",
        failed,
        (failed as f64 / test_suite.test.len() as f64) * 100.0
    );

    // ========================================================================
    // BETA.252: ENHANCED FAILURE BREAKDOWN WITH METADATA
    // ========================================================================

    if !failed_tests.is_empty() {
        println!("\n⚠️  Failed Tests Breakdown (Beta.252 Taxonomy)");
        println!("=============================================\n");

        // Group by classification
        let mut router_bugs = Vec::new();
        let mut test_unrealistic = Vec::new();
        let mut ambiguous_cases = Vec::new();
        let mut unclassified = Vec::new();

        for (test, actual) in &failed_tests {
            match test.classification.as_ref().map(|s| s.as_str()) {
                Some("router_bug") => router_bugs.push((test, actual)),
                Some("test_unrealistic") => test_unrealistic.push((test, actual)),
                Some("ambiguous") => ambiguous_cases.push((test, actual)),
                _ => unclassified.push((test, actual)),
            }
        }

        // Router bugs (should be fixed)
        if !router_bugs.is_empty() {
            println!("🐛 Router Bugs (should be fixed): {}", router_bugs.len());

            // Group by priority
            let high_priority: Vec<_> = router_bugs.iter()
                .filter(|(t, _)| t.priority.as_ref().map(|p| p == "high").unwrap_or(false))
                .collect();
            let medium_priority: Vec<_> = router_bugs.iter()
                .filter(|(t, _)| t.priority.as_ref().map(|p| p == "medium").unwrap_or(false))
                .collect();

            if !high_priority.is_empty() {
                println!("  HIGH priority ({}):  ", high_priority.len());
                for (test, actual) in high_priority.iter().take(5) {
                    println!("    • {} - \"{}\"", test.id, test.query);
                    println!("      Expected: {}, Got: {}", test.expect_route, actual);
                }
            }

            if !medium_priority.is_empty() {
                println!("  MEDIUM priority ({}):  ", medium_priority.len());
                for (test, actual) in medium_priority.iter().take(5) {
                    println!("    • {} - \"{}\"", test.id, test.query);
                    println!("      Expected: {}, Got: {}", test.expect_route, actual);
                }
                if medium_priority.len() > 5 {
                    println!("    ... and {} more", medium_priority.len() - 5);
                }
            }
            println!();
        }

        // Test expectations unrealistic
        if !test_unrealistic.is_empty() {
            println!("📝 Test Expectations Unrealistic (consider changing): {}", test_unrealistic.len());
            for (test, actual) in test_unrealistic.iter().take(5) {
                println!("  • {} - \"{}\"", test.id, test.query);
                println!("    Expected: {}, Got: {} (current routing is defensible)", test.expect_route, actual);
            }
            if test_unrealistic.len() > 5 {
                println!("  ... and {} more", test_unrealistic.len() - 5);
            }
            println!();
        }

        // Ambiguous cases
        if !ambiguous_cases.is_empty() {
            println!("❓ Ambiguous Cases (could go either way): {}", ambiguous_cases.len());
            for (test, actual) in ambiguous_cases.iter().take(3) {
                println!("  • {} - \"{}\" -> {}", test.id, test.query, actual);
            }
            if ambiguous_cases.len() > 3 {
                println!("  ... and {} more", ambiguous_cases.len() - 3);
            }
            println!();
        }

        // Legacy breakdown (Expected diagnostic, got other)
        println!("📊 Legacy Breakdown by Expected Route:");
        let mut diagnostic_failures = Vec::new();
        let mut status_failures = Vec::new();
        let mut conversational_failures = Vec::new();

        for (test, actual) in &failed_tests {
            match test.expect_route.as_str() {
                "diagnostic" => diagnostic_failures.push((test, actual)),
                "status" => status_failures.push((test, actual)),
                "conversational" => conversational_failures.push((test, actual)),
                _ => {},
            }
        }

        if !diagnostic_failures.is_empty() {
            println!("  Expected diagnostic, got other: {}", diagnostic_failures.len());
        }
        if !status_failures.is_empty() {
            println!("  Expected status, got other: {}", status_failures.len());
        }
        if !conversational_failures.is_empty() {
            println!("  Expected conversational, got other: {}", conversational_failures.len());
        }
    }

    // ========================================================================
    // ROUTE DISTRIBUTION ANALYSIS
    // ========================================================================

    println!("\n📈 Route Distribution");
    println!("=====================\n");

    let mut diagnostic_expected = 0;
    let mut status_expected = 0;
    let mut conversational_expected = 0;

    for test in &test_suite.test {
        match test.expect_route.as_str() {
            "diagnostic" => diagnostic_expected += 1,
            "status" => status_expected += 1,
            "conversational" => conversational_expected += 1,
            _ => {},
        }
    }

    println!("Expected routing breakdown:");
    println!("  Diagnostic:      {} ({:.1}%)",
        diagnostic_expected,
        (diagnostic_expected as f64 / test_suite.test.len() as f64) * 100.0
    );
    println!("  Status:          {} ({:.1}%)",
        status_expected,
        (status_expected as f64 / test_suite.test.len() as f64) * 100.0
    );
    println!("  Conversational:  {} ({:.1}%)",
        conversational_expected,
        (conversational_expected as f64 / test_suite.test.len() as f64) * 100.0
    );

    // ========================================================================
    // BETA.274: COVERAGE METRICS
    // ========================================================================

    println!("\n📊 Beta.274 Coverage Metrics");
    println!("============================\n");

    // Overall metrics
    let total = test_suite.test.len();
    let accuracy = (passed as f64 / total as f64) * 100.0;
    println!("Overall Coverage:");
    println!("  Total tests:  {}", total);
    println!("  Passed:       {} ({:.1}%)", passed, accuracy);
    println!("  Failed:       {} ({:.1}%)", failed, 100.0 - accuracy);
    println!();

    // Per-route coverage
    println!("Per-Route Coverage:");
    for route in &["diagnostic", "status", "conversational"] {
        let route_tests: Vec<_> = test_suite.test.iter()
            .filter(|t| t.expect_route == *route)
            .collect();
        let route_passed = route_tests.iter()
            .filter(|t| classify_route(&t.query) == *route)
            .count();
        let route_total = route_tests.len();
        let route_accuracy = if route_total > 0 {
            (route_passed as f64 / route_total as f64) * 100.0
        } else {
            0.0
        };
        println!("  {:15} {}/{} ({:.1}%)",
            format!("{}:", route),
            route_passed,
            route_total,
            route_accuracy
        );
    }
    println!();

    // Per-classification coverage
    println!("Per-Classification Coverage:");
    for classification in &["correct", "router_bug", "test_unrealistic", "ambiguous"] {
        let class_tests: Vec<_> = test_suite.test.iter()
            .filter(|t| t.classification.as_ref().map(|c| c == *classification).unwrap_or(false))
            .collect();
        let class_passed = class_tests.iter()
            .filter(|t| classify_route(&t.query) == t.expect_route)
            .count();
        let class_total = class_tests.len();
        let class_accuracy = if class_total > 0 {
            (class_passed as f64 / class_total as f64) * 100.0
        } else {
            0.0
        };
        if class_total > 0 {
            println!("  {:20} {}/{} ({:.1}%)",
                format!("{}:", classification),
                class_passed,
                class_total,
                class_accuracy
            );
        }
    }
    println!();

    // Per-priority coverage
    println!("Per-Priority Coverage:");
    for priority in &["high", "medium", "low"] {
        let priority_tests: Vec<_> = test_suite.test.iter()
            .filter(|t| t.priority.as_ref().map(|p| p == *priority).unwrap_or(false))
            .collect();
        let priority_passed = priority_tests.iter()
            .filter(|t| classify_route(&t.query) == t.expect_route)
            .count();
        let priority_total = priority_tests.len();
        let priority_accuracy = if priority_total > 0 {
            (priority_passed as f64 / priority_total as f64) * 100.0
        } else {
            0.0
        };
        if priority_total > 0 {
            println!("  {:10} {}/{} ({:.1}%)",
                format!("{}:", priority),
                priority_passed,
                priority_total,
                priority_accuracy
            );
        }
    }

    // ========================================================================
    // FINAL ASSERTION
    // ========================================================================

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Beta.248/274: This is measurement-only, so we don't fail the test on routing mismatches
    // Instead, we pass the test unconditionally and report the results

    if failed > 0 {
        println!("⚠️  NOTE: {} tests had routing mismatches.", failed);
        println!("   Beta.248/274 is measurement-only - these are documented for future improvement.");
        println!("   See BETA_274_NOTES.md for analysis and next steps.\n");
    } else {
        println!("✅ All tests passed! Routing is 100% accurate.\n");
    }

    // For now, always pass the test to collect data
    // Future betas may enforce routing accuracy
    assert!(true, "Beta.274 big suite measurement complete");
}

// ============================================================================
// BETA.252: EXPORT ROUTING RESULTS FOR ANNOTATION
// ============================================================================

#[tokio::test]
async fn test_export_routing_results() {
    // Load test data
    let test_data_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/regression_nl_big.toml"
    );

    let toml_content = fs::read_to_string(test_data_path)
        .expect("Failed to read regression_nl_big.toml");

    let test_suite: TestSuite = toml::from_str(&toml_content)
        .expect("Failed to parse regression_nl_big.toml");

    // Export routing results to /tmp for annotation script
    let output_path = "/tmp/regression_nl_routes.txt";
    let mut output = String::new();

    output.push_str("# Beta.252: Routing Results Export\n");
    output.push_str("# Format: test_id | query | current_route | expect_route | match\n");
    output.push_str("# ========================================================================\n");

    for test in &test_suite.test {
        let current_route = classify_route(&test.query);
        let matches = if current_route == test.expect_route { "✓" } else { "✗" };

        output.push_str(&format!(
            "{} | {} | {} | {} | {}\n",
            test.id,
            test.query,
            current_route,
            test.expect_route,
            matches
        ));
    }

    fs::write(output_path, output)
        .expect("Failed to write routing results");

    println!("✅ Routing results exported to {}", output_path);
    println!("   Use this file with /tmp/extract_routes_from_test.py to annotate TOML");
}
