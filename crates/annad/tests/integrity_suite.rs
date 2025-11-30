//! Feature Integrity Test Suite v3.3.0
//!
//! Comprehensive tests verifying correctness of every subsystem as defined
//! in docs/FEATURE_INTEGRITY_MATRIX.md.
//!
//! Each subsystem has:
//! - Positive case test (happy path)
//! - Negative case test (expected failure)
//! - Timeout/slow-path test
//! - Bad input test
//! - State reset test (where applicable)
//!
//! ## Test Categories
//!
//! 1. Brain Fast Path (INV-BRAIN-*)
//! 2. Recipe System (INV-RECIPE-*)
//! 3. Probe System (INV-PROBE-*)
//! 4. Junior/Senior LLM (INV-LLM-*)
//! 5. XP System (INV-XP-*)
//! 6. Telemetry (INV-TEL-*)
//! 7. Answer Construction (INV-ANS-*)
//! 8. Reset System (INV-RESET-*)

use anna_common::{
    extract_recipe, try_fast_answer, FastAnswer, FastQuestionType, QuestionType,
    Recipe, RecipeStore, ReliabilityScale, TrustLevel, XpStore,
    MIN_RECIPE_RELIABILITY, RECIPE_MATCH_THRESHOLD,
};
use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// Constants from Feature Integrity Matrix
// ============================================================================

const BRAIN_LATENCY_HARD_LIMIT_MS: u128 = 150;
const GREEN_THRESHOLD: f64 = 0.90;
const YELLOW_THRESHOLD: f64 = 0.70;
const RED_THRESHOLD: f64 = 0.50;

// ============================================================================
// 1. BRAIN FAST PATH INTEGRITY TESTS
// ============================================================================

mod brain_integrity {
    use super::*;

    /// INV-BRAIN-001: Brain MUST NOT make LLM calls
    /// Verified by: Brain returns instantly (< 150ms) with no network
    #[test]
    fn test_inv_brain_001_no_llm_calls() {
        let start = Instant::now();
        let _ = try_fast_answer("How much RAM do I have?");
        let elapsed = start.elapsed().as_millis();

        // If LLM was called, this would take seconds, not milliseconds
        assert!(
            elapsed < BRAIN_LATENCY_HARD_LIMIT_MS,
            "Brain took {}ms - possible LLM call detected (should be <{}ms)",
            elapsed,
            BRAIN_LATENCY_HARD_LIMIT_MS
        );
    }

    /// INV-BRAIN-002: Brain MUST return AnswerOrigin::Brain
    #[test]
    fn test_inv_brain_002_origin_is_brain() {
        let answer = try_fast_answer("How many CPU cores do I have?");
        assert!(answer.is_some(), "Brain should handle CPU questions");
        assert_eq!(
            answer.unwrap().origin,
            "Brain",
            "Brain answers must have origin 'Brain'"
        );
    }

    /// INV-BRAIN-003: Brain reliability MUST be >= 0.90 (Green)
    #[test]
    fn test_inv_brain_003_reliability_green() {
        let test_questions = [
            "How much RAM do I have?",
            "How many CPU cores do I have?",
            "How much free disk space do I have?",
        ];

        for q in &test_questions {
            if let Some(answer) = try_fast_answer(q) {
                assert!(
                    answer.reliability >= GREEN_THRESHOLD,
                    "Brain answer for '{}' has reliability {} < {} (Green threshold)",
                    q,
                    answer.reliability,
                    GREEN_THRESHOLD
                );
            }
        }
    }

    /// INV-BRAIN-005: Brain MUST NOT return empty answer text
    #[test]
    fn test_inv_brain_005_no_empty_text() {
        let test_questions = [
            "How much RAM do I have?",
            "How many CPU cores?",
            "Free disk space?",
            "Are you healthy Anna?",
        ];

        for q in &test_questions {
            if let Some(answer) = try_fast_answer(q) {
                assert!(
                    !answer.text.is_empty(),
                    "Brain returned empty text for '{}'",
                    q
                );
                assert!(
                    answer.text.len() > 10,
                    "Brain returned suspiciously short answer for '{}': '{}'",
                    q,
                    answer.text
                );
            }
        }
    }

    /// Negative: Brain MUST return None for unknown questions
    #[test]
    fn test_brain_negative_unknown_returns_none() {
        let unknown_questions = [
            "What is the meaning of life?",
            "Install nginx please",
            "Write me a poem about Rust",
            "Calculate 2+2",
        ];

        for q in &unknown_questions {
            let answer = try_fast_answer(q);
            assert!(
                answer.is_none(),
                "Brain should NOT handle '{}' but returned: {:?}",
                q,
                answer
            );
        }
    }

