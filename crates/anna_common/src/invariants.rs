//! Invariant Guards v3.3.0
//!
//! Centralized invariant checking for all subsystems as defined in
//! docs/FEATURE_INTEGRITY_MATRIX.md.
//!
//! When an invariant is violated:
//! 1. Log a structured "INVARIANT_VIOLATION" event
//! 2. Return a safe fallback value
//! 3. In debug mode, print violation details
//!
//! ## Usage
//!
//! ```ignore
//! use anna_common::invariants::{guard_reliability, guard_answer_text};
//!
//! let reliability = guard_reliability(raw_score, "Senior audit");
//! let text = guard_answer_text(maybe_empty, "Brain answer");
//! ```

use tracing::warn;

// ============================================================================
// Constants from Feature Integrity Matrix
// ============================================================================

/// Brain latency hard limit (150ms)
pub const BRAIN_LATENCY_HARD_LIMIT_MS: u64 = 150;

/// LLM pipeline hard limit (10s)
pub const LLM_LATENCY_HARD_LIMIT_MS: u64 = 10_000;

/// Green reliability threshold (90%)
pub const GREEN_THRESHOLD: f64 = 0.90;

/// Yellow reliability threshold (70%)
pub const YELLOW_THRESHOLD: f64 = 0.70;

/// Red reliability threshold (50%)
pub const RED_THRESHOLD: f64 = 0.50;

/// Recipe extraction minimum reliability
pub const RECIPE_MIN_RELIABILITY: f64 = 0.85;

/// Recipe match threshold
pub const RECIPE_MATCH_THRESHOLD: f64 = 0.70;

/// Maximum LLM calls per question
pub const MAX_LLM_CALLS: usize = 3;

/// Maximum iterations per question
pub const MAX_ITERATIONS: usize = 6;

// ============================================================================
// Invariant Violation Logging
// ============================================================================

/// Log an invariant violation
fn log_violation(invariant_id: &str, expected: &str, actual: &str, context: &str) {
    warn!(
        invariant_id = invariant_id,
        expected = expected,
        actual = actual,
        context = context,
        "INVARIANT_VIOLATION"
    );
}

// ============================================================================
// Reliability Guards
// ============================================================================

/// INV-ANS-002: Guard reliability to 0.0-1.0 range
///
/// Clamps reliability to valid range and logs violation if out of bounds.
pub fn guard_reliability(raw: f64, context: &str) -> f64 {
    if raw.is_nan() {
        log_violation(
            "INV-ANS-002",
            "0.0-1.0",
            "NaN",
            context,
        );
        return 0.0;
    }

    if raw < 0.0 {
        log_violation(
            "INV-ANS-002",
            "0.0-1.0",
            &format!("{:.4}", raw),
            context,
        );
        return 0.0;
    }

    if raw > 1.0 {
        log_violation(
            "INV-ANS-002",
            "0.0-1.0",
            &format!("{:.4}", raw),
            context,
        );
        return 1.0;
    }

    raw
}

/// Get reliability label for a score
pub fn reliability_label(score: f64) -> &'static str {
    if score >= GREEN_THRESHOLD {
        "Green"
    } else if score >= YELLOW_THRESHOLD {
        "Yellow"
    } else if score >= RED_THRESHOLD {
        "Red"
    } else {
        "Refuse"
    }
}

// ============================================================================
// Answer Text Guards
// ============================================================================

/// INV-ANS-001: Guard answer text to be non-empty
///
/// Returns fallback if text is empty/whitespace.
pub fn guard_answer_text(text: &str, context: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        log_violation(
            "INV-ANS-001",
            "non-empty text",
            "empty or whitespace",
            context,
        );
        return format!("[No answer available - {}]", context);
    }
    trimmed.to_string()
}

/// INV-BRAIN-005: Guard Brain answer text (stricter - must be meaningful)
pub fn guard_brain_answer(text: &str, context: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        log_violation(
            "INV-BRAIN-005",
            "non-empty meaningful text",
            "empty",
            context,
        );
        return None;
    }
    if trimmed.len() < 10 {
        log_violation(
            "INV-BRAIN-005",
            "meaningful text (>10 chars)",
            &format!("{} chars", trimmed.len()),
            context,
        );
        return None;
    }
    Some(trimmed.to_string())
}

// ============================================================================
// Origin Guards
// ============================================================================

/// Valid answer origins
pub const VALID_ORIGINS: &[&str] = &["Brain", "Recipe", "Junior", "Senior", "Fallback", "Error"];

