//! Status Consistency Tests v3.9.0
//!
//! Tests that verify status output is never self-contradictory.
//! These tests ensure the v3.9.0 "Consistency & Migration" patch works correctly.
//!
//! ## Test Scenarios
//!
//! 1. Fresh install - all zeros, no contradictions
//! 2. After experience reset - XP reset but history explains why
//! 3. After 24h inactivity - questions persist but 24h metrics are zero (explained)
//! 4. Active usage - all data stores consistent

use anna_common::{
    status_coherence::{
        CoherentStatus, InconsistencyCode, ResetHistory, StatusInconsistency,
        telemetry_status_message, xp_events_status_message,
    },
    xp_log::Metrics24h,
};

// ============================================================================
// Status Message Tests
// ============================================================================

#[test]
fn test_fresh_install_messages_are_helpful() {
    // Simulate fresh install state
    let status = CoherentStatus {
        total_questions: 0,
        anna_xp: 0,
        anna_level: 1,
        xp_store_exists: false,
        xp_events_24h: Metrics24h::default(),
        xp_log_exists: false,
        telemetry_exists: false,
        telemetry_has_data: false,
        telemetry_question_count: 0,
        autoprovision_ran: false,
        models_selected: false,
        recent_reset: None,
        was_recently_reset: false,
        inconsistencies: vec![],
    };

    // Messages should be helpful, not confusing
    let telemetry_msg = telemetry_status_message(&status);
    assert!(
        telemetry_msg.contains("Ask") || telemetry_msg.contains("yet"),
        "Fresh install telemetry message should invite action: '{}'",
        telemetry_msg
    );

    let xp_msg = xp_events_status_message(&status);
    assert!(
        xp_msg.contains("Ask") || xp_msg.contains("yet") || xp_msg.contains("No XP"),
        "Fresh install XP message should be clear: '{}'",
        xp_msg
    );
}

#[test]
fn test_questions_without_xp_events_gets_explanation() {
    // After 24h of inactivity, questions persist but XP events are pruned
    let mut status = CoherentStatus {
        total_questions: 100, // Many questions answered
        anna_xp: 5000,
        anna_level: 5,
        xp_store_exists: true,
        xp_events_24h: Metrics24h::default(), // But no 24h events (pruned)
        xp_log_exists: true,
        telemetry_exists: true,
        telemetry_has_data: true,
        telemetry_question_count: 100,
        autoprovision_ran: true,
        models_selected: true,
        recent_reset: None,
        was_recently_reset: false,
        inconsistencies: vec![],
    };

    status.inconsistencies = vec![StatusInconsistency {
        code: InconsistencyCode::QuestionsWithoutXpEvents,
        explanation: "No XP events in last 24 hours (events older than 24h are pruned).".to_string(),
    }];

    let xp_msg = xp_events_status_message(&status);
    assert!(
        xp_msg.contains("24") || xp_msg.contains("pruned") || xp_msg.contains("older"),
        "Message should explain 24h window: '{}'",
        xp_msg
    );
}

#[test]
fn test_after_reset_explanation_mentions_reset() {
    use anna_common::status_coherence::ResetEvent;
    use chrono::Utc;

    // After a reset, XP is zero but questions might persist in XP store
    let mut status = CoherentStatus {
        total_questions: 100, // Legacy count
        anna_xp: 0,           // Reset to zero
        anna_level: 1,        // Reset to level 1
        xp_store_exists: true,
        xp_events_24h: Metrics24h::default(),
        xp_log_exists: false, // Cleared by reset
        telemetry_exists: true,
        telemetry_has_data: false, // Cleared by reset
        telemetry_question_count: 0,
        autoprovision_ran: true,
        models_selected: true,
        recent_reset: Some(ResetEvent {
            timestamp: Utc::now().to_rfc3339(),
            reset_type: "experience".to_string(),
            version: "3.9.0".to_string(),
            questions_before: 100,
            xp_before: 5000,
        }),
        was_recently_reset: true,
        inconsistencies: vec![],
    };

    // Analyze consistency
    status.inconsistencies = vec![
        StatusInconsistency {
            code: InconsistencyCode::QuestionsWithoutTelemetry,
            explanation: "Telemetry cleared by recent reset. Questions count persists from XP store.".to_string(),
        },
        StatusInconsistency {
            code: InconsistencyCode::QuestionsWithoutXpEvents,
            explanation: "XP events cleared by recent reset. Questions count persists from XP store.".to_string(),
        },
    ];

    // Both messages should mention reset
    let telemetry_msg = telemetry_status_message(&status);
    assert!(
        telemetry_msg.contains("reset") || telemetry_msg.contains("cleared"),
        "Telemetry message should mention reset: '{}'",
        telemetry_msg
    );

    let xp_msg = xp_events_status_message(&status);
    assert!(
        xp_msg.contains("reset") || xp_msg.contains("cleared"),
        "XP message should mention reset: '{}'",
        xp_msg
    );
}

