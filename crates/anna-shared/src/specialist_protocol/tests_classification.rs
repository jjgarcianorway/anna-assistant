//! Classification and stats tests for specialist protocol.
//!
//! Tests for:
//! - Intent classification accuracy
//! - Honest ticket statistics
//! - Response classification

use super::*;

// Test: Stats are honest
#[test]
fn test_honest_stats() {
    let mut stats = HonestTicketStats::default();

    // Record mixed outcomes
    stats.record(TicketOutcome::Success, 500);
    stats.record(TicketOutcome::Success, 600);
    stats.record(TicketOutcome::UsefulPartial, 800);
    stats.record(TicketOutcome::Failed, 200);
    stats.record_parse_error();

    assert_eq!(stats.total, 5);
    assert_eq!(stats.success, 2);
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.internal_errors, 1); // parse error

    // Success rate should NOT be 100%
    assert!(
        stats.success_rate() < 100.0,
        "Success rate should not be 100% with failures"
    );
    assert_eq!(stats.success_rate(), 40.0); // 2/5 = 40%

    // Resolution rate includes partials
    assert_eq!(stats.resolved(), 3); // 2 success + 1 partial
    assert!((stats.resolution_rate() - 60.0).abs() < 0.01); // 3/5 = 60%

    // Validation should pass (stats are honest)
    assert!(stats.validate().is_ok());
}

// Test: Intent classification accuracy
#[test]
fn test_intent_classification() {
    // State queries
    let state_queries = vec![
        "Do I have any failed services?",
        "How much RAM do I have?",
        "Is nginx running?",
        "Show me my disk usage",
        "What is my IP address?",
    ];

    for query in state_queries {
        let intent = classify_intent(query);
        assert!(
            intent == IntentType::CheckState || intent == IntentType::Unknown,
            "State query '{}' should be CheckState, got {:?}",
            query,
            intent
        );
    }

    // How-to queries
    let howto_queries = vec![
        "How do I configure nginx?",
        "How can I install vim?",
        "How to set up ssh?",
    ];

    for query in howto_queries {
        let intent = classify_intent(query);
        assert_eq!(
            intent,
            IntentType::HowTo,
            "How-to query '{}' should be HowTo",
            query
        );
    }
}
