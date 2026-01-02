//! Validation and guardrail tests for specialist protocol.
//!
//! Tests for:
//! - Vague language detection
//! - Guardrail enforcement
//! - Full pipeline validation

use super::*;

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
            validation
                .errors
                .iter()
                .any(|e| matches!(e, ValidationError::VagueLanguage(_))),
            "Vague response '{}' should be flagged",
            summary
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

    assert!(
        result.passed,
        "Valid response should pass guardrails: {:?}",
        result.violations
    );
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