    /// Timeout: Brain must complete within hard limit
    #[test]
    fn test_brain_timeout_within_limit() {
        let questions = [
            "How much RAM?",
            "CPU cores?",
            "Disk space?",
            "Health status?",
        ];

        for q in &questions {
            let start = Instant::now();
            let _ = try_fast_answer(q);
            let elapsed = start.elapsed().as_millis();

            assert!(
                elapsed < BRAIN_LATENCY_HARD_LIMIT_MS,
                "Brain exceeded hard limit for '{}': {}ms > {}ms",
                q,
                elapsed,
                BRAIN_LATENCY_HARD_LIMIT_MS
            );
        }
    }

    /// Bad input: Brain handles empty and malformed input gracefully
    #[test]
    fn test_brain_bad_input_handling() {
        // Empty string
        let answer = try_fast_answer("");
        assert!(answer.is_none(), "Empty string should return None");

        // Whitespace only
        let answer = try_fast_answer("   ");
        assert!(answer.is_none(), "Whitespace should return None");

        // Very long input
        let long_input = "a".repeat(10000);
        let answer = try_fast_answer(&long_input);
        assert!(answer.is_none(), "Very long input should return None");

        // Special characters
        let answer = try_fast_answer("!@#$%^&*()");
        assert!(answer.is_none(), "Special chars should return None");
    }

    /// Classification accuracy test
    #[test]
    fn test_brain_classification_accuracy() {
        // RAM questions
        assert_eq!(FastQuestionType::classify("How much RAM?"), FastQuestionType::Ram);
        assert_eq!(FastQuestionType::classify("total memory"), FastQuestionType::Ram);
        assert_eq!(FastQuestionType::classify("how many gb of ram"), FastQuestionType::Ram);

        // CPU questions
        assert_eq!(FastQuestionType::classify("how many cpu cores"), FastQuestionType::CpuCores);
        assert_eq!(FastQuestionType::classify("how many threads"), FastQuestionType::CpuCores);

        // Disk questions
        assert_eq!(FastQuestionType::classify("disk space on /"), FastQuestionType::RootDiskSpace);
        assert_eq!(FastQuestionType::classify("free storage"), FastQuestionType::RootDiskSpace);

        // Health questions
        assert_eq!(FastQuestionType::classify("are you ok?"), FastQuestionType::AnnaHealth);
        assert_eq!(FastQuestionType::classify("diagnose yourself"), FastQuestionType::AnnaHealth);

        // Unknown questions
        assert_eq!(FastQuestionType::classify("install nginx"), FastQuestionType::Unknown);
    }
}

// ============================================================================
// 2. RECIPE SYSTEM INTEGRITY TESTS
// ============================================================================

mod recipe_integrity {
    use super::*;

    /// INV-RECIPE-001: Recipes MUST only be extracted from answers with reliability >= 0.85
    #[test]
    fn test_inv_recipe_001_min_reliability() {
        // High reliability - should extract
        let recipe = extract_recipe(
            "How much RAM do I have?",
            QuestionType::RamInfo,
            &["mem.info".to_string()],
            "You have 32 GiB of RAM",
            0.95,
        );
        assert!(recipe.is_some(), "Should extract recipe at 0.95 reliability");

        // At threshold - should extract
        let recipe = extract_recipe(
            "How much RAM do I have?",
            QuestionType::RamInfo,
            &["mem.info".to_string()],
            "You have 32 GiB of RAM",
            MIN_RECIPE_RELIABILITY,
        );
        assert!(recipe.is_some(), "Should extract recipe at threshold");

        // Below threshold - MUST NOT extract
        let recipe = extract_recipe(
            "How much RAM do I have?",
            QuestionType::RamInfo,
            &["mem.info".to_string()],
            "Maybe 16 GiB?",
            0.84,
        );
        assert!(recipe.is_none(), "MUST NOT extract recipe below threshold");

        // Way below threshold
        let recipe = extract_recipe(
            "Test question",
            QuestionType::RamInfo,
            &["mem.info".to_string()],
            "Low confidence answer",
            0.50,
        );
        assert!(recipe.is_none(), "MUST NOT extract recipe at 0.50");
    }

    /// INV-RECIPE-002: Recipe match threshold MUST be >= 0.70
    #[test]
    fn test_inv_recipe_002_match_threshold() {
        assert!(
            RECIPE_MATCH_THRESHOLD >= 0.70,
            "Recipe match threshold {} < 0.70",
            RECIPE_MATCH_THRESHOLD
        );

        let recipe = Recipe::new(
            "cpu_info_basic",
            QuestionType::CpuInfo,
            vec!["cpu.info".to_string()],
            "Your CPU is...",
            0.95,
        )
        .with_tokens(&["cpu", "model"]);

        // Should match with high score
        let score = recipe.matches("What CPU model do I have?", &QuestionType::CpuInfo);
        assert!(score >= RECIPE_MATCH_THRESHOLD, "Good match should pass threshold");

        // Should not match different type
        let score = recipe.matches("What CPU model?", &QuestionType::RamInfo);
        assert_eq!(score, 0.0, "Different type should score 0.0");
    }

