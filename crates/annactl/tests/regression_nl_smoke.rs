//! Natural Language Regression Test Suite - Smoke Tests
//!
//! Beta.241: First proper regression harness for NL query routing
//!
//! This test suite validates that natural language queries route to the
//! correct processing tiers and produce expected output patterns.
//!
//! Design:
//! - Loads test cases from tests/data/regression_nl_smoke.toml
//! - Mocks RPC/daemon dependencies for determinism
//! - Validates routing decisions and output patterns
//! - Fast, deterministic, no real system dependencies
//!
//! Run with: cargo test --test regression_nl_smoke

use serde::Deserialize;
use std::collections::HashMap;

/// Test case from regression data file
#[derive(Debug, Deserialize)]
struct RegressionTest {
    id: String,
    query: String,
    expect_route: String,
    #[serde(default)]
    expect_contains: Vec<String>,
    #[serde(default)]
    notes: String,
}

/// Container for all regression tests
#[derive(Debug, Deserialize)]
struct RegressionSuite {
    test: Vec<RegressionTest>,
}

/// Route classification for assertions
#[derive(Debug, PartialEq, Eq)]
enum RouteType {
    Diagnostic,
    Status,
    Recipe,
    Conversational,
    Fallback,
}

impl RouteType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "diagnostic" => RouteType::Diagnostic,
            "status" => RouteType::Status,
            "recipe" => RouteType::Recipe,
            "conversational" => RouteType::Conversational,
            "fallback" => RouteType::Fallback,
            _ => panic!("Unknown route type: {}", s),
        }
    }
}

/// Mock diagnostic result for deterministic testing
fn mock_diagnostic_response() -> String {
    "[SUMMARY]\nSystem health: **All systems nominal**\n\n[DETAILS]\nNo critical issues detected.\n\n[COMMANDS]\nNo actions required.".to_string()
}

/// Mock status response for deterministic testing
fn mock_status_response() -> String {
    "System Status:\n- CPU: 5%\n- RAM: 2.1GB\n- Disk: 45% used".to_string()
}

/// Mock conversational response for deterministic testing
fn mock_conversational_response() -> String {
    "Based on the available information, here's what I can tell you...".to_string()
}

