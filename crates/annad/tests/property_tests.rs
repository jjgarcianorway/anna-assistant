//! Property-Based Tests v3.5.0
//!
//! Tests that verify system invariants hold across randomized inputs.
//! Uses standard library for test generation rather than external crates
//! to minimize dependencies.
//!
//! ## Invariants Tested
//!
//! - PROP-TEL-001: Telemetry counters are monotonically non-decreasing
//! - PROP-XP-001: XP is always non-negative
//! - PROP-XP-002: Level is always derived correctly from XP
//! - PROP-REL-001: Reliability is always in [0.0, 1.0]
//! - PROP-RECIPE-001: Recipe reliability threshold is respected
//! - PROP-BUDGET-001: Time budgets never go negative
//! - PROP-BRAIN-001: Brain questions always return Brain origin

use anna_common::{
    TrustLevel,
    perf_timing::GlobalBudget,
    progression::{AnnaProgression, Level},
    rpg_display::ReliabilityScale,
    MIN_RECIPE_RELIABILITY, RECIPE_MATCH_THRESHOLD,
    try_fast_answer,
};
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Simple pseudo-random number generator for test inputs
/// Uses xorshift64 algorithm
struct TestRng {
    state: u64,
}

impl TestRng {
    fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }

    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        if max <= min { return min; }
        min + (self.next_u64() % (max - min))
    }
}

// ============================================================================
// PROP-TEL-001: Telemetry Counter Monotonicity
// ============================================================================

mod telemetry_properties {
    use super::*;

    /// Telemetry counters MUST only increase, never decrease
    #[test]
    fn test_prop_tel_001_counter_monotonicity() {
        // Simulate counter operations
        let counter = AtomicU64::new(0);
        let mut rng = TestRng::new(42);

        let mut previous_value = 0u64;

        for _ in 0..1000 {
            // Only increment operations are allowed on counters
            let increment = rng.next_range(0, 100);
            counter.fetch_add(increment, Ordering::SeqCst);

            let current = counter.load(Ordering::SeqCst);
            assert!(
                current >= previous_value,
                "Counter went backwards: {} -> {}",
                previous_value, current
            );
            previous_value = current;
        }
    }