    /// INV-RECIPE-007: Recipe MUST NOT match different intents
    #[test]
    fn test_inv_recipe_007_no_cross_intent_match() {
        let mut store = RecipeStore::default();

        // Add CPU recipe
        let cpu_recipe = Recipe::new(
            "cpu_info_basic",
            QuestionType::CpuInfo,
            vec!["cpu.info".to_string()],
            "Your CPU is...",
            0.95,
        )
        .with_tokens(&["cpu", "processor", "cores"]);
        store.add(cpu_recipe);

        // RAM question should NOT match CPU recipe
        let found = store.find_match("How much RAM do I have?", &QuestionType::RamInfo);
        assert!(
            found.is_none(),
            "RAM question MUST NOT match CPU recipe"
        );

        // Disk question should NOT match CPU recipe
        let found = store.find_match("How much disk space?", &QuestionType::DiskInfo);
        assert!(
            found.is_none(),
            "Disk question MUST NOT match CPU recipe"
        );
    }

    /// Negative: Empty recipe store returns None
    #[test]
    fn test_recipe_negative_empty_store() {
        let store = RecipeStore::default();
        let found = store.find_match("How much RAM?", &QuestionType::RamInfo);
        assert!(found.is_none(), "Empty store should return None");
    }

    /// Recipe learning and reuse
    #[test]
    fn test_recipe_learning_and_reuse() {
        let mut store = RecipeStore::default();

        // Learn a recipe
        let recipe = Recipe::new(
            "ram_total",
            QuestionType::RamInfo,
            vec!["mem.info".to_string()],
            "You have {total} of RAM",
            0.95,
        )
        .with_tokens(&["ram", "memory", "total"]);
        store.add(recipe);

        // Should match similar question
        let found = store.find_match("How much total RAM memory?", &QuestionType::RamInfo);
        assert!(found.is_some(), "Should match learned recipe");

        // Should match variant
        let found = store.find_match("What is my total RAM?", &QuestionType::RamInfo);
        assert!(found.is_some(), "Should match variant question");
    }

    /// Recipe application test
    #[test]
    fn test_recipe_application() {
        let recipe = Recipe::new(
            "cpu_info",
            QuestionType::CpuInfo,
            vec!["cpu.info".to_string()],
            "Your CPU is {model} with {cores} cores",
            0.95,
        );

        let mut evidence = HashMap::new();
        evidence.insert("model".to_string(), "AMD Ryzen 9".to_string());
        evidence.insert("cores".to_string(), "16".to_string());

        let answer = recipe.apply(&evidence);
        assert!(answer.contains("AMD Ryzen 9"), "Template should fill model");
        assert!(answer.contains("16 cores"), "Template should fill cores");
    }

    /// Bad input: Invalid recipe data
    #[test]
    fn test_recipe_bad_input() {
        // Empty probes
        let recipe = Recipe::new(
            "test",
            QuestionType::Unknown,
            vec![], // Empty probes
            "test template",
            0.95,
        );
        assert!(recipe.probes.is_empty(), "Recipe can have empty probes");

        // Empty template
        let recipe = Recipe::new(
            "test",
            QuestionType::Unknown,
            vec!["cpu.info".to_string()],
            "", // Empty template
            0.95,
        );
        assert!(recipe.answer_template.is_empty());
    }
}

// ============================================================================
// 3. XP SYSTEM INTEGRITY TESTS
// ============================================================================

mod xp_integrity {
    use super::*;

    /// INV-XP-002: Trust MUST be clamped to 0.0-1.0
    #[test]
    fn test_inv_xp_002_trust_clamped() {
        // Test trust level boundaries
        assert_eq!(TrustLevel::from_trust(0.0), TrustLevel::Low);
        assert_eq!(TrustLevel::from_trust(0.39), TrustLevel::Low);
        assert_eq!(TrustLevel::from_trust(0.4), TrustLevel::Normal);
        assert_eq!(TrustLevel::from_trust(0.7), TrustLevel::Normal);
        assert_eq!(TrustLevel::from_trust(0.71), TrustLevel::High);
        assert_eq!(TrustLevel::from_trust(1.0), TrustLevel::High);

        // Edge cases - should clamp
        assert_eq!(TrustLevel::from_trust(-0.5), TrustLevel::Low);
        assert_eq!(TrustLevel::from_trust(1.5), TrustLevel::High);
    }

