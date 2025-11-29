//! Snow Leopard Benchmark Suite v1.4.0 (Test Harness)
//!
//! This test harness uses the shared benchmark module from anna_common
//! with simulated LLM responses for deterministic testing.
//!
//! ## Running
//!
//! ```bash
//! # Full benchmark test (simulated)
//! cargo test --test benchmark_snow_leopard -- --nocapture --ignored
//!
//! # Quick tests (non-ignored)
//! cargo test --test benchmark_snow_leopard
//! ```

use anna_common::{
    // Benchmark module
    bench_snow_leopard::{
        SnowLeopardConfig, SnowLeopardResult, BenchmarkMode, PhaseId,
        run_benchmark, is_benchmark_request, parse_benchmark_mode,
        CANONICAL_QUESTIONS, PARAPHRASED_QUESTIONS, NOVEL_QUESTIONS,
        LastBenchmarkSummary,
    },
    // Reset infrastructure
    ExperiencePaths, reset_factory,
    // Brain fast path
    FastQuestionType, try_fast_answer,
};

// ============================================================================
// MAIN BENCHMARK TESTS (IGNORED BY DEFAULT)
// ============================================================================

/// Full Snow Leopard Benchmark Suite (simulated mode)
///
/// Runs all 6 phases with simulated LLM responses for deterministic testing.
#[tokio::test]
#[ignore] // Run with: cargo test --test benchmark_snow_leopard snow_leopard_benchmark_full -- --nocapture --ignored
async fn snow_leopard_benchmark_full() {
    println!("\n{}", "═".repeat(70));
    println!("  SNOW LEOPARD BENCHMARK TEST (Simulated Mode)");
    println!("{}", "═".repeat(70));

    let config = SnowLeopardConfig::test_mode();
    let result = run_benchmark(&config).await;

    // Print the ASCII summary
    println!("{}", result.ascii_summary);

    // Assertions
    assert!(
        result.total_questions >= 60,
        "Should answer at least 60 questions, got {}",
        result.total_questions
    );
    assert!(
        result.ux_consistency_passed,
        "UX consistency check should pass"
    );
    assert!(
        result.overall_success_rate() >= 80.0,
        "Success rate should be >= 80%, got {:.1}%",
        result.overall_success_rate()
    );
    assert_eq!(
        result.phases.len(), 6,
        "Should have 6 phases"
    );

    println!("\n✓ All assertions passed!");
}

/// Quick benchmark test (simulated mode)
///
/// Runs only phases 1, 2, and 6 for faster testing.
#[tokio::test]
#[ignore]
async fn snow_leopard_benchmark_quick() {
    println!("\n[QUICK TEST] Running phases 1, 2, 6 only...\n");

    let config = SnowLeopardConfig::test_mode()
        .with_mode(BenchmarkMode::Quick);

    let result = run_benchmark(&config).await;

    println!("{}", result.ascii_summary);

    assert_eq!(result.phases.len(), 3, "Quick mode should have 3 phases");
    assert!(result.total_questions >= 25, "Should answer at least 25 questions");

    println!("\n✓ Quick test passed!");
}

/// Smoke test - single phase only
#[tokio::test]
#[ignore]
async fn snow_leopard_smoke_test() {
    println!("\n[SMOKE TEST] Running Phase 1 only...\n");

    let mut config = SnowLeopardConfig::test_mode();
    config.phases_enabled = vec![PhaseId::HardReset];
    config.perform_resets = false; // Skip reset for speed

    let result = run_benchmark(&config).await;

    assert_eq!(result.phases.len(), 1, "Should have 1 phase");
    assert_eq!(
        result.phases[0].questions.len(), 10,
        "Phase 1 should have 10 questions"
    );
    assert!(
        result.phases[0].success_rate() >= 80.0,
        "Success rate should be >= 80%"
    );

    println!("✓ Smoke test passed!");
}

// ============================================================================
// UNIT TESTS (NOT IGNORED)
// ============================================================================

/// Test Brain fast path coverage for canonical questions
#[tokio::test]
async fn test_brain_fast_path_coverage() {
    // These questions should all hit Brain fast path
    let brain_questions = [
        "What CPU do I have?",
        "How much RAM?",
        "how much free disk space?",
        "health check",
    ];

    for q in &brain_questions {
        let result = try_fast_answer(q);
        assert!(
            result.is_some(),
            "Question '{}' should hit Brain fast path",
            q
        );
    }
}

/// Test question classification for canonical questions
#[test]
fn test_question_classification() {
    for (id, text) in CANONICAL_QUESTIONS {
        let qt = FastQuestionType::classify(text);
        println!("Question '{}' classified as {:?}", id, qt);
    }
}

