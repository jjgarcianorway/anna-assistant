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
    if reliability < 0.0 || reliability > 1.0 || reliability.is_nan() {
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
}