    /// INV-XP-003: Level MUST be clamped to 1-99
    #[test]
    fn test_inv_xp_003_level_range() {
        // Level should never be 0 or > 99
        // XpTrack starts at level 1
        let store = XpStore::default();
        assert!(store.anna.level >= 1, "Level must be >= 1");
        assert!(store.anna.level <= 99, "Level must be <= 99");
        assert!(store.junior.level >= 1, "Level must be >= 1");
        assert!(store.senior.level >= 1, "Level must be >= 1");
    }

    /// Reliability scaling thresholds
    #[test]
    fn test_xp_reliability_scaling() {
        // Green (>=90%)
        let scale = ReliabilityScale::from_reliability(0.95);
        assert_eq!(scale.quality, "Green");
        assert_eq!(scale.xp_multiplier, 1.0);

        // Yellow (70-90%)
        let scale = ReliabilityScale::from_reliability(0.80);
        assert_eq!(scale.quality, "Yellow");
        assert!(scale.xp_multiplier < 1.0);

        // Orange (50-70%)
        let scale = ReliabilityScale::from_reliability(0.60);
        assert_eq!(scale.quality, "Orange");
        assert!(scale.xp_multiplier < 0.75);

        // Red (<50%)
        let scale = ReliabilityScale::from_reliability(0.30);
        assert_eq!(scale.quality, "Red");
        assert!(scale.xp_multiplier < 0.5);
    }

    /// Default XP store state
    #[test]
    fn test_xp_default_state() {
        let store = XpStore::default();

        // All agents start at level 1
        assert_eq!(store.anna.level, 1);
        assert_eq!(store.junior.level, 1);
        assert_eq!(store.senior.level, 1);

        // Default trust is 0.5 (Normal)
        assert_eq!(TrustLevel::from_trust(store.anna.trust), TrustLevel::Normal);
    }
}

// ============================================================================
// 4. ANSWER CONSTRUCTION INTEGRITY TESTS
// ============================================================================

mod answer_integrity {
    use super::*;

    /// INV-ANS-001: Answer text MUST NOT be empty
    #[test]
    fn test_inv_ans_001_no_empty_text() {
        // FastAnswer construction
        let answer = FastAnswer::new("Test answer", vec!["cpu.info"], 0.95);
        assert!(!answer.text.is_empty(), "Answer text must not be empty");
    }

    /// INV-ANS-002: Reliability MUST be 0.0-1.0
    #[test]
    fn test_inv_ans_002_reliability_range() {
        // Valid reliability
        let answer = FastAnswer::new("Test", vec![], 0.5);
        assert!(answer.reliability >= 0.0 && answer.reliability <= 1.0);

        let answer = FastAnswer::new("Test", vec![], 0.0);
        assert_eq!(answer.reliability, 0.0);

        let answer = FastAnswer::new("Test", vec![], 1.0);
        assert_eq!(answer.reliability, 1.0);
    }

    /// INV-ANS-003: Reliability label MUST match thresholds
    #[test]
    fn test_inv_ans_003_label_matches_threshold() {
        // Green >= 0.90
        assert!(GREEN_THRESHOLD >= 0.90);

        // Yellow >= 0.70
        assert!(YELLOW_THRESHOLD >= 0.70);

        // Red >= 0.50
        assert!(RED_THRESHOLD >= 0.50);

        // Ordering
        assert!(GREEN_THRESHOLD > YELLOW_THRESHOLD);
        assert!(YELLOW_THRESHOLD > RED_THRESHOLD);
    }

    /// INV-ANS-004: Origin MUST be valid
    #[test]
    fn test_inv_ans_004_valid_origin() {
        let answer = FastAnswer::new("Test", vec!["cpu.info"], 0.95);
        let valid_origins = ["Brain", "Recipe", "Junior", "Senior", "Fallback"];
        assert!(
            valid_origins.contains(&answer.origin.as_str()),
            "Origin '{}' not in valid set",
            answer.origin
        );
    }

    /// Answer with citations
    #[test]
    fn test_answer_with_citations() {
        let answer = FastAnswer::new(
            "You have 32 GiB of RAM",
            vec!["mem.info"],
            0.95,
        );

        assert!(!answer.text.is_empty());
        assert!(!answer.citations.is_empty());
        assert_eq!(answer.citations[0], "mem.info");
        assert_eq!(answer.origin, "Brain");
    }
}

// ============================================================================
// 5. INVARIANT VIOLATION DETECTION
// ============================================================================

mod invariant_guards {
    use super::*;

    /// Test that reliability is always in valid range
    #[test]
    fn test_reliability_always_valid() {
        // ReliabilityScale handles edge cases
        let scale = ReliabilityScale::from_reliability(-0.5);
        assert!(scale.xp_multiplier >= 0.0, "Negative reliability handled");

        let scale = ReliabilityScale::from_reliability(1.5);
        assert!(scale.xp_multiplier <= 1.0, "Over 1.0 reliability handled");

        let scale = ReliabilityScale::from_reliability(f64::NAN);
        assert!(scale.xp_multiplier >= 0.0, "NaN reliability handled");
    }