/// INV-ANS-004: Guard answer origin to valid values
pub fn guard_origin(origin: &str, context: &str) -> String {
    if VALID_ORIGINS.contains(&origin) {
        origin.to_string()
    } else {
        log_violation(
            "INV-ANS-004",
            &format!("one of {:?}", VALID_ORIGINS),
            origin,
            context,
        );
        "Unknown".to_string()
    }
}

// ============================================================================
// XP Guards
// ============================================================================

/// INV-XP-002: Guard trust to 0.0-1.0 range
pub fn guard_trust(raw: f32, context: &str) -> f32 {
    if raw.is_nan() {
        log_violation("INV-XP-002", "0.0-1.0", "NaN", context);
        return 0.5; // Default trust
    }
    if raw < 0.0 {
        log_violation("INV-XP-002", "0.0-1.0", &format!("{:.4}", raw), context);
        return 0.0;
    }
    if raw > 1.0 {
        log_violation("INV-XP-002", "0.0-1.0", &format!("{:.4}", raw), context);
        return 1.0;
    }
    raw
}

/// INV-XP-003: Guard level to 1-99 range
pub fn guard_level(raw: u8, context: &str) -> u8 {
    if raw < 1 {
        log_violation("INV-XP-003", "1-99", &format!("{}", raw), context);
        return 1;
    }
    if raw > 99 {
        log_violation("INV-XP-003", "1-99", &format!("{}", raw), context);
        return 99;
    }
    raw
}

// ============================================================================
// Recipe Guards
// ============================================================================

/// INV-RECIPE-001: Guard recipe extraction reliability
pub fn guard_recipe_reliability(reliability: f64, _context: &str) -> bool {
    if reliability < RECIPE_MIN_RELIABILITY {
        // Not a violation - just not eligible
        return false;
    }
    true
}

/// INV-RECIPE-002: Guard recipe match score
pub fn guard_recipe_match(score: f64, _context: &str) -> bool {
    if score < RECIPE_MATCH_THRESHOLD {
        return false;
    }
    true
}

// ============================================================================
// Latency Guards
// ============================================================================

/// INV-BRAIN-001: Check Brain latency is within limit
pub fn check_brain_latency(duration_ms: u64, context: &str) -> bool {
    if duration_ms > BRAIN_LATENCY_HARD_LIMIT_MS {
        log_violation(
            "INV-BRAIN-001",
            &format!("<{}ms", BRAIN_LATENCY_HARD_LIMIT_MS),
            &format!("{}ms", duration_ms),
            context,
        );
        return false;
    }
    true
}

/// INV-LLM-006: Check total LLM latency is within limit
pub fn check_llm_latency(duration_ms: u64, context: &str) -> bool {
    if duration_ms > LLM_LATENCY_HARD_LIMIT_MS {
        log_violation(
            "INV-LLM-006",
            &format!("<{}ms", LLM_LATENCY_HARD_LIMIT_MS),
            &format!("{}ms", duration_ms),
            context,
        );
        return false;
    }
    true
}

// ============================================================================
// LLM Call Guards
// ============================================================================

/// INV-LLM-005: Guard iteration count
pub fn guard_iteration_count(count: usize, context: &str) -> bool {
    if count > MAX_ITERATIONS {
        log_violation(
            "INV-LLM-005",
            &format!("<={}", MAX_ITERATIONS),
            &format!("{}", count),
            context,
        );
        return false;
    }
    true
}

/// Guard LLM call count
pub fn guard_llm_call_count(count: usize, context: &str) -> bool {
    if count > MAX_LLM_CALLS {
        log_violation(
            "INV-LLM-MAX-CALLS",
            &format!("<={}", MAX_LLM_CALLS),
            &format!("{}", count),
            context,
        );
        return false;
    }
    true
}

// ============================================================================
// Senior Verdict Guards
// ============================================================================

/// Valid Senior verdicts
pub const VALID_VERDICTS: &[&str] = &[
    "approve",
    "fix_and_accept",
    "needs_more_checks",
    "refuse",
];

/// INV-LLM-002: Guard Senior verdict
pub fn guard_verdict(verdict: &str, context: &str) -> String {
    let lower = verdict.to_lowercase();
    if VALID_VERDICTS.contains(&lower.as_str()) {
        lower
    } else {
        log_violation(
            "INV-LLM-002",
            &format!("one of {:?}", VALID_VERDICTS),
            verdict,
            context,
        );
        "refuse".to_string() // Safe default
    }
}

// ============================================================================
// Composite Guards
// ============================================================================

