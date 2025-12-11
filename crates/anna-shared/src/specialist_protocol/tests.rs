//! Acceptance criteria tests for specialist protocol (v0.0.428).
//!
//! Tests from the spec:
//! 1. "How much free RAM do I have right now?" - single number, grounded
//! 2. "Do I have any failed systemd services?" - yes/no, not tutorial
//! 3. Timeout on complex question - partial/failure, not "Failed to parse"
//! 4. Nonsense detection - "unknown is installed" blocked

use super::*;
use std::collections::HashMap;

// Test 1: RAM question should get direct answer
#[test]
fn test_ram_question_direct_answer() {
    // Create a successful response about RAM
    let response = StrictResponse::success(
        "performance.memory",
        "check_free_ram",
        "17.0 GiB available out of 31.0 GiB (54%)",
        vec![
            "17.0 GiB available".to_string(),
            "31.0 GiB total RAM".to_string(),
            "No swap configured".to_string(),
        ],
        vec![ProbeEvidence {
            id: "free".to_string(),
            summary: "Mem: 31Gi, Available: 17Gi".to_string(),
            raw_reference: Some("/proc/meminfo".to_string()),
        }],
        ResponseMeta {
            handled_by: "Sofia (Desktop Administrator)".to_string(),
            ticket_id: "DSK-001".to_string(),
            version: 1,
        },
    );

    // Validate
    let validation = validate_response(&response);
    assert!(validation.valid, "Response should be valid: {:?}", validation.errors);

    // Check it's not a tutorial
    let response_type = classify_response(&response);
    assert_ne!(response_type, ResponseType::Tutorial, "RAM answer should not be a tutorial");

    // Check outcome
    let outcome = determine_outcome(&response, &validation);
    assert_eq!(outcome, TicketOutcome::Success, "Should be full success");

    // Check learnable
    assert!(response.is_learnable(), "High-confidence answer with evidence should be learnable");
}

// Test 2: Failed services question - yes/no, not tutorial
#[test]
fn test_failed_services_yes_no_answer() {
    // Case A: No failed services
    let no_failures = StrictResponse::success(
        "services.systemd",
        "check_failed_services",
        "No failed systemd services.",
        vec!["0 failed units".to_string()],
        vec![ProbeEvidence {
            id: "systemctl_failed".to_string(),
            summary: "0 loaded units listed".to_string(),
            raw_reference: None,
        }],
        ResponseMeta {
            handled_by: "Sofia".to_string(),
            ticket_id: "DSK-002".to_string(),
            version: 1,
        },
    );

    let validation = validate_response(&no_failures);
    assert!(validation.valid);
    assert_eq!(classify_response(&no_failures), ResponseType::StateAnswer);

    // Case B: Has failed services
    let has_failures = StrictResponse::success(
        "services.systemd",
        "check_failed_services",
        "You have 2 failed systemd services: nginx.service, redis.service",
        vec![
            "nginx.service: failed".to_string(),
            "redis.service: failed".to_string(),
        ],
        vec![ProbeEvidence {
            id: "systemctl_failed".to_string(),
            summary: "2 failed units: nginx.service, redis.service".to_string(),
            raw_reference: None,
        }],
        ResponseMeta {
            handled_by: "Sofia".to_string(),
            ticket_id: "DSK-003".to_string(),
            version: 1,
        },
    );

    let validation = validate_response(&has_failures);
    assert!(validation.valid);
    assert_eq!(classify_response(&has_failures), ResponseType::StateAnswer);

    // Case C: Tutorial response - should be flagged
    let tutorial_response = StrictResponse::success(
        "services.systemd",
        "check_failed_services",
        "Step 1: Run systemctl --failed. Step 2: Check the logs with journalctl.",
        vec!["Here's how to debug systemd services".to_string()],
        vec![],
        ResponseMeta {
            handled_by: "Sofia".to_string(),
            ticket_id: "DSK-004".to_string(),
            version: 1,
        },
    );

    let validation = validate_response(&tutorial_response);
    // Should have violations
    assert!(
        validation.errors.iter().any(|e| matches!(e, ValidationError::GenericHowTo)),
        "Tutorial response to state question should be flagged"
    );
}