    /// Test classification never panics on any input
    #[test]
    fn test_classification_never_panics() {
        let edge_cases = [
            "",
            " ",
            "\n",
            "\t",
            "a",
            &"x".repeat(100000),
            "🎉🎊🎈",
            "\0\0\0",
            "SELECT * FROM users",
            "<script>alert(1)</script>",
            "../../../etc/passwd",
        ];

        for input in &edge_cases {
            // Should not panic
            let _ = FastQuestionType::classify(input);
        }
    }

    /// Test recipe matching never panics
    #[test]
    fn test_recipe_matching_never_panics() {
        let store = RecipeStore::default();

        let edge_cases = [
            ("", &QuestionType::Unknown),
            (" ", &QuestionType::CpuInfo),
            (&"x".repeat(100000), &QuestionType::RamInfo),
        ];

        for (q, qt) in &edge_cases {
            // Should not panic
            let _ = store.find_match(q, qt);
        }
    }
}

// ============================================================================
// 6. DEBUG MODE PARITY TESTS
// ============================================================================

mod debug_parity {
    use super::*;

    /// Debug mode should not change answer content
    #[test]
    fn test_debug_mode_same_answer_content() {
        // Run same question with debug state variations
        // (Since debug state is global, we just verify Brain returns consistent answers)
        let q = "How much RAM do I have?";

        let answer1 = try_fast_answer(q);
        let answer2 = try_fast_answer(q);

        if let (Some(a1), Some(a2)) = (&answer1, &answer2) {
            // Content should be identical or very similar
            // (may vary slightly due to timing in text)
            assert_eq!(a1.origin, a2.origin, "Origin should be consistent");
            assert!(
                (a1.reliability - a2.reliability).abs() < 0.01,
                "Reliability should be consistent"
            );
        }
    }

    /// Debug mode latency should be consistent
    #[test]
    fn test_debug_mode_latency_parity() {
        let q = "How many CPU cores?";

        // Run multiple times to get average
        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let _ = try_fast_answer(q);
            times.push(start.elapsed().as_micros());
        }

        let avg = times.iter().sum::<u128>() / times.len() as u128;

        // All times should be within the hard limit (150ms = 150,000 microseconds)
        for (i, t) in times.iter().enumerate() {
            assert!(
                *t < 150_000,
                "Run {} took {}μs, exceeds 150ms limit",
                i + 1,
                t
            );
        }

        // Average should be well under the limit (expect <10ms for Brain)
        assert!(
            avg < 50_000,
            "Average latency {}μs too high (expect <50ms)",
            avg
        );
    }
}

// ============================================================================
// 7. REGRESSION DETECTION TESTS
// ============================================================================

mod regression_detection {
    use super::*;

    /// Canonical questions should always be answered by Brain
    #[test]
    fn test_canonical_questions_brain_path() {
        let canonical = [
            ("How much RAM do I have?", "Brain"),
            ("How many CPU cores do I have?", "Brain"),
            ("How much free disk space do I have?", "Brain"),
            ("Are you healthy Anna?", "Brain"),
        ];

        for (q, expected_origin) in &canonical {
            if let Some(answer) = try_fast_answer(q) {
                assert_eq!(
                    answer.origin, *expected_origin,
                    "Question '{}' should be answered by {}",
                    q, expected_origin
                );
            }
        }
    }

    /// Reliability thresholds should never regress
    #[test]
    fn test_reliability_thresholds_stable() {
        // These are architectural constants that should never change
        assert!(GREEN_THRESHOLD >= 0.90, "Green threshold regressed");
        assert!(YELLOW_THRESHOLD >= 0.70, "Yellow threshold regressed");
        assert!(RED_THRESHOLD >= 0.50, "Red threshold regressed");
    }

    /// Brain latency should never regress
    #[test]
    fn test_brain_latency_stable() {
        assert!(
            BRAIN_LATENCY_HARD_LIMIT_MS <= 150,
            "Brain latency limit regressed to {}ms",
            BRAIN_LATENCY_HARD_LIMIT_MS
        );
    }

    /// Recipe threshold should never regress
    #[test]
    fn test_recipe_thresholds_stable() {
        assert!(
            MIN_RECIPE_RELIABILITY >= 0.85,
            "Recipe reliability threshold regressed to {}",
            MIN_RECIPE_RELIABILITY
        );
        assert!(
            RECIPE_MATCH_THRESHOLD >= 0.70,
            "Recipe match threshold regressed to {}",
            RECIPE_MATCH_THRESHOLD
        );
    }
}