#[test]
fn test_consistent_data_shows_no_messages() {
    // When all data is consistent, no special messages needed
    let status = CoherentStatus {
        total_questions: 50,
        anna_xp: 2000,
        anna_level: 3,
        xp_store_exists: true,
        xp_events_24h: Metrics24h {
            total_events: 10,
            xp_gained: 100,
            xp_lost: 10,
            net_xp: 90,
            positive_events: 8,
            negative_events: 2,
            top_positive: Some("BrainSelfSolve".to_string()),
            top_negative: None,
            questions_answered: 10,
        },
        xp_log_exists: true,
        telemetry_exists: true,
        telemetry_has_data: true,
        telemetry_question_count: 50,
        autoprovision_ran: true,
        models_selected: true,
        recent_reset: None,
        was_recently_reset: false,
        inconsistencies: vec![],
    };

    // No special messages needed - data is consistent
    // XP events message should be empty since we have events
    let xp_msg = xp_events_status_message(&status);
    assert!(xp_msg.is_empty(), "Consistent data should show no special XP message: '{}'", xp_msg);

    // Telemetry message should be empty since we have data
    let telemetry_msg = telemetry_status_message(&status);
    assert!(telemetry_msg.is_empty(), "Consistent data should show no special telemetry message: '{}'", telemetry_msg);
}

// ============================================================================
// Reset History Tests
// ============================================================================

#[test]
fn test_reset_history_tracks_events() {
    // This is a unit test - we just verify serialization/deserialization works
    let history = ResetHistory {
        resets: vec![
            anna_common::status_coherence::ResetEvent {
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                reset_type: "experience".to_string(),
                version: "3.8.0".to_string(),
                questions_before: 50,
                xp_before: 2000,
            },
            anna_common::status_coherence::ResetEvent {
                timestamp: "2024-02-01T00:00:00Z".to_string(),
                reset_type: "factory".to_string(),
                version: "3.9.0".to_string(),
                questions_before: 100,
                xp_before: 5000,
            },
        ],
    };

    // Should have 2 resets
    assert_eq!(history.resets.len(), 2);

    // Last reset should be factory
    let last = history.last_reset().unwrap();
    assert_eq!(last.reset_type, "factory");
    assert_eq!(last.questions_before, 100);
}

// ============================================================================
// Invariant Tests: Status Must Never Lie
// ============================================================================

#[test]
fn test_status_never_shows_questions_and_no_telemetry_without_explanation() {
    // INVARIANT: If questions > 0 and telemetry is empty, there MUST be an explanation

    // Simulated bad state (questions > 0, no telemetry, no explanation)
    let mut status = CoherentStatus {
        total_questions: 100,
        anna_xp: 0,
        anna_level: 1,
        xp_store_exists: true,
        xp_events_24h: Metrics24h::default(),
        xp_log_exists: false,
        telemetry_exists: false,
        telemetry_has_data: false,
        telemetry_question_count: 0,
        autoprovision_ran: false,
        models_selected: false,
        recent_reset: None,
        was_recently_reset: false,
        inconsistencies: vec![],
    };

    // Analyze - this should detect the inconsistency
    // (In real code, analyze_consistency() does this, but it's private)
    // Here we verify that telemetry_status_message provides context
    let msg = telemetry_status_message(&status);

    // The message should NOT be empty when there's an inconsistency
    // It should explain why telemetry is missing
    assert!(
        !msg.is_empty() || status.is_fresh_install(),
        "Status must explain missing telemetry when questions > 0"
    );
}

#[test]
fn test_is_fresh_install_detection_accurate() {
    // Fresh install: all zeros
    let fresh = CoherentStatus {
        total_questions: 0,
        anna_xp: 0,
        anna_level: 1,
        xp_store_exists: false,
        xp_events_24h: Metrics24h::default(),
        xp_log_exists: false,
        telemetry_exists: false,
        telemetry_has_data: false,
        telemetry_question_count: 0,
        autoprovision_ran: false,
        models_selected: false,
        recent_reset: None,
        was_recently_reset: false,
        inconsistencies: vec![],
    };
    assert!(fresh.is_fresh_install(), "All zeros should be fresh install");

    // Not fresh: has questions
    let has_questions = CoherentStatus {
        total_questions: 1, // At least one question
        ..fresh.clone()
    };
    assert!(!has_questions.is_fresh_install(), "Questions > 0 means not fresh");

    // Not fresh: has XP
    let has_xp = CoherentStatus {
        anna_xp: 1, // At least some XP
        ..fresh.clone()
    };
    assert!(!has_xp.is_fresh_install(), "XP > 0 means not fresh");

    // Not fresh: has telemetry
    let has_telemetry = CoherentStatus {
        telemetry_has_data: true,
        ..fresh.clone()
    };
    assert!(!has_telemetry.is_fresh_install(), "Telemetry data means not fresh");
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_xp_without_questions_is_flagged() {
    // This shouldn't happen normally but could from data corruption
    let status = CoherentStatus {
        total_questions: 0, // No questions
        anna_xp: 1000,      // But has XP (weird!)
        anna_level: 2,
        xp_store_exists: true,
        xp_events_24h: Metrics24h::default(),
        xp_log_exists: false,
        telemetry_exists: false,
        telemetry_has_data: false,
        telemetry_question_count: 0,
        autoprovision_ran: false,
        models_selected: false,
        recent_reset: None,
        was_recently_reset: false,
        // Include the inconsistency directly
        inconsistencies: vec![
            StatusInconsistency {
                code: InconsistencyCode::XpWithoutQuestions,
                explanation: "XP accumulated but no questions recorded (possible data migration).".to_string(),
            },
        ],
    };

    // The status should not claim to be consistent
    assert!(!status.is_consistent(), "XP without questions should be inconsistent");
}