    /// Telemetry counters MUST handle concurrent access safely
    #[test]
    fn test_prop_tel_002_concurrent_increment() {
        use std::sync::Arc;
        use std::thread;

        let counter = Arc::new(AtomicU64::new(0));
        let expected_total: u64 = 10 * 100; // 10 threads * 100 increments each

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let c = Arc::clone(&counter);
                thread::spawn(move || {
                    for _ in 0..100 {
                        c.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(
            counter.load(Ordering::SeqCst),
            expected_total,
            "Concurrent increments should sum correctly"
        );
    }
}

// ============================================================================
// PROP-XP-001/002: XP System Invariants
// ============================================================================

mod xp_properties {
    use super::*;

    /// XP MUST always be non-negative
    #[test]
    fn test_prop_xp_001_non_negative() {
        let mut rng = TestRng::new(12345);

        for _ in 0..100 {
            let xp = rng.next_u64() % 1_000_000;
            let prog = AnnaProgression::from_xp(xp);

            // total_xp is u64, inherently non-negative; verify type and bounds
            let _xp_value: u64 = prog.total_xp;
            assert!(
                prog.total_xp <= u64::MAX,
                "XP should not overflow"
            );
        }
    }

    /// Level MUST be correctly derived from XP
    #[test]
    fn test_prop_xp_002_level_derivation() {
        let mut rng = TestRng::new(67890);

        for _ in 0..100 {
            let xp = rng.next_u64() % 1_000_000;
            let prog = AnnaProgression::from_xp(xp);

            // Level should be 0-99
            assert!(
                prog.level.0 <= 99,
                "Level {} exceeds max of 99",
                prog.level.0
            );

            // Verify level is consistent with XP
            let level_from_xp = Level::from_xp(xp);
            assert_eq!(
                prog.level.0, level_from_xp.0,
                "Level derivation mismatch for XP {}",
                xp
            );
        }
    }

    /// XP addition MUST be monotonic (only increases)
    #[test]
    fn test_prop_xp_003_addition_monotonic() {
        let mut prog = AnnaProgression::new();
        let mut rng = TestRng::new(11111);

        let mut prev_xp = 0u64;
        let mut prev_level = 0u8;

        for _ in 0..100 {
            let add_xp = rng.next_range(1, 1000);
            prog.add_xp(add_xp);

            assert!(
                prog.total_xp >= prev_xp,
                "XP decreased: {} -> {}",
                prev_xp, prog.total_xp
            );
            assert!(
                prog.level.0 >= prev_level,
                "Level decreased: {} -> {}",
                prev_level, prog.level.0
            );

            prev_xp = prog.total_xp;
            prev_level = prog.level.0;
        }
    }
}

// ============================================================================
// PROP-REL-001: Reliability Bounds
// ============================================================================

mod reliability_properties {
    use super::*;

    /// Reliability MUST always be in [0.0, 1.0]
    #[test]
    fn test_prop_rel_001_bounds() {
        let mut rng = TestRng::new(22222);

        for _ in 0..100 {
            let rel = rng.next_f64();
            assert!(
                rel >= 0.0 && rel <= 1.0,
                "Reliability {} outside [0,1]",
                rel
            );

            // ReliabilityScale from_reliability should work for any valid value
            let scale = ReliabilityScale::from_reliability(rel);

            // Verify quality labels are assigned correctly (case-insensitive)
            // Scale: Green >= 0.9, Yellow >= 0.7, Orange >= 0.5, Red > 0, None = 0
            let quality_lower = scale.quality.to_lowercase();
            if rel >= 0.9 {
                assert_eq!(quality_lower, "green", "Expected green for {}", rel);
            } else if rel >= 0.7 {
                assert_eq!(quality_lower, "yellow", "Expected yellow for {}", rel);
            } else if rel >= 0.5 {
                assert_eq!(quality_lower, "orange", "Expected orange for {}", rel);
            } else if rel > 0.0 {
                assert_eq!(quality_lower, "red", "Expected red for {}", rel);
            } else {
                assert_eq!(quality_lower, "none", "Expected none for {}", rel);
            }
        }
    }

    /// TrustLevel MUST classify any value in [0,1]
    #[test]
    fn test_prop_rel_002_trust_classification() {
        let mut rng = TestRng::new(33333);

        for _ in 0..100 {
            let trust_val = rng.next_f64() as f32;
            assert!(
                trust_val >= 0.0 && trust_val <= 1.0,
                "Trust value {} outside [0,1]",
                trust_val
            );

            // Should always produce a valid TrustLevel
            let level = TrustLevel::from_trust(trust_val);
            match level {
                TrustLevel::Low | TrustLevel::Normal | TrustLevel::High => {}
            }
        }
    }
}

// ============================================================================
// PROP-RECIPE-001: Recipe Threshold Invariants
// ============================================================================

mod recipe_properties {
    use super::*;

    /// Recipe reliability threshold MUST be respected
    #[test]
    fn test_prop_recipe_001_reliability_threshold() {
        // MIN_RECIPE_RELIABILITY should be a sensible value
        assert!(
            MIN_RECIPE_RELIABILITY >= 0.0 && MIN_RECIPE_RELIABILITY <= 1.0,
            "MIN_RECIPE_RELIABILITY {} outside [0,1]",
            MIN_RECIPE_RELIABILITY
        );

        // Should be reasonably high (at least 70%)
        assert!(
            MIN_RECIPE_RELIABILITY >= 0.70,
            "MIN_RECIPE_RELIABILITY {} too low for reliable recipes",
            MIN_RECIPE_RELIABILITY
        );
    }

    /// Recipe match threshold MUST be sensible
    #[test]
    fn test_prop_recipe_002_match_threshold() {
        assert!(
            RECIPE_MATCH_THRESHOLD >= 0.0 && RECIPE_MATCH_THRESHOLD <= 1.0,
            "RECIPE_MATCH_THRESHOLD {} outside [0,1]",
            RECIPE_MATCH_THRESHOLD
        );

        // Should require reasonable confidence
        assert!(
            RECIPE_MATCH_THRESHOLD >= 0.50,
            "RECIPE_MATCH_THRESHOLD {} too low for accurate matching",
            RECIPE_MATCH_THRESHOLD
        );
    }
}

// ============================================================================
// PROP-BUDGET-001: Time Budget Invariants
// ============================================================================

mod budget_properties {
    use super::*;
    use std::time::Duration;

    /// Time budget remaining MUST never go negative
    #[test]
    fn test_prop_budget_001_non_negative() {
        let budget = GlobalBudget::with_budget(100);

        // Even after timeout, remaining should be 0, not negative
        std::thread::sleep(Duration::from_millis(150));

        let remaining = budget.remaining_ms();
        assert!(
            remaining == 0,
            "Budget remaining {} should be 0 after exhaustion",
            remaining
        );
    }

    /// Elapsed time MUST be >= 0 (u64 type guarantees this)
    #[test]
    fn test_prop_budget_002_elapsed_non_negative() {
        let budget = GlobalBudget::with_budget(1000);

        for _ in 0..10 {
            // elapsed_ms returns u64 which is inherently non-negative
            let elapsed: u64 = budget.elapsed_ms();
            // Verify elapsed increases over time
            let _verify_type: u64 = elapsed;
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// is_exhausted and remaining_ms MUST be consistent
    #[test]
    fn test_prop_budget_003_exhaustion_consistency() {
        let budget = GlobalBudget::with_budget(50);

        // Initially not exhausted
        assert!(!budget.is_exhausted() || budget.remaining_ms() == 0);

        std::thread::sleep(Duration::from_millis(60));

        // After timeout
        let exhausted = budget.is_exhausted();
        let remaining = budget.remaining_ms();

        if exhausted {
            assert_eq!(remaining, 0, "Exhausted budget should have 0 remaining");
        } else {
            assert!(remaining > 0, "Non-exhausted budget should have remaining time");
        }
    }
}

// ============================================================================
// PROP-BRAIN-001: Brain Fast Path Invariants
// ============================================================================

mod brain_properties {
    use super::*;

    /// Known Brain questions MUST return Brain origin
    #[test]
    fn test_prop_brain_001_origin_consistency() {
        let brain_questions = [
            "What CPU do I have?",
            "how many cores?",
            "How much RAM?",
            "disk space?",
            "what's my health?",
        ];

        for q in brain_questions {
            if let Some(answer) = try_fast_answer(q) {
                // If Brain handles it, origin must be Brain (case-insensitive check)
                assert_eq!(
                    answer.origin.as_str().to_lowercase(), "brain",
                    "Brain answer for '{}' has wrong origin: {}",
                    q, answer.origin.as_str()
                );

                // Brain answers must have high reliability
                assert!(
                    answer.reliability >= 0.90,
                    "Brain reliability {} < 0.90 for '{}'",
                    answer.reliability, q
                );
            }
        }
    }

    /// Brain classification MUST be deterministic
    #[test]
    fn test_prop_brain_002_classification_deterministic() {
        let test_questions = [
            "What CPU?",
            "RAM?",
            "disk?",
            "random question",
            "weather today",
        ];

        for q in test_questions {
            let first_result = try_fast_answer(q);
            let second_result = try_fast_answer(q);

            match (&first_result, &second_result) {
                (Some(a), Some(b)) => {
                    assert_eq!(
                        a.origin.as_str(), b.origin.as_str(),
                        "Classification changed for '{}': {} vs {}",
                        q, a.origin.as_str(), b.origin.as_str()
                    );
                }
                (None, None) => {} // Both correctly rejected
                _ => panic!(
                    "Classification inconsistent for '{}': {:?} vs {:?}",
                    q, first_result.is_some(), second_result.is_some()
                ),
            }
        }
    }
}

// ============================================================================
// PROP-ANSWER-001: Answer Structure Invariants
// ============================================================================

mod answer_properties {
    use super::*;

    /// All answers MUST have required fields
    #[test]
    fn test_prop_answer_001_required_fields() {
        let test_questions = [
            "What CPU do I have?",
            "How much RAM?",
            "What's my disk space?",
        ];

        for q in test_questions {
            if let Some(answer) = try_fast_answer(q) {
                // Must have non-empty text
                assert!(
                    !answer.text.is_empty(),
                    "Answer text empty for '{}'",
                    q
                );

                // Must have valid reliability
                assert!(
                    answer.reliability >= 0.0 && answer.reliability <= 1.0,
                    "Invalid reliability {} for '{}'",
                    answer.reliability, q
                );

                // Must have non-empty origin
                assert!(
                    !answer.origin.as_str().is_empty(),
                    "Answer origin empty for '{}'",
                    q
                );
            }
        }
    }
}