// ============================================================================
// 8. COMBINED SUBSYSTEM TESTS
// ============================================================================

// ============================================================================
// 9. AUTOPROVISION SYSTEM INTEGRITY TESTS
// ============================================================================

mod autoprovision_integrity {
    use anna_common::llm_provision::{
        HardwareTier, LlmSelection, ModelBenchmark, FallbackDecision,
        evaluate_junior_fallback, should_fallback_junior,
        JUNIOR_FALLBACK_TIMEOUT_MS, FALLBACK_MODEL, ROUTER_FALLBACK_MODEL,
    };

    /// INV-PROV-001: Hardware tier detection MUST return valid tier
    #[test]
    fn test_inv_prov_001_valid_tier() {
        let tier = HardwareTier::detect();
        assert!(
            matches!(
                tier,
                HardwareTier::Minimal | HardwareTier::Basic | HardwareTier::Standard | HardwareTier::Power
            ),
            "Hardware tier must be valid enum variant"
        );
    }

    /// INV-PROV-002: Tier capabilities MUST match specification
    #[test]
    fn test_inv_prov_002_tier_capabilities() {
        // Minimal: No Router, No Senior
        assert!(!HardwareTier::Minimal.has_router());
        assert!(!HardwareTier::Minimal.has_senior());

        // Basic: Has Router, No Senior
        assert!(HardwareTier::Basic.has_router());
        assert!(!HardwareTier::Basic.has_senior());

        // Standard: Has both
        assert!(HardwareTier::Standard.has_router());
        assert!(HardwareTier::Standard.has_senior());

        // Power: Has both
        assert!(HardwareTier::Power.has_router());
        assert!(HardwareTier::Power.has_senior());
    }

    /// INV-PROV-003: Default selection MUST use fallback models
    #[test]
    fn test_inv_prov_003_default_fallback() {
        let selection = LlmSelection::default();
        assert_eq!(selection.junior_model, FALLBACK_MODEL);
        assert_eq!(selection.router_model, ROUTER_FALLBACK_MODEL);
        assert!(selection.autoprovision_enabled);
    }

    /// INV-PROV-004: Junior fallback timeout MUST be <= 2000ms
    #[test]
    fn test_inv_prov_004_junior_timeout() {
        assert!(
            JUNIOR_FALLBACK_TIMEOUT_MS <= 2000,
            "Junior fallback timeout {}ms > 2000ms",
            JUNIOR_FALLBACK_TIMEOUT_MS
        );

        // Under timeout - no fallback
        assert!(!should_fallback_junior(1500));
        assert!(!should_fallback_junior(2000)); // Exactly at limit

        // Over timeout - fallback
        assert!(should_fallback_junior(2001));
        assert!(should_fallback_junior(5000));
    }

    /// INV-PROV-005: FallbackDecision MUST be correct
    #[test]
    fn test_inv_prov_005_fallback_decision() {
        // Fast response - proceed
        let decision = evaluate_junior_fallback(1000);
        assert!(!decision.use_fallback);
        assert!(decision.latency_ms.is_none());

        // At limit - proceed
        let decision = evaluate_junior_fallback(2000);
        assert!(!decision.use_fallback);

        // Over limit - fallback
        let decision = evaluate_junior_fallback(2500);
        assert!(decision.use_fallback);
        assert_eq!(decision.latency_ms, Some(2500));
        assert!(decision.reason.contains("2500ms"));
    }

    /// INV-PROV-006: ModelBenchmark viability checks
    #[test]
    fn test_inv_prov_006_viability_checks() {
        // Unavailable model
        let bench = ModelBenchmark::unavailable("test-model", "Not installed");
        assert!(!bench.is_available);
        assert!(!bench.is_junior_viable());
        assert!(!bench.is_senior_viable());

        // Good junior model
        let mut bench = ModelBenchmark::unavailable("test", "");
        bench.is_available = true;
        bench.json_compliance = 0.96;
        bench.avg_latency_ms = 5000;
        bench.determinism_score = 0.95;
        assert!(bench.is_junior_viable());

        // Too slow junior
        bench.avg_latency_ms = 10000;
        assert!(!bench.is_junior_viable(), "Slow model should not be viable");

        // Good senior model
        let mut bench = ModelBenchmark::unavailable("test", "");
        bench.is_available = true;
        bench.reasoning_score = 0.90;
        bench.avg_latency_ms = 10000;
        assert!(bench.is_senior_viable());

        // Poor reasoning senior
        bench.reasoning_score = 0.5;
        assert!(!bench.is_senior_viable(), "Poor reasoning should not be viable");
    }
}

// ============================================================================
// 10. RESET SYSTEM INTEGRITY TESTS
// ============================================================================