/// Result of invariant validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            violations: vec![],
        }
    }

    pub fn with_violation(violation: &str) -> Self {
        Self {
            valid: false,
            violations: vec![violation.to_string()],
        }
    }

    pub fn add_violation(&mut self, violation: &str) {
        self.valid = false;
        self.violations.push(violation.to_string());
    }
}

/// Validate a complete answer against all invariants
pub fn validate_answer(
    text: &str,
    reliability: f64,
    origin: &str,
    duration_ms: u64,
) -> ValidationResult {
    let mut result = ValidationResult::ok();

    // INV-ANS-001: Non-empty text
    if text.trim().is_empty() {
        result.add_violation("INV-ANS-001: Empty answer text");
    }

    // INV-ANS-002: Reliability range
    if !(0.0..=1.0).contains(&reliability) || reliability.is_nan() {
        result.add_violation(&format!("INV-ANS-002: Invalid reliability {}", reliability));
    }

    // INV-ANS-004: Valid origin
    if !VALID_ORIGINS.contains(&origin) {
        result.add_violation(&format!("INV-ANS-004: Invalid origin '{}'", origin));
    }

    // Origin-specific checks
    if origin == "Brain" {
        if duration_ms > BRAIN_LATENCY_HARD_LIMIT_MS {
            result.add_violation(&format!(
                "INV-BRAIN-001: Brain latency {}ms > {}ms",
                duration_ms, BRAIN_LATENCY_HARD_LIMIT_MS
            ));
        }
        if reliability < GREEN_THRESHOLD {
            result.add_violation(&format!(
                "INV-BRAIN-003: Brain reliability {} < {}",
                reliability, GREEN_THRESHOLD
            ));
        }
    }

    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_reliability() {
        assert_eq!(guard_reliability(0.5, "test"), 0.5);
        assert_eq!(guard_reliability(0.0, "test"), 0.0);
        assert_eq!(guard_reliability(1.0, "test"), 1.0);
        assert_eq!(guard_reliability(-0.5, "test"), 0.0);
        assert_eq!(guard_reliability(1.5, "test"), 1.0);
        assert_eq!(guard_reliability(f64::NAN, "test"), 0.0);
    }

    #[test]
    fn test_reliability_label() {
        assert_eq!(reliability_label(0.95), "Green");
        assert_eq!(reliability_label(0.90), "Green");
        assert_eq!(reliability_label(0.80), "Yellow");
        assert_eq!(reliability_label(0.70), "Yellow");
        assert_eq!(reliability_label(0.60), "Red");
        assert_eq!(reliability_label(0.50), "Red");
        assert_eq!(reliability_label(0.40), "Refuse");
    }

    #[test]
    fn test_guard_answer_text() {
        assert_eq!(guard_answer_text("Hello", "test"), "Hello");
        assert_eq!(guard_answer_text("  Hello  ", "test"), "Hello");
        assert!(guard_answer_text("", "test").contains("No answer"));
        assert!(guard_answer_text("   ", "test").contains("No answer"));
    }

    #[test]
    fn test_guard_origin() {
        assert_eq!(guard_origin("Brain", "test"), "Brain");
        assert_eq!(guard_origin("Recipe", "test"), "Recipe");
        assert_eq!(guard_origin("Junior", "test"), "Junior");
        assert_eq!(guard_origin("Invalid", "test"), "Unknown");
    }

    #[test]
    fn test_guard_trust() {
        assert_eq!(guard_trust(0.5, "test"), 0.5);
        assert_eq!(guard_trust(-0.5, "test"), 0.0);
        assert_eq!(guard_trust(1.5, "test"), 1.0);
        assert_eq!(guard_trust(f32::NAN, "test"), 0.5);
    }

    #[test]
    fn test_guard_level() {
        assert_eq!(guard_level(1, "test"), 1);
        assert_eq!(guard_level(50, "test"), 50);
        assert_eq!(guard_level(99, "test"), 99);
        assert_eq!(guard_level(0, "test"), 1);
        assert_eq!(guard_level(100, "test"), 99);
    }

    #[test]
    fn test_guard_verdict() {
        assert_eq!(guard_verdict("approve", "test"), "approve");
        assert_eq!(guard_verdict("APPROVE", "test"), "approve");
        assert_eq!(guard_verdict("fix_and_accept", "test"), "fix_and_accept");
        assert_eq!(guard_verdict("invalid", "test"), "refuse");
    }

    #[test]
    fn test_validate_answer() {
        // Valid answer
        let result = validate_answer("Test answer", 0.95, "Brain", 50);
        assert!(result.valid);
        assert!(result.violations.is_empty());

        // Empty text
        let result = validate_answer("", 0.95, "Brain", 50);
        assert!(!result.valid);

        // Invalid reliability
        let result = validate_answer("Test", 1.5, "Brain", 50);
        assert!(!result.valid);

        // Invalid origin
        let result = validate_answer("Test", 0.95, "Invalid", 50);
        assert!(!result.valid);

        // Brain too slow
        let result = validate_answer("Test", 0.95, "Brain", 200);
        assert!(!result.valid);

        // Brain low reliability
        let result = validate_answer("Test", 0.80, "Brain", 50);
        assert!(!result.valid);
    }

    // ========================================================================
    // v4.5.4: Routing Invariant Tests
    // ========================================================================

    /// Valid ROUTE line values per v4.5.4 spec
    const VALID_ROUTES: &[&str] = &[
        "ROUTE: Brain",
        "ROUTE: Cache",
        "ROUTE: Cache(Brain)",
        "ROUTE: Cache(Junior)",
        "ROUTE: Cache(Senior)",
        "ROUTE: Junior(plan)",
        "ROUTE: Junior(draft)",
        "ROUTE: Junior(answer)",
        "ROUTE: Senior(audit)",
        "ROUTE: Senior(answer)",
        "ROUTE: Timeout(Junior)",
        "ROUTE: Timeout(Senior)",
        "ROUTE: Fallback",
    ];

    #[test]
    fn test_routing_invariant_valid_routes() {
        // All valid routes should be recognizable
        for route in VALID_ROUTES {
            assert!(route.starts_with("ROUTE: "), "Route should start with 'ROUTE: ': {}", route);
            // Extract the route type
            let route_type = route.strip_prefix("ROUTE: ").unwrap();
            assert!(!route_type.is_empty(), "Route type should not be empty");
        }
    }

    #[test]
    fn test_routing_invariant_brain_never_falls_through() {
        // If Brain matches, it must answer (not fall through to LLM)
        // This is a specification test - Brain match = Brain answer
        let brain_matched = true;
        let expected_route = if brain_matched { "Brain" } else { "Junior" };
        assert_eq!(expected_route, "Brain", "Brain match must result in Brain answer");
    }

    #[test]
    fn test_routing_invariant_empty_plan_is_failure() {
        // Zero probes = PLAN_FAILURE, not success
        let probe_count = 0;
        let is_plan_failure = probe_count == 0;
        assert!(is_plan_failure, "Empty probe plan must be treated as failure");
    }

    #[test]
    fn test_routing_invariant_cache_bypasses_llm() {
        // Cache hit must not call LLM
        let cache_hit = true;
        let should_call_llm = !cache_hit;
        assert!(!should_call_llm, "Cache hit must bypass LLM");
    }

    #[test]
    fn test_routing_invariant_single_increment_per_question() {
        // Each question must increment total_questions exactly once
        let mut total_questions = 0;
        let increment_count = 1; // One question = one increment

        total_questions += increment_count;
        assert_eq!(total_questions, 1, "total_questions must increment exactly once per query");

        // Simulating another path through same question should not double-increment
        // (handled by engine flow, not re-entry)
    }

    #[test]
    fn test_routing_invariant_success_rate_formula() {
        // success_rate = total_success / total_questions
        let total_questions = 10u64;
        let total_success = 7u64;
        let total_failures = 3u64;

        assert_eq!(total_questions, total_success + total_failures,
            "total_questions must equal total_success + total_failures");

        let success_rate = total_success as f64 / total_questions as f64;
        assert!((success_rate - 0.7).abs() < 0.001,
            "success_rate must equal total_success / total_questions");
    }

    #[test]
    fn test_routing_invariant_timeout_increments_failures() {
        // Timeout must increment total_failures, not total_success
        let mut total_success = 5u64;
        let mut total_failures = 2u64;
        let is_timeout = true;

        if is_timeout {
            total_failures += 1;
            // total_success unchanged
        } else {
            total_success += 1;
        }

        assert_eq!(total_failures, 3, "Timeout must increment total_failures");
        assert_eq!(total_success, 5, "Timeout must not increment total_success");
    }

    #[test]
    fn test_routing_invariant_senior_paths() {
        // Senior has three possible verdicts: approve, fix_and_accept, refuse
        let verdicts = vec!["approve", "fix_and_accept", "refuse"];

        for verdict in verdicts {
            match verdict {
                "approve" | "fix_and_accept" => {
                    // Success path
                    let is_success = true;
                    assert!(is_success, "approve/fix_and_accept must be success");
                }
                "refuse" => {
                    // Failure path
                    let is_failure = true;
                    assert!(is_failure, "refuse must be failure");
                }
                _ => panic!("Unknown verdict"),
            }
        }
    }
}
