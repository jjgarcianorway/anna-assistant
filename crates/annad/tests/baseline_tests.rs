//! Baseline Performance Tests v1.0.0
//!
//! See `docs/architecture.md` Section 2 for latency targets.
//!
//! These tests verify that core operations meet baseline performance requirements:
//! - Brain fast path: <150ms latency, 0 LLM calls
//! - Reliability thresholds: Green >= 0.90, Yellow >= 0.70
//!
//! ## Architecture Note (v1.0.0)
//!
//! These tests are DETERMINISTIC - they don't require LLM or network.
//! They test the Brain layer and data structures only.
//!
//! For LLM path tests, see `orchestration_tests.rs` which uses fake LLM clients.

use anna_common::{
    try_fast_answer, FastQuestionType,
    TrustLevel, ReliabilityScale,
};
use std::time::Instant;

// ============================================================================
// Architecture Constants - From docs/architecture.md Section 2
// ============================================================================

/// Brain fast path maximum latency (150ms)
const BRAIN_LATENCY_MAX_MS: u128 = 150;

/// Green reliability threshold (90%)
const GREEN_THRESHOLD: f64 = 0.90;

/// Yellow reliability threshold (70%)
const YELLOW_THRESHOLD: f64 = 0.70;

// ============================================================================
// Brain Fast Path Latency Tests
// ============================================================================

/// Brain RAM question should complete in <150ms
#[test]
fn test_brain_ram_latency() {
    let start = Instant::now();
    let answer = try_fast_answer("How much RAM do I have?");
    let elapsed = start.elapsed().as_millis();

    assert!(answer.is_some(), "Brain should handle RAM questions");
    assert!(
        elapsed < BRAIN_LATENCY_MAX_MS,
        "Brain RAM answer took {}ms, expected <{}ms",
        elapsed,
        BRAIN_LATENCY_MAX_MS
    );

    let answer = answer.unwrap();
    assert_eq!(answer.origin, "Brain");
    assert!(answer.reliability >= GREEN_THRESHOLD);
}

/// Brain CPU question should complete in <150ms
#[test]
fn test_brain_cpu_latency() {
    let start = Instant::now();
    let answer = try_fast_answer("How many CPU cores do I have?");
    let elapsed = start.elapsed().as_millis();

    assert!(answer.is_some(), "Brain should handle CPU questions");
    assert!(
        elapsed < BRAIN_LATENCY_MAX_MS,
        "Brain CPU answer took {}ms, expected <{}ms",
        elapsed,
        BRAIN_LATENCY_MAX_MS
    );

    let answer = answer.unwrap();
    assert_eq!(answer.origin, "Brain");
    assert!(answer.reliability >= GREEN_THRESHOLD);
}

/// Brain disk question should complete in <150ms
#[test]
fn test_brain_disk_latency() {
    let start = Instant::now();
    let answer = try_fast_answer("How much free disk space do I have?");
    let elapsed = start.elapsed().as_millis();

    assert!(answer.is_some(), "Brain should handle disk questions");
    assert!(
        elapsed < BRAIN_LATENCY_MAX_MS,
        "Brain disk answer took {}ms, expected <{}ms",
        elapsed,
        BRAIN_LATENCY_MAX_MS
    );

    let answer = answer.unwrap();
    assert_eq!(answer.origin, "Brain");
    assert!(answer.reliability >= GREEN_THRESHOLD);
}

/// Brain health check should complete in <150ms
#[test]
fn test_brain_health_latency() {
    let start = Instant::now();
    let answer = try_fast_answer("Are you healthy Anna?");
    let elapsed = start.elapsed().as_millis();

    assert!(answer.is_some(), "Brain should handle health questions");
    assert!(
        elapsed < BRAIN_LATENCY_MAX_MS,
        "Brain health answer took {}ms, expected <{}ms",
        elapsed,
        BRAIN_LATENCY_MAX_MS
    );

    let answer = answer.unwrap();
    assert_eq!(answer.origin, "Brain");
}

/// Brain should NOT handle unknown questions
#[test]
fn test_brain_unknown_returns_none() {
    let start = Instant::now();
    let answer = try_fast_answer("What is the meaning of life?");
    let elapsed = start.elapsed().as_millis();

    // Should return None quickly (classification only, no command execution)
    assert!(answer.is_none(), "Brain should NOT handle unknown questions");
    assert!(
        elapsed < 10, // Classification should be near-instant
        "Brain classification took {}ms, expected <10ms",
        elapsed
    );
}

// ============================================================================
// Reliability Threshold Tests
// ============================================================================

/// Green reliability (>=90%) should give full XP
#[test]
fn test_reliability_green_scaling() {
    let scale = ReliabilityScale::from_reliability(0.95);
    assert_eq!(scale.quality, "Green");
    assert_eq!(scale.xp_multiplier, 1.0);
    assert_eq!(scale.trust_multiplier, 1.0);
}