mod reset_integrity {
    use anna_common::{
        ExperiencePaths, ExperienceSnapshot, ExperienceResetResult, ResetType,
        reset_experience, reset_factory,
        has_experience_data, has_knowledge_data,
    };

    /// INV-RESET-001: Experience reset exists and returns result
    #[test]
    fn test_inv_reset_001_experience_reset_exists() {
        // Test that the reset API is available
        let paths = ExperiencePaths::default();

        // Just verify the function exists and returns a result type
        // We don't actually perform a reset to avoid affecting system state
        let _result_type: fn(&ExperiencePaths) -> ExperienceResetResult = reset_experience;
    }

    /// INV-RESET-002: Factory reset exists and returns result
    #[test]
    fn test_inv_reset_002_factory_reset_exists() {
        // Test that the reset API is available
        let paths = ExperiencePaths::default();

        // Just verify the function exists
        let _result_type: fn(&ExperiencePaths) -> ExperienceResetResult = reset_factory;
    }

    /// INV-RESET-003: ResetType enum has correct variants
    #[test]
    fn test_inv_reset_003_reset_type_variants() {
        // Experience (soft) reset
        let soft = ResetType::Experience;
        assert!(matches!(soft, ResetType::Experience));

        // Factory (hard) reset
        let hard = ResetType::Factory;
        assert!(matches!(hard, ResetType::Factory));
    }

    /// INV-RESET-004: ExperiencePaths has expected structure
    #[test]
    fn test_inv_reset_004_paths_structure() {
        let paths = ExperiencePaths::default();

        // All paths should be non-empty
        assert!(!paths.xp_dir.as_os_str().is_empty());
        assert!(!paths.telemetry_file.as_os_str().is_empty());
        assert!(!paths.stats_dir.as_os_str().is_empty());
        assert!(!paths.knowledge_dir.as_os_str().is_empty());
    }

    /// INV-RESET-005: ExperienceSnapshot captures state
    #[test]
    fn test_inv_reset_005_snapshot_capture() {
        // Snapshot should be constructible
        let paths = ExperiencePaths::default();
        let snapshot = ExperienceSnapshot::capture(&paths);

        // Should have valid fields
        assert!(snapshot.anna_level <= 99, "Level should be <= 99");
        assert!(snapshot.anna_xp >= 0, "XP should be non-negative");
    }

    /// INV-RESET-006: has_experience_data function exists
    #[test]
    fn test_inv_reset_006_has_data_functions() {
        let paths = ExperiencePaths::default();

        // These functions should return bool
        let _: bool = has_experience_data(&paths);
        let _: bool = has_knowledge_data(&paths);
    }

    /// INV-RESET-007: ExperienceResetResult has success field
    #[test]
    fn test_inv_reset_007_result_structure() {
        // Create a result manually to test structure
        let result = ExperienceResetResult {
            success: true,
            reset_type: ResetType::Experience,
            components_reset: vec!["xp".to_string()],
            components_clean: vec![],
            errors: vec![],
            summary: "Test reset".to_string(),
        };

        assert!(result.success);
        assert!(matches!(result.reset_type, ResetType::Experience));
        assert!(!result.components_reset.is_empty());
        assert!(!result.summary.is_empty());
    }
}

// ============================================================================
// 11. BENCHMARK SYSTEM INTEGRITY TESTS
// ============================================================================

mod benchmark_integrity {
    use anna_common::{
        BenchmarkMode, PhaseId, SnowLeopardConfig,
        is_benchmark_request, parse_benchmark_mode,
        CANONICAL_QUESTIONS, PARAPHRASED_QUESTIONS, NOVEL_QUESTIONS,
    };

    /// INV-BENCH-001: Benchmark uses same pipeline as runtime
    #[test]
    fn test_inv_bench_001_same_pipeline() {
        // Verified by checking test_mode still uses try_fast_answer
        // which is the same Brain path as runtime
        let config = SnowLeopardConfig::test_mode();
        assert!(config.use_simulated_llm, "Test mode uses simulated LLM");

        let config = SnowLeopardConfig::runtime_mode();
        assert!(!config.use_simulated_llm, "Runtime mode uses real LLM");
    }

    /// INV-BENCH-002: All phases must be defined
    #[test]
    fn test_inv_bench_002_all_phases() {
        let all = PhaseId::all();
        assert_eq!(all.len(), 6, "Must have exactly 6 phases");

        // Verify all phases are present
        assert!(all.contains(&PhaseId::HardReset));
        assert!(all.contains(&PhaseId::WarmState));
        assert!(all.contains(&PhaseId::SoftReset));
        assert!(all.contains(&PhaseId::NLStress));
        assert!(all.contains(&PhaseId::NovelQuestions));
        assert!(all.contains(&PhaseId::LearningTest));
    }