/// Beta.243: Normalize query text for consistent matching (matches production)
fn normalize_query_for_intent(text: &str) -> String {
    let mut normalized = text.to_lowercase();
    normalized = normalized.trim_end_matches(|c| c == '?' || c == '.' || c == '!').to_string();
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

/// Classify a query based on the routing logic from unified_query_handler.rs
/// Beta.241: This mimics the actual routing tiers without requiring daemon
/// Beta.243: Updated to match production routing with normalization
fn classify_query_route(query: &str) -> RouteType {
    // TIER 0.5: Diagnostic queries (Beta.238/239/243)
    if is_full_diagnostic_query(query) {
        return RouteType::Diagnostic;
    }

    // TIER 0: System report queries (Beta.243: expanded keywords)
    if is_system_report_query(query) {
        return RouteType::Status;
    }

    // TIER 1: Recipe queries (simplified check)
    // For smoke tests, we don't need full recipe matching
    // Just check if it's clearly not diagnostic/status

    // Default to conversational for everything else
    RouteType::Conversational
}

/// Beta.241: Diagnostic query detection (matches unified_query_handler.rs)
/// Beta.243: Updated with normalization, phrase variations, bi-directional matching
/// Beta.244: Conceptual question exclusions, contextual system references
/// This is a copy of the production logic for testing purposes
fn is_full_diagnostic_query(query: &str) -> bool {
    // Beta.243: Apply normalization for robust matching
    let normalized = normalize_query_for_intent(query);

    // Beta.244: Exclude conceptual/definition questions
    let conceptual_patterns = [
        "what is",
        "what does",
        "what's",
        "explain",
        "define",
        "definition of",
        "what are",
        "describe",
        "tell me about",
    ];

    let conceptual_subjects = [
        "healthy system",
        "system health",
        "a healthy",
        "good system health",
    ];

    for concept_pattern in &conceptual_patterns {
        if normalized.contains(concept_pattern) {
            for subject in &conceptual_subjects {
                if normalized.contains(subject) {
                    return false;
                }
            }
        }
    }

    // Exact phrase matches (matches production)
    let exact_matches = [
        "run a full diagnostic",
        "run full diagnostic",
        "full diagnostic",
        "check my system health",
        "check the system health",  // Beta.243
        "check system health",
        "show any problems",
        "show me any problems",
        "full system diagnostic",
        "system health check",
        "health check",
        "check health",  // Beta.243: bi-directional
        "system check",
        "check system",  // Beta.243: bi-directional
        "health report",
        "system report",
        "is my system ok",
        "is the system ok",  // Beta.243
        "is the system okay",  // Beta.243
        "is everything ok with my system",
        "is my system okay",
        "is everything okay with my system",
        // Beta.243: Terse patterns
        "my system ok",
        "my system okay",
        "the system ok",
        "the system okay",
        "system ok",
        "system okay",
        // Beta.243: Verb variations
        "run diagnostic",
        "perform diagnostic",
        "execute diagnostic",
    ];

    for phrase in &exact_matches {
        if normalized.contains(phrase) {
            return true;
        }
    }

    // Pattern matches
    if normalized.contains("diagnose") {
        if normalized.contains("system") || normalized.contains("my system") || normalized.contains("the system") {
            return true;
        }
    }

    if normalized.contains("full") && normalized.contains("diagnostic") {
        return true;
    }

    // Beta.244: Enhanced system+health pattern with contextual awareness
    if normalized.contains("system") && normalized.contains("health") {
        let positive_indicators = [
            "this system",
            "this machine",
            "my system",
            "my machine",
            "on this computer",
            "on this system",
            "on my system",
            "here",
            "this computer",
        ];

        let negative_indicators = [
            "in general",
            "in theory",
            "on linux",
            "on arch linux",
            "in arch",
            "for linux",
        ];

        let has_positive = positive_indicators.iter().any(|ind| normalized.contains(ind));
        let has_negative = negative_indicators.iter().any(|ind| normalized.contains(ind));

        if has_positive && has_negative {
            return false;
        }

        if has_positive {
            return true;
        }

        return true;
    }

    false
}

/// Beta.241: System report query detection (simplified)
/// Beta.243: Expanded status keyword coverage
/// Beta.244: Temporal and importance-based status patterns
fn is_system_report_query(query: &str) -> bool {
    let normalized = normalize_query_for_intent(query);

    let status_keywords = [
        "show me status",
        "system status",
        "what's running",
        "system information",
        "system info",
        // Beta.243: New status keywords
        "current status",
        "what is the current status",
        "what is the status",
        "system state",
        "show system state",
        "what's happening on my system",
        "what's happening",
        "how is my system",
        "how is the system",
    ];

    for keyword in &status_keywords {
        if normalized.contains(keyword) {
            return true;
        }
    }

    // Beta.244: Temporal and importance-based status patterns
    let temporal_indicators = ["today", "now", "currently", "right now"];
    let importance_indicators = [
        "anything important",
        "anything critical",
        "anything wrong",
        "any issues",
        "any problems",
        "important to review",
        "to review",
        "should know",
        "need to know",
    ];
    let system_references = [
        "this system",
        "this machine",
        "this computer",
        "my system",
        "my machine",
        "my computer",
        "the system",
        "the machine",
    ];

    let has_temporal = temporal_indicators.iter().any(|ind| normalized.contains(ind));
    let has_importance = importance_indicators.iter().any(|ind| normalized.contains(ind));
    let has_system_ref = system_references.iter().any(|ind| normalized.contains(ind));

    if (has_temporal || has_importance) && has_system_ref {
        return true;
    }

    false
}

/// Load regression test suite from TOML file
fn load_regression_suite() -> RegressionSuite {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/regression_nl_smoke.toml");
    let toml_content = std::fs::read_to_string(toml_path)
        .expect("Failed to read regression test data file");

    toml::from_str(&toml_content)
        .expect("Failed to parse regression test TOML")
}

/// Generate mock response based on route type
fn generate_mock_response(route: &RouteType) -> String {
    match route {
        RouteType::Diagnostic => mock_diagnostic_response(),
        RouteType::Status => mock_status_response(),
        RouteType::Conversational | RouteType::Fallback => mock_conversational_response(),
        RouteType::Recipe => "Mock recipe response".to_string(),
    }
}

// ============================================================================
// TEST HARNESS UTILITIES (Beta.242)
// ============================================================================

/// Assert that a query routes to the expected route type
///
/// Beta.242: Helper function to make test assertions more readable
fn assert_route(query: &str, expected: RouteType) -> bool {
    let actual = classify_query_route(query);
    actual == expected
}

/// Assert that a response contains any of the given substrings
///
/// Beta.242: Helper for content validation
fn assert_contains_any(response: &str, expected_substrings: &[String]) -> (bool, Vec<String>) {
    let mut missing = Vec::new();

    for substring in expected_substrings {
        if !response.contains(substring) {
            missing.push(substring.clone());
        }
    }

    (missing.is_empty(), missing)
}

/// Assert that a response does NOT contain any of the given substrings
///
/// Beta.242: Helper for negative assertions
fn assert_not_contains(response: &str, forbidden_substrings: &[String]) -> (bool, Vec<String>) {
    let mut found = Vec::new();

    for substring in forbidden_substrings {
        if response.contains(substring) {
            found.push(substring.clone());
        }
    }

    (found.is_empty(), found)
}

#[test]
fn test_regression_nl_smoke_suite() {
    let suite = load_regression_suite();

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<String> = Vec::new();

    println!("\n🧪 Running Natural Language Regression Suite (Smoke Tests)");
    println!("══════════════════════════════════════════════════════════\n");

    for test_case in &suite.test {
        let expected_route = RouteType::from_str(&test_case.expect_route);
        let actual_route = classify_query_route(&test_case.query);

        // Check route classification
        let route_matches = expected_route == actual_route;

        // Generate mock response for this route
        let response = generate_mock_response(&actual_route);

        // Check expected substrings (if any)
        let mut contains_checks_pass = true;
        let mut missing_substrings = Vec::new();

        for expected_substring in &test_case.expect_contains {
            if !response.contains(expected_substring) {
                contains_checks_pass = false;
                missing_substrings.push(expected_substring.clone());
            }
        }

        // Determine if test passed
        let test_passed = route_matches && contains_checks_pass;

        if test_passed {
            passed += 1;
            println!("✅ {} - {}", test_case.id, test_case.notes);
        } else {
            failed += 1;
            let mut failure_msg = format!("❌ {} - FAILED", test_case.id);

            if !route_matches {
                failure_msg.push_str(&format!(
                    "\n   Route mismatch: expected {:?}, got {:?}",
                    expected_route, actual_route
                ));
            }

            if !contains_checks_pass {
                failure_msg.push_str(&format!(
                    "\n   Missing substrings: {:?}",
                    missing_substrings
                ));
            }

            failure_msg.push_str(&format!("\n   Query: \"{}\"", test_case.query));

            println!("{}", failure_msg);
            failures.push(failure_msg);
        }
    }

    println!("\n══════════════════════════════════════════════════════════");
    println!("📊 Test Summary:");
    println!("   Total: {}", suite.test.len());
    println!("   Passed: {} ({}%)", passed, (passed * 100) / suite.test.len());
    println!("   Failed: {}", failed);
    println!("══════════════════════════════════════════════════════════\n");

    if failed > 0 {
        println!("\n❌ FAILURES:\n");
        for failure in &failures {
            println!("{}\n", failure);
        }
        panic!("{} test(s) failed", failed);
    }
}

#[test]
fn test_diagnostic_phrase_coverage() {
    // Verify all Beta.238 original phrases are tested
    let suite = load_regression_suite();

    let beta_238_phrases = vec![
        "run a full diagnostic",
        "check my system health",
        "show any problems",
        "full diagnostic",
        "diagnose my system",
    ];

    for phrase in beta_238_phrases {
        let found = suite.test.iter().any(|t| t.query.to_lowercase().contains(phrase));
        assert!(found, "Beta.238 phrase not tested: {}", phrase);
    }
}

#[test]
fn test_beta_239_phrase_coverage() {
    // Verify all Beta.239 new phrases are tested
    let suite = load_regression_suite();

    let beta_239_phrases = vec![
        "health check",
        "system check",
        "health report",
        "system report",
        "is my system ok",
    ];

    for phrase in beta_239_phrases {
        let found = suite.test.iter().any(|t| t.query.to_lowercase().contains(phrase));
        assert!(found, "Beta.239 phrase not tested: {}", phrase);
    }
}

#[test]
fn test_false_positive_prevention() {
    // Ensure known false positives are explicitly tested
    let suite = load_regression_suite();

    let false_positives = vec![
        "health insurance",
        "system update",
    ];

    for fp in false_positives {
        let found = suite.test.iter().any(|t| {
            t.query.to_lowercase().contains(fp) && t.expect_route != "diagnostic"
        });
        assert!(found, "False positive not tested: {}", fp);
    }
}