/// Yellow reliability (70-90%) should give reduced XP
#[test]
fn test_reliability_yellow_scaling() {
    let scale = ReliabilityScale::from_reliability(0.80);
    assert_eq!(scale.quality, "Yellow");
    assert_eq!(scale.xp_multiplier, 0.75);
    assert_eq!(scale.trust_multiplier, 0.5);
}

/// Orange reliability (50-70%) should give minimal XP
#[test]
fn test_reliability_orange_scaling() {
    let scale = ReliabilityScale::from_reliability(0.60);
    assert_eq!(scale.quality, "Orange");
    assert_eq!(scale.xp_multiplier, 0.5);
    assert_eq!(scale.trust_multiplier, 0.25);
}

/// Red reliability (<50%) should give tiny XP
#[test]
fn test_reliability_red_scaling() {
    let scale = ReliabilityScale::from_reliability(0.30);
    assert_eq!(scale.quality, "Red");
    assert_eq!(scale.xp_multiplier, 0.25);
    assert_eq!(scale.trust_multiplier, 0.0);
}

/// No reliability should give XP penalty
#[test]
fn test_reliability_none_scaling() {
    let scale = ReliabilityScale::from_reliability(0.0);
    assert_eq!(scale.quality, "None");
    assert_eq!(scale.xp_multiplier, 0.0);
    assert!(scale.trust_multiplier < 0.0); // Penalty
}

// ============================================================================
// Trust Level Tests
// ============================================================================

/// Low trust threshold is 0.4
#[test]
fn test_trust_level_low() {
    assert_eq!(TrustLevel::from_trust(0.2), TrustLevel::Low);
    assert_eq!(TrustLevel::from_trust(0.39), TrustLevel::Low);
}

/// Normal trust is 0.4-0.7
#[test]
fn test_trust_level_normal() {
    assert_eq!(TrustLevel::from_trust(0.4), TrustLevel::Normal);
    assert_eq!(TrustLevel::from_trust(0.5), TrustLevel::Normal);
    assert_eq!(TrustLevel::from_trust(0.7), TrustLevel::Normal);
}

/// High trust is >0.7
#[test]
fn test_trust_level_high() {
    assert_eq!(TrustLevel::from_trust(0.71), TrustLevel::High);
    assert_eq!(TrustLevel::from_trust(0.9), TrustLevel::High);
}

// ============================================================================
// Question Classification Tests
// ============================================================================

/// RAM questions should be classified correctly
#[test]
fn test_classify_ram_questions() {
    assert_eq!(FastQuestionType::classify("How much RAM?"), FastQuestionType::Ram);
    assert_eq!(FastQuestionType::classify("total memory"), FastQuestionType::Ram);
    assert_eq!(FastQuestionType::classify("how many gb of ram"), FastQuestionType::Ram);
}

/// CPU questions should be classified correctly
#[test]
fn test_classify_cpu_questions() {
    // Must have "how many" + cpu/core/thread/processor
    assert_eq!(FastQuestionType::classify("how many cpu cores"), FastQuestionType::CpuCores);
    assert_eq!(FastQuestionType::classify("how many threads do I have"), FastQuestionType::CpuCores);
}

/// Disk questions should be classified correctly
#[test]
fn test_classify_disk_questions() {
    assert_eq!(FastQuestionType::classify("disk space on /"), FastQuestionType::RootDiskSpace);
    assert_eq!(FastQuestionType::classify("free storage"), FastQuestionType::RootDiskSpace);
}

/// Health questions should be classified correctly
#[test]
fn test_classify_health_questions() {
    // Requires specific patterns: "are you ok/healthy/working" or "diagnose yourself" or "your health"
    assert_eq!(FastQuestionType::classify("are you ok?"), FastQuestionType::AnnaHealth);
    assert_eq!(FastQuestionType::classify("diagnose yourself"), FastQuestionType::AnnaHealth);
}

/// Unknown questions should be classified as Unknown
#[test]
fn test_classify_unknown_questions() {
    assert_eq!(FastQuestionType::classify("install nginx"), FastQuestionType::Unknown);
    assert_eq!(FastQuestionType::classify("what is rust"), FastQuestionType::Unknown);
}

// ============================================================================
// Reliability Range Invariants
// ============================================================================

/// Green threshold should be higher than yellow
#[test]
fn test_reliability_thresholds_ordered() {
    assert!(GREEN_THRESHOLD > YELLOW_THRESHOLD);
    assert!(YELLOW_THRESHOLD > 0.0);
    assert!(GREEN_THRESHOLD <= 1.0);
}

/// Brain answers should always be Green reliability
#[test]
fn test_brain_answers_are_green() {
    // Test all Brain question types
    let questions = [
        "How much RAM do I have?",
        "How many CPU cores?",
        "Free disk space?",
    ];

    for q in questions {
        if let Some(answer) = try_fast_answer(q) {
            assert!(
                answer.reliability >= GREEN_THRESHOLD,
                "Brain answer for '{}' has reliability {} < {}",
                q,
                answer.reliability,
                GREEN_THRESHOLD
            );
        }
    }
}
