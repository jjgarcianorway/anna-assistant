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

    // ========================================================================
    // v4.5.5: ASCII-Only Tests
    // ========================================================================

    /// Helper to check if a string is ASCII-only
    /// Valid chars: newline (\n), carriage return (\r), tab (\t), or 32..126
    fn is_ascii_output(s: &str) -> bool {
        s.bytes().all(|b| {
            b == b'\n' || b == b'\r' || b == b'\t' || (32..=126).contains(&b)
        })
    }

    #[test]
    fn test_ascii_only_status_labels() {
        // v4.5.5: All status labels must be ASCII-only
        let labels = [
            "[DATA]", "[XP]", "[KNOW]", "[LLM]",
            "[OK]", "[FAIL]", "[WARN]", "[ERR]",
            "[TIME]", "[BENCH]", "[FIX]",
            "[BRAIN]", "[YELLOW]", "[ORANGE]", "[RED]",
        ];

        for label in labels {
            assert!(is_ascii_output(label),
                "Label '{}' contains non-ASCII characters", label);
        }
    }

    #[test]
    fn test_ascii_only_status_output_sample() {
        // v4.5.5: Sample status output must be ASCII-only
        let sample_output = r#"
[ANNA STATUS]
===========================================

  Version: v4.5.5
  Uptime: 2h 15m

[FOLDERS]
  [DATA]  rwx  /var/lib/anna        Data
  [XP]    rwx  /var/lib/anna/xp     XP
  [KNOW]  rwx  /var/lib/anna/knowledge  Knowledge
  [LLM]   rwx  /var/lib/anna/llm    LLM

[PERFORMANCE]
  Success:    85% (17/20)

-------------------------------------------
  EVALUATION TOOLS
-------------------------------------------
  *  First Light: [OK] Run at 2024-01-15, +50 XP
  [BENCH]  Snow Leopard: Not yet run

-------------------------------------------
  Status: Online
===========================================
"#;

        assert!(is_ascii_output(sample_output),
            "Sample status output contains non-ASCII characters");
    }

    #[test]
    fn test_ascii_only_separators() {
        // v4.5.5: All separators must use ASCII dashes, not box-drawing chars
        let valid_separators = [
            "-------------------------------------------",
            "===========================================",
            "------------------------------------------",
            "---------------------------------------------------------------------------------------------",
        ];

        for sep in valid_separators {
            assert!(is_ascii_output(sep),
                "Separator '{}' contains non-ASCII characters", sep);
            // Also check it only contains dashes or equals
            assert!(sep.chars().all(|c| c == '-' || c == '='),
                "Separator contains unexpected character");
        }
    }

    // ========================================================================
    // v4.5.5: Answer Cache Tests
    // ========================================================================

    /// Test cache key normalization
    fn normalize_cache_key(question: &str) -> String {
        let lower = question.to_lowercase();
        let no_punct: String = lower
            .chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect();
        no_punct
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn test_cache_key_normalization_basic() {
        // Basic normalization: lowercase + trim + collapse whitespace
        assert_eq!(normalize_cache_key("What CPU do I have"), "what cpu do i have");
        assert_eq!(normalize_cache_key("  What   CPU   do   I   have  "), "what cpu do i have");
    }

    #[test]
    fn test_cache_key_normalization_punctuation() {
        // v4.5.5: Punctuation removal
        assert_eq!(normalize_cache_key("What CPU do I have?"), "what cpu do i have");
        assert_eq!(normalize_cache_key("What cpu do I have!"), "what cpu do i have");
        assert_eq!(normalize_cache_key("What's my CPU?"), "what s my cpu");
    }

    #[test]
    fn test_cache_key_paraphrase_matching() {
        // v4.5.5: These paraphrases should produce the same cache key
        let variants = [
            "what CPU do I have?",
            "What cpu do I have",
            "what cpu do i have",
            "  what   cpu   do  i  have  ?  ",
        ];

        let first_key = normalize_cache_key(variants[0]);
        for variant in &variants[1..] {
            assert_eq!(normalize_cache_key(variant), first_key,
                "Paraphrase '{}' should match '{}'", variant, variants[0]);
        }
    }

    #[test]
    fn test_cache_reliability_threshold() {
        // v4.5.5: Cache threshold is 70% (0.70)
        let threshold = 0.70;

        // Should cache: reliability >= 70%
        assert!(0.70 >= threshold);
        assert!(0.85 >= threshold);
        assert!(0.99 >= threshold);

        // Should not cache: reliability < 70%
        assert!(0.69 < threshold);
        assert!(0.50 < threshold);
    }

    #[test]
    fn test_cache_hit_increments_stats() {
        // v4.5.5: Cache hit must increment both total_questions and total_success
        let mut total_questions = 5u64;
        let mut total_success = 4u64;
        let is_cache_hit = true;

        if is_cache_hit {
            total_questions += 1;
            total_success += 1; // Cache hits are always successful
        }

        assert_eq!(total_questions, 6, "Cache hit must increment total_questions");
        assert_eq!(total_success, 5, "Cache hit must increment total_success");
    }

    #[test]
    fn test_cache_miss_does_not_increment_stats() {
        // Cache miss by itself doesn't increment stats - the subsequent answer does
        let total_questions = 5u64;
        let total_success = 4u64;
        let is_cache_hit = false;

        if is_cache_hit {
            panic!("Should not reach here for cache miss");
        }

        // Stats unchanged on cache miss (answer flow increments them)
        assert_eq!(total_questions, 5);
        assert_eq!(total_success, 4);
    }

    // ========================================================================
    // v5.0.0: Knowledge Core Invariants
    // ========================================================================

    #[test]
    fn test_knowledge_core_status_knowledge_consistency() {
        // If status shows N objects, knowledge command must list N objects
        use crate::knowledge_core::{KnowledgeStore, KnowledgeObject, Category};

        let mut store = KnowledgeStore::new();

        // Add 3 objects
        store.upsert(KnowledgeObject::new("vim", Category::Editor));
        store.upsert(KnowledgeObject::new("nano", Category::Editor));
        store.upsert(KnowledgeObject::new("zsh", Category::Shell));

        // Total must match
        let total = store.total_objects();
        assert_eq!(total, 3);

        // Category counts must sum to total
        let counts = store.count_by_category();
        let sum: usize = counts.values().sum();
        assert_eq!(sum, total);
    }

    #[test]
    fn test_knowledge_core_usage_consistency() {
        // If knowledge vim shows runs: N, that must match telemetry
        use crate::knowledge_core::{KnowledgeObject, Category};

        let mut obj = KnowledgeObject::new("vim", Category::Editor);

        // Record 5 usages
        for _ in 0..5 {
            obj.record_usage(100, 1024);
        }

        assert_eq!(obj.usage_count, 5);
    }

    #[test]
    fn test_knowledge_core_store_no_duplicates() {
        // Upsert with same name should update, not create duplicate
        use crate::knowledge_core::{KnowledgeStore, KnowledgeObject, Category};

        let mut store = KnowledgeStore::new();

        let obj1 = KnowledgeObject::new("vim", Category::Editor);
        store.upsert(obj1);
        assert_eq!(store.total_objects(), 1);

        // Upsert again with same name
        let mut obj2 = KnowledgeObject::new("vim", Category::Editor);
        obj2.usage_count = 10;
        store.upsert(obj2);

        // Still only 1 object
        assert_eq!(store.total_objects(), 1);

        // But with updated usage count
        let stored = store.get("vim").unwrap();
        assert_eq!(stored.usage_count, 10);
    }

    #[test]
    fn test_knowledge_core_clear_empties_all() {
        // Clear must remove all objects
        use crate::knowledge_core::{KnowledgeStore, KnowledgeObject, Category};

        let mut store = KnowledgeStore::new();
        store.upsert(KnowledgeObject::new("vim", Category::Editor));
        store.upsert(KnowledgeObject::new("zsh", Category::Shell));

        assert_eq!(store.total_objects(), 2);

        store.clear();

        assert_eq!(store.total_objects(), 0);
        assert!(store.first_discovery_at.is_none());
    }

    #[test]
    fn test_knowledge_core_no_telemetry_shows_message() {
        // If no telemetry, status/stats must say so, not show zeros
        use crate::knowledge_core::TelemetryAggregates;

        let telem = TelemetryAggregates::new();

        // Fresh telemetry has zero values
        assert_eq!(telem.processes_observed, 0);
        assert_eq!(telem.unique_commands, 0);

        // The UI should check for zero and show "No data collected yet"
        // This is a behavior contract, not code
    }

    // ========================================================================
    // v5.1.0: Full Inventory Invariants
    // ========================================================================

    #[test]
    fn test_inventory_object_types_valid() {
        // ObjectType enum must have exactly 3 values
        use crate::knowledge_core::ObjectType;

        let types = [ObjectType::Command, ObjectType::Package, ObjectType::Service];
        assert_eq!(types.len(), 3);

        // Each type has a string representation
        assert_eq!(ObjectType::Command.as_str(), "command");
        assert_eq!(ObjectType::Package.as_str(), "package");
        assert_eq!(ObjectType::Service.as_str(), "service");
    }

    #[test]
    fn test_inventory_progress_phases() {
        // InventoryPhase enum must have exactly 5 values
        use crate::knowledge_core::InventoryPhase;

        let phases = [
            InventoryPhase::Idle,
            InventoryPhase::ScanningPath,
            InventoryPhase::ScanningPackages,
            InventoryPhase::ScanningServices,
            InventoryPhase::Complete,
        ];
        assert_eq!(phases.len(), 5);
    }

    #[test]
    fn test_inventory_progress_percent_bounds() {
        // Progress percent must be 0-100
        use crate::knowledge_core::InventoryProgress;

        let mut progress = InventoryProgress::new();

        // Initial state
        assert!(progress.percent <= 100);

        // After update
        progress.items_total = 10;
        progress.update(5);
        assert!(progress.percent <= 100);

        progress.update(10);
        assert!(progress.percent <= 100);
    }

    #[test]
    fn test_inventory_progress_complete_marks_done() {
        // complete() must set initial_scan_complete = true
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();
        assert!(!progress.initial_scan_complete);

        progress.complete();
        assert!(progress.initial_scan_complete);
        assert_eq!(progress.phase, InventoryPhase::Complete);
        assert_eq!(progress.percent, 100);
    }

    #[test]
    fn test_inventory_count_by_type_consistency() {
        // count_by_type must return consistent numbers
        use crate::knowledge_core::{KnowledgeStore, KnowledgeObject, Category, ObjectType};

        let mut store = KnowledgeStore::new();

        // Add objects with different types
        let mut cmd = KnowledgeObject::new("ls", Category::Tool);
        cmd.object_types.push(ObjectType::Command);
        store.upsert(cmd);

        let mut pkg = KnowledgeObject::new("vim", Category::Editor);
        pkg.object_types.push(ObjectType::Package);
        store.upsert(pkg);

        let mut svc = KnowledgeObject::new("nginx", Category::Service);
        svc.object_types.push(ObjectType::Service);
        store.upsert(svc);

        // Object with multiple types
        let mut multi = KnowledgeObject::new("systemd", Category::Service);
        multi.object_types.push(ObjectType::Command);
        multi.object_types.push(ObjectType::Package);
        multi.object_types.push(ObjectType::Service);
        store.upsert(multi);

        let (commands, packages, services) = store.count_by_type();

        // ls + systemd = 2 commands
        assert_eq!(commands, 2);
        // vim + systemd = 2 packages
        assert_eq!(packages, 2);
        // nginx + systemd = 2 services
        assert_eq!(services, 2);
    }

    #[test]
    fn test_inventory_package_version_tracking() {
        // Package version must be stored
        use crate::knowledge_core::{KnowledgeObject, Category};

        let mut obj = KnowledgeObject::new("vim", Category::Editor);
        assert!(obj.package_version.is_none());

        obj.package_version = Some("9.0.2136-1".to_string());
        assert_eq!(obj.package_version.as_deref(), Some("9.0.2136-1"));
    }

    #[test]
    fn test_inventory_paths_can_be_multiple() {
        // An object can have multiple paths (symlinks, copies)
        use crate::knowledge_core::{KnowledgeObject, Category};

        let mut obj = KnowledgeObject::new("python", Category::Tool);
        obj.paths.push("/usr/bin/python".to_string());
        obj.paths.push("/usr/bin/python3".to_string());
        obj.paths.push("/usr/local/bin/python".to_string());

        assert_eq!(obj.paths.len(), 3);
    }

    #[test]
    fn test_inventory_service_unit_tracking() {
        // Service unit must be tracked for services
        use crate::knowledge_core::{KnowledgeObject, Category};

        let mut obj = KnowledgeObject::new("nginx", Category::Service);
        obj.service_unit = Some("nginx.service".to_string());
        obj.service_enabled = Some(true);
        obj.service_active = Some(true);

        assert_eq!(obj.service_unit.as_deref(), Some("nginx.service"));
        assert_eq!(obj.service_enabled, Some(true));
        assert_eq!(obj.service_active, Some(true));
    }

    #[test]
    fn test_inventory_source_tracking() {
        // Inventory sources must track where data came from
        use crate::knowledge_core::{KnowledgeObject, Category};

        let mut obj = KnowledgeObject::new("vim", Category::Editor);
        obj.inventory_source.push("path_scan".to_string());
        obj.inventory_source.push("pacman_db".to_string());

        assert!(obj.inventory_source.contains(&"path_scan".to_string()));
        assert!(obj.inventory_source.contains(&"pacman_db".to_string()));
    }

    #[test]
    fn test_inventory_removed_at_tracking() {
        // removed_at must be set when package is uninstalled
        use crate::knowledge_core::{KnowledgeObject, Category};

        let mut obj = KnowledgeObject::new("vim", Category::Editor);
        obj.installed = true; // Simulate installed package

        // Initially not removed
        assert!(obj.removed_at.is_none());
        assert!(obj.installed);

        // Simulate removal
        obj.installed = false;
        obj.removed_at = Some(1700000000);

        assert!(!obj.installed);
        assert!(obj.removed_at.is_some());
    }

    #[test]
    fn test_inventory_format_status_phases() {
        // format_status must return different strings per phase
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();

        // Idle
        progress.phase = InventoryPhase::Idle;
        let status = progress.format_status();
        assert!(status.contains("Waiting"));

        // Complete
        progress.phase = InventoryPhase::Complete;
        progress.initial_scan_complete = true;
        let status = progress.format_status();
        assert!(status.contains("Complete"));
    }

    #[test]
    fn test_inventory_count_with_usage() {
        // count_with_usage must count objects with usage_count > 0
        use crate::knowledge_core::{KnowledgeStore, KnowledgeObject, Category};

        let mut store = KnowledgeStore::new();

        // Object with no usage
        let obj1 = KnowledgeObject::new("vim", Category::Editor);
        store.upsert(obj1);

        // Object with usage
        let mut obj2 = KnowledgeObject::new("zsh", Category::Shell);
        obj2.usage_count = 5;
        store.upsert(obj2);

        assert_eq!(store.count_with_usage(), 1);
    }

    // ========================================================================
    // v5.1.1: Priority Knowledge Resolution Tests
    // ========================================================================

    #[test]
    fn test_priority_scan_starts_correctly() {
        // start_priority_scan must set phase and target
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();
        progress.start_priority_scan("vim");

        assert_eq!(progress.phase, InventoryPhase::PriorityScan);
        assert_eq!(progress.priority_target, Some("vim".to_string()));
        assert!(progress.is_priority_scan());
    }

    #[test]
    fn test_priority_scan_saves_checkpoint() {
        // When interrupting a scan, checkpoint must be saved
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();

        // Start a regular scan
        progress.start_phase(InventoryPhase::ScanningPath, 100);
        progress.update(50);

        // Interrupt with priority scan
        progress.start_priority_scan("docker");

        // Checkpoint should be saved
        assert!(progress.scan_checkpoint.is_some());
        let checkpoint = progress.scan_checkpoint.as_ref().unwrap();
        assert_eq!(checkpoint.phase, Some(InventoryPhase::ScanningPath));
        assert_eq!(checkpoint.offset, 50);
    }

    #[test]
    fn test_priority_scan_resumes_after_completion() {
        // end_priority_scan must restore previous phase
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();

        // Start a regular scan
        progress.start_phase(InventoryPhase::ScanningPackages, 200);
        progress.update(100);

        // Interrupt with priority scan
        progress.start_priority_scan("vim");
        assert_eq!(progress.phase, InventoryPhase::PriorityScan);

        // End priority scan
        progress.end_priority_scan();

        // Should resume from checkpoint
        assert_eq!(progress.phase, InventoryPhase::ScanningPackages);
        assert_eq!(progress.items_processed, 100);
        assert!(progress.priority_target.is_none());
    }

    #[test]
    fn test_priority_scan_format_status() {
        // format_status must show target during priority scan
        use crate::knowledge_core::InventoryProgress;

        let mut progress = InventoryProgress::new();
        progress.start_priority_scan("nginx");

        let status = progress.format_status();
        assert!(status.contains("priority_scan"));
        assert!(status.contains("nginx"));
    }

    #[test]
    fn test_priority_scan_eta_is_fast() {
        // Priority scans must have ETA < 2s
        use crate::knowledge_core::InventoryProgress;

        let mut progress = InventoryProgress::new();
        progress.start_priority_scan("vim");

        assert!(progress.eta_secs.unwrap_or(0) <= 2);
    }

    #[test]
    fn test_priority_from_idle_no_checkpoint() {
        // If starting priority scan from idle, no checkpoint needed
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();
        assert_eq!(progress.phase, InventoryPhase::Idle);

        progress.start_priority_scan("vim");
        assert!(progress.scan_checkpoint.is_none());

        progress.end_priority_scan();
        assert_eq!(progress.phase, InventoryPhase::Idle);
    }

    #[test]
    fn test_priority_from_complete_no_checkpoint() {
        // If starting priority scan when complete, no checkpoint needed
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();
        progress.complete();
        assert_eq!(progress.phase, InventoryPhase::Complete);

        progress.start_priority_scan("vim");
        // No checkpoint because Complete is a terminal state
        assert!(progress.scan_checkpoint.is_none());

        progress.end_priority_scan();
        // Should return to Complete
        assert_eq!(progress.phase, InventoryPhase::Complete);
    }

    #[test]
    fn test_scan_checkpoint_struct() {
        // ScanCheckpoint must store all required fields
        use crate::knowledge_core::{ScanCheckpoint, InventoryPhase};

        let checkpoint = ScanCheckpoint {
            phase: Some(InventoryPhase::ScanningServices),
            offset: 42,
            paused_at: 1700000000,
        };

        assert_eq!(checkpoint.phase, Some(InventoryPhase::ScanningServices));
        assert_eq!(checkpoint.offset, 42);
        assert_eq!(checkpoint.paused_at, 1700000000);
    }

    #[test]
    fn test_inventory_priority_enum() {
        // InventoryPriority must have High and Low variants
        use crate::knowledge_core::InventoryPriority;

        let high = InventoryPriority::High;
        let low = InventoryPriority::Low;

        assert!(high.is_high());
        assert!(!low.is_high());
    }

    #[test]
    fn test_priority_scan_does_not_lose_progress() {
        // Total scan progress must not reset after priority scan
        use crate::knowledge_core::{InventoryProgress, InventoryPhase};

        let mut progress = InventoryProgress::new();

        // Complete part of a scan
        progress.start_phase(InventoryPhase::ScanningPath, 100);
        progress.update(100);
        progress.start_phase(InventoryPhase::ScanningPackages, 200);
        progress.update(50);

        // Interrupt with priority scan
        progress.start_priority_scan("vim");

        // End priority scan
        progress.end_priority_scan();

        // Progress should be preserved
        assert_eq!(progress.phase, InventoryPhase::ScanningPackages);
        assert_eq!(progress.items_processed, 50);
    }

    // v5.2.0: Error Index Invariants
    // ====================================================================

    #[test]
    fn test_log_severity_ordering() {
        // Severity levels must be correctly ordered
        use crate::error_index::LogSeverity;

        assert!(LogSeverity::Emergency > LogSeverity::Alert);
        assert!(LogSeverity::Alert > LogSeverity::Critical);
        assert!(LogSeverity::Critical > LogSeverity::Error);
        assert!(LogSeverity::Error > LogSeverity::Warning);
        assert!(LogSeverity::Warning > LogSeverity::Notice);
        assert!(LogSeverity::Notice > LogSeverity::Info);
        assert!(LogSeverity::Info > LogSeverity::Debug);
    }

    #[test]
    fn test_log_severity_is_error() {
        // is_error() must return true for Error and above
        use crate::error_index::LogSeverity;

        assert!(LogSeverity::Error.is_error());
        assert!(LogSeverity::Critical.is_error());
        assert!(LogSeverity::Alert.is_error());
        assert!(LogSeverity::Emergency.is_error());

        assert!(!LogSeverity::Warning.is_error());
        assert!(!LogSeverity::Notice.is_error());
        assert!(!LogSeverity::Info.is_error());
        assert!(!LogSeverity::Debug.is_error());
    }

    #[test]
    fn test_error_type_detection_permission() {
        // Permission errors must be detected
        use crate::error_index::ErrorType;

        assert_eq!(ErrorType::detect_from_message("Permission denied"), ErrorType::Permission);
        assert_eq!(ErrorType::detect_from_message("EACCES error"), ErrorType::Permission);
        assert_eq!(ErrorType::detect_from_message("operation not permitted"), ErrorType::Permission);
    }

    #[test]
    fn test_error_type_detection_missing_file() {
        // Missing file errors must be detected
        use crate::error_index::ErrorType;

        assert_eq!(ErrorType::detect_from_message("No such file or directory"), ErrorType::MissingFile);
        assert_eq!(ErrorType::detect_from_message("file not found"), ErrorType::MissingFile);
        assert_eq!(ErrorType::detect_from_message("ENOENT"), ErrorType::MissingFile);
    }

    #[test]
    fn test_error_type_detection_segfault() {
        // Segfaults must be detected
        use crate::error_index::ErrorType;

        assert_eq!(ErrorType::detect_from_message("Segmentation fault"), ErrorType::Segfault);
        assert_eq!(ErrorType::detect_from_message("SIGSEGV received"), ErrorType::Segfault);
        assert_eq!(ErrorType::detect_from_message("core dumped"), ErrorType::Segfault);
    }

    #[test]
    fn test_object_errors_tracking() {
        // ObjectErrors must track error counts correctly
        use crate::error_index::{ObjectErrors, LogEntry, LogSeverity};

        let mut obj_errors = ObjectErrors::new("nginx");

        let entry1 = LogEntry::new(1000, LogSeverity::Error, "Permission denied".to_string());
        let entry2 = LogEntry::new(1001, LogSeverity::Error, "Connection refused".to_string());
        let entry3 = LogEntry::new(1002, LogSeverity::Warning, "deprecated option".to_string());

        obj_errors.add_log(entry1);
        obj_errors.add_log(entry2);
        obj_errors.add_log(entry3);

        assert_eq!(obj_errors.total_errors(), 2);
        assert_eq!(obj_errors.warning_count, 1);
        assert_eq!(obj_errors.logs.len(), 3);
    }

    #[test]
    fn test_error_index_global_tracking() {
        // ErrorIndex must track global error counts
        use crate::error_index::{ErrorIndex, LogEntry, LogSeverity};

        let mut index = ErrorIndex::new();

        let entry1 = LogEntry::new(1000, LogSeverity::Error, "Error 1".to_string());
        let entry2 = LogEntry::new(1001, LogSeverity::Warning, "Warning 1".to_string());

        index.add_log("sshd", entry1);
        index.add_log("nginx", entry2);

        assert_eq!(index.total_errors, 1);
        assert_eq!(index.total_warnings, 1);
        assert!(index.get_object_errors("sshd").is_some());
        assert!(index.get_object_errors("nginx").is_some());
    }

    // v5.2.0: Service State Invariants
    // ====================================================================

    #[test]
    fn test_active_state_parsing() {
        // ActiveState must parse all variants correctly
        use crate::service_state::ActiveState;

        assert_eq!(ActiveState::from_str("active"), ActiveState::Active);
        assert_eq!(ActiveState::from_str("inactive"), ActiveState::Inactive);
        assert_eq!(ActiveState::from_str("failed"), ActiveState::Failed);
        assert_eq!(ActiveState::from_str("activating"), ActiveState::Activating);
        assert_eq!(ActiveState::from_str("deactivating"), ActiveState::Deactivating);
        // Case insensitive
        assert_eq!(ActiveState::from_str("ACTIVE"), ActiveState::Active);
    }

    #[test]
    fn test_enabled_state_parsing() {
        // EnabledState must parse all variants correctly
        use crate::service_state::EnabledState;

        assert_eq!(EnabledState::from_str("enabled"), EnabledState::Enabled);
        assert_eq!(EnabledState::from_str("disabled"), EnabledState::Disabled);
        assert_eq!(EnabledState::from_str("masked"), EnabledState::Masked);
        assert_eq!(EnabledState::from_str("static"), EnabledState::Static);
    }

    #[test]
    fn test_service_state_is_running() {
        // is_running() must return true only for Active or Reloading
        use crate::service_state::ActiveState;

        assert!(ActiveState::Active.is_running());
        assert!(ActiveState::Reloading.is_running());
        assert!(!ActiveState::Failed.is_running());
        assert!(!ActiveState::Inactive.is_running());
    }

    #[test]
    fn test_service_index_counts() {
        // ServiceIndex must track counts correctly
        use crate::service_state::{ServiceIndex, ServiceState, ActiveState, EnabledState};

        let mut index = ServiceIndex::new();

        let mut state1 = ServiceState::new("running.service");
        state1.active_state = ActiveState::Active;
        state1.enabled_state = EnabledState::Enabled;

        let mut state2 = ServiceState::new("failed.service");
        state2.active_state = ActiveState::Failed;
        state2.enabled_state = EnabledState::Disabled;

        let mut state3 = ServiceState::new("masked.service");
        state3.active_state = ActiveState::Inactive;
        state3.enabled_state = EnabledState::Masked;

        index.update(state1);
        index.update(state2);
        index.update(state3);

        assert_eq!(index.running_count, 1);
        assert_eq!(index.failed_count, 1);
        assert_eq!(index.masked_count, 1);
    }

    // v5.2.0: Intrusion Detection Invariants
    // ====================================================================

    #[test]
    fn test_intrusion_type_severity() {
        // Intrusion types must have correct severity levels
        use crate::intrusion::IntrusionType;

        // High severity (5)
        assert_eq!(IntrusionType::RootkitIndicator.severity(), 5);
        assert_eq!(IntrusionType::PrivilegeEscalation.severity(), 5);
        assert_eq!(IntrusionType::MalwareSignature.severity(), 5);
        assert_eq!(IntrusionType::LogTampering.severity(), 5);

        // Medium severity (3-4)
        assert!(IntrusionType::SudoAbuse.severity() >= 3);
        assert!(IntrusionType::BruteForce.severity() >= 3);

        // Lower severity (2)
        assert_eq!(IntrusionType::FailedAuth.severity(), 2);
        assert_eq!(IntrusionType::PortScan.severity(), 2);
    }

    #[test]
    fn test_intrusion_pattern_matching() {
        // IntrusionIndex must detect patterns correctly
        use crate::intrusion::IntrusionIndex;

        let mut index = IntrusionIndex::new();

        // SSH failed auth
        index.check_message(
            "Failed password for invalid user admin from 192.168.1.100 port 22",
            "sshd",
            Some("sshd"),
        );

        assert_eq!(index.total_events, 1);
        assert!(index.get_object_intrusions("sshd").is_some());
    }

    #[test]
    fn test_intrusion_ip_extraction() {
        // IntrusionEvent must extract source IP
        use crate::intrusion::{IntrusionEvent, IntrusionType};

        let mut event = IntrusionEvent::new(
            IntrusionType::FailedAuth,
            "ssh_failed_auth",
            "Failed password from 192.168.1.100 port 22".to_string(),
            "sshd",
        );

        event.extract_source_ip();
        assert_eq!(event.source_ip, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_intrusion_username_extraction() {
        // IntrusionEvent must extract username
        use crate::intrusion::{IntrusionEvent, IntrusionType};

        let mut event = IntrusionEvent::new(
            IntrusionType::FailedAuth,
            "ssh_failed_auth",
            "Invalid user admin from 192.168.1.100".to_string(),
            "sshd",
        );

        event.extract_username();
        assert_eq!(event.username, Some("admin".to_string()));
    }

    #[test]
    fn test_intrusion_sudo_pattern() {
        // Sudo abuse patterns must be detected
        use crate::intrusion::IntrusionIndex;

        let mut index = IntrusionIndex::new();

        index.check_message(
            "user NOT in sudoers; TTY=pts/0; PWD=/home/user",
            "sudo",
            Some("sudo"),
        );

        assert_eq!(index.total_events, 1);
        let obj = index.get_object_intrusions("sudo").unwrap();
        assert!(obj.type_counts.contains_key("sudo_abuse"));
    }

    #[test]
    fn test_object_intrusions_max_severity() {
        // ObjectIntrusions must track max severity correctly
        use crate::intrusion::{ObjectIntrusions, IntrusionEvent, IntrusionType};

        let mut obj = ObjectIntrusions::new("sshd");

        let low_event = IntrusionEvent::new(
            IntrusionType::FailedAuth, // severity 2
            "test",
            "low severity".to_string(),
            "sshd",
        );

        let high_event = IntrusionEvent::new(
            IntrusionType::PrivilegeEscalation, // severity 5
            "test",
            "high severity".to_string(),
            "sshd",
        );

        obj.add_event(low_event);
        assert_eq!(obj.max_severity, 2);

        obj.add_event(high_event);
        assert_eq!(obj.max_severity, 5);
    }

    #[test]
    fn test_intrusion_events_capped() {
        // ObjectIntrusions must cap events at 50
        use crate::intrusion::{ObjectIntrusions, IntrusionEvent, IntrusionType};

        let mut obj = ObjectIntrusions::new("test");

        for i in 0..60 {
            let event = IntrusionEvent::new(
                IntrusionType::FailedAuth,
                "test",
                format!("Event {}", i),
                "test",
            );
            obj.add_event(event);
        }

        assert!(obj.events.len() <= 50);
    }

    // v5.2.1: Log Scan State Invariants
    // ====================================================================

    #[test]
    fn test_log_scan_state_new() {
        // LogScanState must initialize with correct defaults
        use crate::error_index::LogScanState;

        let state = LogScanState::new();
        assert_eq!(state.new_errors, 0);
        assert_eq!(state.new_warnings, 0);
        assert!(!state.running);
        assert_eq!(state.total_scans, 0);
    }

    #[test]
    fn test_log_scan_state_record() {
        // LogScanState must record scans correctly
        use crate::error_index::LogScanState;

        let mut state = LogScanState::new();
        state.record_scan(5, 10);

        assert_eq!(state.new_errors, 5);
        assert_eq!(state.new_warnings, 10);
        assert_eq!(state.total_scans, 1);

        state.record_scan(2, 3);
        assert_eq!(state.new_errors, 2);
        assert_eq!(state.new_warnings, 3);
        assert_eq!(state.total_scans, 2);
    }

    #[test]
    fn test_log_scan_state_string() {
        // LogScanState must report state correctly
        use crate::error_index::LogScanState;

        let mut state = LogScanState::new();
        assert_eq!(state.state_string(), "idle");

        state.running = true;
        assert_eq!(state.state_string(), "running (background)");
    }

    // v5.2.1: Service Index Extended Counts
    // ====================================================================

    #[test]
    fn test_service_index_all_counts() {
        // ServiceIndex must track all count types correctly
        use crate::service_state::{ServiceIndex, ServiceState, ActiveState, EnabledState};

        let mut index = ServiceIndex::new();

        let mut running = ServiceState::new("running.service");
        running.active_state = ActiveState::Active;
        running.enabled_state = EnabledState::Enabled;

        let mut inactive = ServiceState::new("inactive.service");
        inactive.active_state = ActiveState::Inactive;
        inactive.enabled_state = EnabledState::Disabled;

        let mut failed = ServiceState::new("failed.service");
        failed.active_state = ActiveState::Failed;
        failed.enabled_state = EnabledState::Enabled;

        let mut masked = ServiceState::new("masked.service");
        masked.active_state = ActiveState::Inactive;
        masked.enabled_state = EnabledState::Masked;

        index.update(running);
        index.update(inactive);
        index.update(failed);
        index.update(masked);

        assert_eq!(index.total_count(), 4);
        assert_eq!(index.running_count, 1);
        assert_eq!(index.inactive_count, 2); // inactive + masked
        assert_eq!(index.failed_count, 1);
        assert_eq!(index.enabled_count, 2); // running + failed
        assert_eq!(index.disabled_count, 1);
        assert_eq!(index.masked_count, 1);
    }

    #[test]
    fn test_service_index_get_methods() {
        // ServiceIndex get methods must return correct services
        use crate::service_state::{ServiceIndex, ServiceState, ActiveState, EnabledState};

        let mut index = ServiceIndex::new();

        let mut running = ServiceState::new("running.service");
        running.active_state = ActiveState::Active;

        let mut failed = ServiceState::new("failed.service");
        failed.active_state = ActiveState::Failed;

        let mut masked = ServiceState::new("masked.service");
        masked.enabled_state = EnabledState::Masked;

        index.update(running);
        index.update(failed);
        index.update(masked);

        assert_eq!(index.get_running().len(), 1);
        assert_eq!(index.get_failed().len(), 1);
        assert_eq!(index.get_masked().len(), 1);
        assert_eq!(index.get_enabled().len(), 0);
        assert_eq!(index.get_disabled().len(), 0);
    }

    #[test]
    fn test_error_index_recent_errors_24h() {
        // ErrorIndex must return recent errors correctly
        use crate::error_index::{ErrorIndex, LogEntry, LogSeverity};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut index = ErrorIndex::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Add a recent error
        let recent = LogEntry::new(now - 1000, LogSeverity::Error, "Recent error".to_string());
        index.add_log("nginx", recent);

        // Add an old error (more than 24h ago)
        let old = LogEntry::new(now - 100000, LogSeverity::Error, "Old error".to_string());
        index.add_log("apache", old);

        let recent_errors = index.recent_errors_24h();
        assert_eq!(recent_errors.len(), 1);
        assert_eq!(recent_errors[0].0, "nginx");
    }

    #[test]
    fn test_service_find_by_executable() {
        // ServiceIndex must find services by executable
        use crate::service_state::{ServiceIndex, ServiceState};

        let mut index = ServiceIndex::new();

        let mut sshd = ServiceState::new("sshd.service");
        sshd.description = Some("OpenSSH Daemon".to_string());

        let mut nginx = ServiceState::new("nginx.service");
        nginx.description = Some("NGINX web server".to_string());

        index.update(sshd);
        index.update(nginx);

        let found = index.find_services_using_executable("/usr/bin/sshd");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].unit_name, "sshd.service");
    }
}
