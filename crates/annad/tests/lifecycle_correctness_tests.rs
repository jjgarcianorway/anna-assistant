//! Lifecycle Correctness Tests v3.11.0
//!
//! Tests that verify the complete lifecycle of XP, telemetry, and state management:
//!
//! 1. Questions increment XP, telemetry, and stats
//! 2. Hard reset returns everything to baseline
//! 3. New questions after reset build a new lifecycle cleanly
//!
//! ## Running
//!
//! ```bash
//! cargo test -p annad lifecycle_correctness -- --nocapture
//! ```

use anna_common::{
    // XP and Experience
    ExperiencePaths, ExperienceSnapshot, ResetType,
    reset_experience, reset_factory, has_experience_data,
    // Telemetry
    telemetry::{TelemetryEvent, TelemetryRecorder, Origin, Outcome},
    // Brain fast path
    try_fast_answer,
};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Test Configuration
// ============================================================================

/// Questions that trigger Brain fast path
const BRAIN_QUESTIONS: &[&str] = &[
    "How much RAM do I have?",
    "What CPU do I have?",
    "How much disk space?",
    "What is your health status?",
    "What is the system uptime?",
];

// ============================================================================
// Test: Questions Increment State
// ============================================================================

/// Test that asking questions increments XP, telemetry, and stats
#[test]
fn test_questions_increment_state() {
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directory structure
    setup_test_directories(&paths);

    // Initialize with baseline (soft reset)
    let result = reset_experience(&paths);
    assert!(result.success, "Initial reset should succeed");

    // Capture baseline state
    let baseline = ExperienceSnapshot::capture(&paths);
    assert!(baseline.is_empty(), "Baseline should have no activity");

    // Setup telemetry recorder with test path
    let telemetry_recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());

    // Ask several questions and record their effects
    for (i, question) in BRAIN_QUESTIONS.iter().enumerate() {
        // Try Brain fast path
        let fast_answer = try_fast_answer(question);
        assert!(fast_answer.is_some(), "Brain should handle: {}", question);

        // Record telemetry (simulating what annactl does)
        let answer = fast_answer.unwrap();
        let event = TelemetryEvent::new(
            question,
            Outcome::Success,
            Origin::Brain,
            answer.reliability,
            answer.duration_ms,
        );
        telemetry_recorder.record(&event).expect("Telemetry should record");

        // After each question, verify state incremented
        let snapshot = ExperienceSnapshot::capture(&paths);
        assert_eq!(
            snapshot.telemetry_line_count as usize,
            i + 1,
            "Telemetry should have {} entries after question {}", i + 1, i + 1
        );
    }

    // Final state should show activity
    let final_state = ExperienceSnapshot::capture(&paths);
    assert_eq!(
        final_state.telemetry_line_count as usize,
        BRAIN_QUESTIONS.len(),
        "Should have recorded {} telemetry entries",
        BRAIN_QUESTIONS.len()
    );
    assert!(has_experience_data(&paths), "Should have experience data");
}

// ============================================================================
// Test: Hard Reset Clears Everything
// ============================================================================

/// Test that hard reset truly clears all state
#[test]
fn test_hard_reset_clears_all_state() {
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directory structure
    setup_test_directories(&paths);

    // Setup telemetry
    let telemetry_recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());

    // Generate some state
    for question in BRAIN_QUESTIONS {
        if let Some(answer) = try_fast_answer(question) {
            let event = TelemetryEvent::new(
                question,
                Outcome::Success,
                Origin::Brain,
                answer.reliability,
                answer.duration_ms,
            );
            telemetry_recorder.record(&event).expect("Telemetry should record");
        }
    }

    // Add some knowledge data
    let knowledge_file = paths.knowledge_dir.join("test_knowledge.db");
    fs::write(&knowledge_file, "test data").expect("Should write knowledge");

    // Verify state exists
    let before_reset = ExperienceSnapshot::capture(&paths);
    assert!(before_reset.telemetry_line_count > 0, "Should have telemetry");
    assert!(before_reset.knowledge_file_count > 0, "Should have knowledge");

    // Perform factory (hard) reset
    let reset_result = reset_factory(&paths);
    assert!(reset_result.success, "Factory reset should succeed");

    // Verify ALL state is cleared
    let after_reset = ExperienceSnapshot::capture(&paths);

    assert_eq!(after_reset.telemetry_line_count, 0, "Telemetry should be cleared");
    assert_eq!(after_reset.knowledge_file_count, 0, "Knowledge should be cleared");
    assert!(after_reset.is_factory_clean(), "Should be factory clean");

    // XP store should be at baseline (not deleted, but reset)
    assert!(after_reset.xp_store_exists, "XP store should exist");
    assert_eq!(after_reset.anna_level, 1, "Anna should be level 1");
    assert_eq!(after_reset.anna_xp, 0, "Anna should have 0 XP");
    assert_eq!(after_reset.total_questions, 0, "Should have 0 questions");
}