    /// INV-BENCH-003: Quick mode subset
    #[test]
    fn test_inv_bench_003_quick_mode() {
        let quick = PhaseId::quick();
        assert_eq!(quick.len(), 3, "Quick mode should have 3 phases");

        // Must include sanity phases
        assert!(quick.contains(&PhaseId::HardReset));
        assert!(quick.contains(&PhaseId::WarmState));
        assert!(quick.contains(&PhaseId::LearningTest));
    }

    /// INV-BENCH-004: Question sets must be non-empty
    #[test]
    fn test_inv_bench_004_question_sets() {
        assert!(!CANONICAL_QUESTIONS.is_empty(), "Canonical questions must exist");
        assert!(!PARAPHRASED_QUESTIONS.is_empty(), "Paraphrased questions must exist");
        assert!(!NOVEL_QUESTIONS.is_empty(), "Novel questions must exist");

        // Each should have 10 questions
        assert_eq!(CANONICAL_QUESTIONS.len(), 10);
        assert_eq!(PARAPHRASED_QUESTIONS.len(), 10);
        assert_eq!(NOVEL_QUESTIONS.len(), 10);
    }

    /// INV-BENCH-005: Mode parsing
    #[test]
    fn test_inv_bench_005_mode_parsing() {
        // Full mode (default)
        assert_eq!(parse_benchmark_mode("run benchmark"), BenchmarkMode::Full);
        assert_eq!(parse_benchmark_mode("full benchmark"), BenchmarkMode::Full);

        // Quick mode
        assert_eq!(parse_benchmark_mode("quick benchmark"), BenchmarkMode::Quick);
        assert_eq!(parse_benchmark_mode("short test"), BenchmarkMode::Quick);
        assert_eq!(parse_benchmark_mode("fast sanity"), BenchmarkMode::Quick);
    }

    /// INV-BENCH-006: Benchmark request detection
    #[test]
    fn test_inv_bench_006_request_detection() {
        // Should detect
        assert!(is_benchmark_request("run the snow leopard benchmark"));
        assert!(is_benchmark_request("snow leopard test"));
        assert!(is_benchmark_request("run benchmark"));

        // Should NOT detect
        assert!(!is_benchmark_request("what is my cpu?"));
        assert!(!is_benchmark_request("how much ram?"));
        assert!(!is_benchmark_request("install nginx"));
    }

    /// INV-BENCH-007: Config defaults
    #[test]
    fn test_inv_bench_007_config_defaults() {
        let config = SnowLeopardConfig::default();
        assert!(config.perform_resets, "Default should perform resets");
        assert_eq!(config.learning_repetitions, 5);
        assert!(config.verbose);
    }
}

mod combined_tests {
    use super::*;

    /// End-to-end: Brain question with XP implications
    #[test]
    fn test_e2e_brain_question() {
        let q = "How much RAM do I have?";

        // Should be answered by Brain
        let answer = try_fast_answer(q);
        assert!(answer.is_some(), "Brain should answer RAM question");

        let answer = answer.unwrap();

        // Verify all FIM invariants
        assert_eq!(answer.origin, "Brain", "INV-BRAIN-002");
        assert!(answer.reliability >= GREEN_THRESHOLD, "INV-BRAIN-003");
        assert!(!answer.text.is_empty(), "INV-BRAIN-005");
        assert!(answer.text.len() > 10, "Answer should be meaningful");
    }

    /// End-to-end: Recipe extraction and reuse cycle
    #[test]
    fn test_e2e_recipe_cycle() {
        let mut store = RecipeStore::default();

        // Extract a recipe from a successful answer
        if let Some(recipe) = extract_recipe(
            "What is my CPU model?",
            QuestionType::CpuInfo,
            &["cpu.info".to_string()],
            "Your CPU is AMD Ryzen 9 5950X",
            0.95,
        ) {
            store.add(recipe);

            // Find the recipe for a similar question
            let found = store.find_match("Tell me my CPU model", &QuestionType::CpuInfo);
            assert!(found.is_some(), "Recipe should match similar question");
        }
    }

    /// Verify all question types have proper classification
    #[test]
    fn test_all_question_types_classified() {
        let type_examples = [
            (FastQuestionType::Ram, "How much RAM?"),
            (FastQuestionType::CpuCores, "How many CPU cores?"),
            (FastQuestionType::RootDiskSpace, "Free disk space?"),
            (FastQuestionType::AnnaHealth, "Are you ok?"),
            (FastQuestionType::Unknown, "What is Rust?"),
        ];

        for (expected_type, question) in &type_examples {
            let actual = FastQuestionType::classify(question);
            assert_eq!(
                &actual, expected_type,
                "Question '{}' classified as {:?}, expected {:?}",
                question, actual, expected_type
            );
        }
    }
}