/// Test benchmark mode parsing from natural language
#[test]
fn test_benchmark_mode_parsing() {
    assert_eq!(parse_benchmark_mode("run the full benchmark"), BenchmarkMode::Full);
    assert_eq!(parse_benchmark_mode("full snow leopard test"), BenchmarkMode::Full);
    assert_eq!(parse_benchmark_mode("quick benchmark"), BenchmarkMode::Quick);
    assert_eq!(parse_benchmark_mode("run a short test"), BenchmarkMode::Quick);
    assert_eq!(parse_benchmark_mode("fast sanity check"), BenchmarkMode::Quick);
    assert_eq!(parse_benchmark_mode("snow leopard"), BenchmarkMode::Full);
}

/// Test benchmark request detection
#[test]
fn test_is_benchmark_request() {
    // Should detect benchmark requests
    assert!(is_benchmark_request("run the snow leopard benchmark"));
    assert!(is_benchmark_request("run a quick Snow Leopard benchmark"));
    assert!(is_benchmark_request("snow leopard tests"));
    assert!(is_benchmark_request("run benchmark"));
    assert!(is_benchmark_request("run the snow leopard tests"));

    // Should NOT detect non-benchmark requests
    assert!(!is_benchmark_request("what is my cpu?"));
    assert!(!is_benchmark_request("how much ram?"));
    assert!(!is_benchmark_request("show me a leopard"));
    assert!(!is_benchmark_request("what is snow?"));
}

/// Test config defaults
#[test]
fn test_config_defaults() {
    let default = SnowLeopardConfig::default();
    assert!(!default.use_simulated_llm);
    assert!(default.perform_resets);
    assert_eq!(default.learning_repetitions, 5);
    assert_eq!(default.phases_enabled.len(), 6);

    let test = SnowLeopardConfig::test_mode();
    assert!(test.use_simulated_llm);
    assert!(test.perform_resets);

    let quick = SnowLeopardConfig::quick_mode();
    assert_eq!(quick.phases_enabled.len(), 3);
    assert_eq!(quick.learning_repetitions, 3);
}

/// Test phase IDs
#[test]
fn test_phase_ids() {
    assert_eq!(PhaseId::all().len(), 6);
    assert_eq!(PhaseId::quick().len(), 3);

    assert_eq!(PhaseId::HardReset.number(), 1);
    assert_eq!(PhaseId::WarmState.number(), 2);
    assert_eq!(PhaseId::SoftReset.number(), 3);
    assert_eq!(PhaseId::NLStress.number(), 4);
    assert_eq!(PhaseId::NovelQuestions.number(), 5);
    assert_eq!(PhaseId::LearningTest.number(), 6);

    assert_eq!(PhaseId::HardReset.name(), "Hard Reset");
    assert_eq!(PhaseId::LearningTest.name(), "Learning Test");
}

/// Test question set sizes
#[test]
fn test_question_sets() {
    assert_eq!(CANONICAL_QUESTIONS.len(), 10);
    assert_eq!(PARAPHRASED_QUESTIONS.len(), 10);
    assert_eq!(NOVEL_QUESTIONS.len(), 10);
}

/// Test LastBenchmarkSummary formatting
#[test]
fn test_last_benchmark_summary_format() {
    let summary = LastBenchmarkSummary {
        timestamp: "2025-11-29 12:00:00".to_string(),
        mode: BenchmarkMode::Full,
        phases: 6,
        total_questions: 60,
        success_rate: 88.5,
        avg_latency_ms: 230,
        brain_usage_pct: 45.0,
        llm_usage_pct: 55.0,
        status_hint: "Anna is behaving reliably.".to_string(),
        report_path: Some("/var/lib/anna/benchmarks/test.json".to_string()),
    };

    let formatted = summary.format_for_status();
    assert!(formatted.contains("SNOW LEOPARD BENCHMARK"));
    assert!(formatted.contains("2025-11-29"));
    assert!(formatted.contains("full"));
    assert!(formatted.contains("60"));
    assert!(formatted.contains("88%"));
}

/// Test result status hints
#[tokio::test]
async fn test_result_status_hints() {
    let mut config = SnowLeopardConfig::test_mode();
    config.phases_enabled = vec![PhaseId::HardReset];
    config.perform_resets = false;
    config.verbose = false;

    let result = run_benchmark(&config).await;

    // With simulated responses, should have high success
    let hint = result.status_hint();
    assert!(
        hint.contains("excellently") || hint.contains("reliably"),
        "Status hint should indicate good performance: {}",
        hint
    );
}

/// Test that XP snapshot captures correctly
#[test]
fn test_xp_snapshot() {
    use anna_common::bench_snow_leopard::XpSnapshot;

    let snapshot = XpSnapshot::capture();

    // Basic sanity checks
    assert!(snapshot.anna_level >= 1);
    assert!(snapshot.anna_trust >= 0.0 && snapshot.anna_trust <= 1.0);
}