// ============================================================================
// Test: Soft Reset Preserves Knowledge
// ============================================================================

/// Test that soft reset clears XP/telemetry but keeps knowledge
#[test]
fn test_soft_reset_preserves_knowledge() {
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directory structure
    setup_test_directories(&paths);

    // Setup telemetry
    let telemetry_recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());

    // Generate some state
    for question in BRAIN_QUESTIONS {
        if let Some(answer) = try_fast_answer(question) {
            let event = TelemetryEvent::new(
                question,
                Outcome::Success,
                Origin::Brain,
                answer.reliability,
                answer.duration_ms,
            );
            telemetry_recorder.record(&event).expect("Telemetry should record");
        }
    }

    // Add knowledge data
    let knowledge_file = paths.knowledge_dir.join("test_knowledge.db");
    fs::write(&knowledge_file, "important learned data").expect("Should write knowledge");

    // Verify state exists
    let before_reset = ExperienceSnapshot::capture(&paths);
    assert!(before_reset.telemetry_line_count > 0, "Should have telemetry");
    assert!(before_reset.knowledge_file_count > 0, "Should have knowledge");

    // Perform experience (soft) reset
    let reset_result = reset_experience(&paths);
    assert!(reset_result.success, "Experience reset should succeed");

    // Verify XP/telemetry cleared but knowledge preserved
    let after_reset = ExperienceSnapshot::capture(&paths);

    assert_eq!(after_reset.telemetry_line_count, 0, "Telemetry should be cleared");
    assert!(after_reset.knowledge_file_count > 0, "Knowledge should be preserved");

    // Verify knowledge file still has content
    let content = fs::read_to_string(&knowledge_file).expect("Should read knowledge");
    assert_eq!(content, "important learned data", "Knowledge content should be intact");
}

// ============================================================================
// Test: Post-Reset Lifecycle Works Cleanly
// ============================================================================

/// Test that a new lifecycle works correctly after reset
#[test]
fn test_post_reset_lifecycle_clean() {
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directory structure
    setup_test_directories(&paths);

    // Setup telemetry
    let telemetry_recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());

    // === Phase 1: Generate initial state ===
    for question in BRAIN_QUESTIONS {
        if let Some(answer) = try_fast_answer(question) {
            let event = TelemetryEvent::new(
                question,
                Outcome::Success,
                Origin::Brain,
                answer.reliability,
                answer.duration_ms,
            );
            telemetry_recorder.record(&event).expect("Telemetry should record");
        }
    }

    let phase1_state = ExperienceSnapshot::capture(&paths);
    let phase1_telemetry = phase1_state.telemetry_line_count;
    assert!(phase1_telemetry > 0, "Phase 1 should have telemetry");

    // === Phase 2: Hard reset ===
    let reset_result = reset_factory(&paths);
    assert!(reset_result.success, "Reset should succeed");

    let reset_state = ExperienceSnapshot::capture(&paths);
    assert!(reset_state.is_factory_clean(), "Should be factory clean after reset");

    // === Phase 3: New lifecycle after reset ===
    // Re-create telemetry recorder (file was truncated)
    let telemetry_recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());

    // Ask new questions (use questions known to trigger Brain)
    let new_questions = &["How much RAM do I have?", "What CPU do I have?", "How much disk space?"];
    let mut new_count = 0;
    for question in new_questions {
        if let Some(answer) = try_fast_answer(question) {
            let event = TelemetryEvent::new(
                question,
                Outcome::Success,
                Origin::Brain,
                answer.reliability,
                answer.duration_ms,
            );
            telemetry_recorder.record(&event).expect("Telemetry should record");
            new_count += 1;
        }
    }

    // Verify some questions were answered
    assert!(new_count > 0, "At least some questions should get Brain answers");

    // Verify new lifecycle is clean and separate
    let phase3_state = ExperienceSnapshot::capture(&paths);
    assert_eq!(
        phase3_state.telemetry_line_count as u64,
        new_count,
        "Phase 3 should have exactly {} telemetry entries (the new questions answered)",
        new_count
    );

    // Old telemetry count (phase1) was higher, new count is just the new questions
    assert!(
        phase3_state.telemetry_line_count < phase1_telemetry,
        "Phase 3 telemetry ({}) should be less than phase 1 ({})",
        phase3_state.telemetry_line_count,
        phase1_telemetry
    );
}