// Test 3: Timeout should give partial/failure, not "Failed to parse"
#[test]
fn test_timeout_graceful_degradation() {
    let ctx = FallbackContext {
        ticket_id: "DSK-005".to_string(),
        domain: "storage.disk".to_string(),
        intent: "check_disk_usage".to_string(),
        question: "How much disk space do I have?".to_string(),
        probe_results: {
            let mut m = HashMap::new();
            m.insert(
                "df".to_string(),
                "Filesystem     Size  Used Avail Use% Mounted on\n/dev/sda1      803G  773G   30G  97% /".to_string()
            );
            m
        },
        reason: FallbackReason::Timeout,
        elapsed_ms: 30000,
    };

    let response = generate_fallback(&ctx);

    // Should be partial (we have probe data) or failure
    assert!(
        response.status == ResponseStatus::Partial || response.status == ResponseStatus::Failure,
        "Timeout should give partial or failure, not crash"
    );

    // Should NOT say "Failed to parse specialist response"
    assert!(
        !response.summary.contains("Failed to parse"),
        "Should never show parse errors to user: {}", response.summary
    );

    // Should have some useful info from probes
    if response.status == ResponseStatus::Partial {
        assert!(!response.details.key_facts.is_empty() || !response.evidence.probes_used.is_empty(),
            "Partial response should have some facts or evidence");
    }

    // Outcome should be usefulpartial or failure, not internal error
    let validation = validate_response(&response);
    let outcome = determine_outcome(&response, &validation);
    assert!(
        outcome == TicketOutcome::UsefulPartial
            || outcome == TicketOutcome::Failed
            || outcome == TicketOutcome::HonestUnknown,
        "Timeout outcome should be honest: {:?}", outcome
    );
}

// Test 4: Nonsense detection - "unknown is installed" blocked
#[test]
fn test_nonsense_detection_blocked() {
    let nonsense_responses = vec![
        ("unknown is installed on your system", "unknown is installed"),
        ("You have unknown is installed", "unknown pattern"),
        ("2 is installed", "numeric nonsense"),
        ("1 is installed on your machine", "numeric nonsense"),
        ("**unknown** is your package manager", "markdown unknown"),
    ];

    for (summary, description) in nonsense_responses {
        let response = StrictResponse::success(
            "packages",
            "check_installed",
            summary,
            vec![],
            vec![],
            ResponseMeta::default(),
        );

        let validation = validate_response(&response);
        assert!(
            !validation.valid,
            "Nonsense '{}' ({}) should be invalid: {:?}",
            summary, description, validation.errors
        );
        assert!(
            validation.errors.iter().any(|e| matches!(e, ValidationError::ForbiddenPattern(_))),
            "Should have forbidden pattern error for '{}': {:?}", summary, validation.errors
        );
    }
}

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
    assert!(stats.success_rate() < 100.0, "Success rate should not be 100% with failures");
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
            "State query '{}' should be CheckState, got {:?}", query, intent
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
        assert_eq!(intent, IntentType::HowTo, "How-to query '{}' should be HowTo", query);
    }
}

// Test: Vague language blocked in success responses
#[test]
fn test_vague_language_blocked() {
    let vague_responses = vec![
        "Your system might be running low on memory",
        "This could be a disk issue",
        "Perhaps you should restart the service",
        "I think there are some failed units",
        "It appears to be working",
    ];

    for summary in vague_responses {
        let response = StrictResponse::success(
            "system",
            "check",
            summary,
            vec![],
            vec![ProbeEvidence {
                id: "test".to_string(),
                summary: "test".to_string(),
                raw_reference: None,
            }],
            ResponseMeta::default(),
        );

        let validation = validate_response(&response);
        assert!(
            validation.errors.iter().any(|e| matches!(e, ValidationError::VagueLanguage(_))),
            "Vague response '{}' should be flagged", summary
        );
    }
}

// Test: Guardrail context with probes
#[test]
fn test_guardrail_with_probes() {
    let response = StrictResponse::success(
        "services.systemd",
        "check_failed_services",
        "No failed systemd services.",
        vec!["0 failed units".to_string()],
        vec![ProbeEvidence {
            id: "systemctl_failed".to_string(),
            summary: "0 failed".to_string(),
            raw_reference: None,
        }],
        ResponseMeta::default(),
    );

    let ctx = GuardrailContext::from_question("Do I have any failed services?", "services.systemd")
        .with_probe("systemctl_failed", "0 loaded units listed.");

    let validation = validate_response(&response);
    let result = check_guardrails(&response, &ctx, &validation);

    assert!(result.passed, "Valid response should pass guardrails: {:?}", result.violations);
    assert_eq!(result.outcome, TicketOutcome::Success);
}

// Test: Full pipeline with guardrails
#[test]
fn test_full_pipeline() {
    let valid_json = r#"{
        "status": "success",
        "confidence": 0.9,
        "domain": "services.systemd",
        "intent": "check_failed_services",
        "summary": "No failed systemd services detected.",
        "details": {
            "key_facts": ["0 failed units"],
            "diagnosis": null,
            "recommendations": []
        },
        "evidence": {
            "probes_used": [
                { "id": "systemctl_failed", "summary": "0 failed units" }
            ]
        },
        "meta": { "handled_by": "Sofia", "ticket_id": "T-1" }
    }"#;

    let ctx = GuardrailContext::from_question("Do I have any failed services?", "services.systemd")
        .with_probe("systemctl_failed", "0 loaded units listed.");

    let (response, result) = process_with_guardrails(valid_json, &ctx);

    assert_eq!(response.status, ResponseStatus::Success);
    assert!(result.passed || result.violations.is_empty());
}