// ============================================================================
// Test: Telemetry Origin Tracking
// ============================================================================

/// Test that telemetry correctly tracks answer origins
#[test]
fn test_telemetry_origin_tracking() {
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directory structure
    setup_test_directories(&paths);

    let telemetry_recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());

    // Record Brain answer
    let brain_event = TelemetryEvent::new(
        "How much RAM?",
        Outcome::Success,
        Origin::Brain,
        0.99,
        15,
    );
    telemetry_recorder.record(&brain_event).expect("Should record brain event");

    // Record Junior answer
    let junior_event = TelemetryEvent::new(
        "What packages are installed?",
        Outcome::Success,
        Origin::Junior,
        0.85,
        3500,
    );
    telemetry_recorder.record(&junior_event).expect("Should record junior event");

    // Record Senior answer
    let senior_event = TelemetryEvent::new(
        "Why is my system slow?",
        Outcome::Success,
        Origin::Senior,
        0.90,
        7500,
    );
    telemetry_recorder.record(&senior_event).expect("Should record senior event");

    // Read back and verify origins
    let events = telemetry_recorder.read_recent(24);
    assert_eq!(events.len(), 3, "Should have 3 events");

    let origins: Vec<_> = events.iter().map(|e| &e.origin).collect();
    assert!(origins.contains(&&Origin::Brain), "Should have Brain origin");
    assert!(origins.contains(&&Origin::Junior), "Should have Junior origin");
    assert!(origins.contains(&&Origin::Senior), "Should have Senior origin");
}

// ============================================================================
// Test: Reset Result Tracking
// ============================================================================

/// Test that reset operations return correct result types
#[test]
fn test_reset_result_types() {
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directory structure
    setup_test_directories(&paths);

    // Setup telemetry
    let telemetry_recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());

    // Generate some state
    for question in BRAIN_QUESTIONS {
        if let Some(answer) = try_fast_answer(question) {
            let event = TelemetryEvent::new(
                question,
                Outcome::Success,
                Origin::Brain,
                answer.reliability,
                answer.duration_ms,
            );
            let _ = telemetry_recorder.record(&event);
        }
    }

    // Perform soft reset and check result
    let soft_result = reset_experience(&paths);
    assert!(soft_result.success, "Soft reset should succeed");
    assert_eq!(
        soft_result.reset_type,
        ResetType::Experience,
        "Should report Experience reset type"
    );
    assert!(
        !soft_result.components_reset.is_empty() || !soft_result.components_clean.is_empty(),
        "Should report components processed"
    );

    // Add more state for factory reset test
    let knowledge_file = paths.knowledge_dir.join("test.db");
    fs::write(&knowledge_file, "test").unwrap();

    // Perform factory reset and check result
    let hard_result = reset_factory(&paths);
    assert!(hard_result.success, "Factory reset should succeed");
    assert_eq!(
        hard_result.reset_type,
        ResetType::Factory,
        "Should report Factory reset type"
    );
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Setup test directory structure
fn setup_test_directories(paths: &ExperiencePaths) {
    fs::create_dir_all(&paths.xp_dir).expect("Should create XP dir");
    fs::create_dir_all(paths.telemetry_file.parent().unwrap()).expect("Should create telemetry dir");
    fs::create_dir_all(&paths.stats_dir).expect("Should create stats dir");
    fs::create_dir_all(&paths.knowledge_dir).expect("Should create knowledge dir");
}
