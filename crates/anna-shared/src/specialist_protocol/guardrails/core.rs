//! Core guardrail checking and processing logic.

use super::context::GuardrailContext;
use super::intent::IntentType;
use super::response_type::classify_response;
use super::violations::{
    check_intent_match, check_invented_facts, is_vague_state_answer, GuardrailViolation,
};
use crate::specialist_protocol::{
    fallback::{generate_fallback, FallbackContext, FallbackReason},
    outcome::{determine_outcome, TicketOutcome},
    parser::{parse_specialist_response, ParseOutcome},
    schema::StrictResponse,
    validation_core::{is_useful_response, validate_response},
    validation_types::ValidationResult,
};

/// Guardrail check result
#[derive(Debug)]
pub struct GuardrailResult {
    /// Whether the response passes guardrails
    pub passed: bool,
    /// Violations found
    pub violations: Vec<GuardrailViolation>,
    /// Adjusted response (if violations were auto-fixed)
    pub adjusted_response: Option<StrictResponse>,
    /// Final outcome for stats
    pub outcome: TicketOutcome,
}

/// Check response against guardrails
pub fn check_guardrails(
    response: &StrictResponse,
    ctx: &GuardrailContext,
    validation: &ValidationResult,
) -> GuardrailResult {
    let mut violations = vec![];

    // 1. Check intent match
    let response_type = classify_response(response);
    if let Some(violation) = check_intent_match(ctx.intent_type, response_type) {
        violations.push(violation);
    }

    // 2. Check for invented facts
    if let Some(invented) = check_invented_facts(response, &ctx.available_probes) {
        violations.push(invented);
    }

    // 3. Check validation errors
    if !validation.valid {
        let error_strs: Vec<String> = validation.errors.iter().map(|e| e.to_string()).collect();
        violations.push(GuardrailViolation::ValidationFailed(error_strs));
    }

    // 4. Check for vagueness in state queries
    if ctx.intent_type == IntentType::CheckState && is_vague_state_answer(response) {
        violations.push(GuardrailViolation::TooVague);
    }

    // Determine outcome
    let outcome = if violations.is_empty() {
        determine_outcome(response, validation)
    } else {
        // Has violations - check severity
        let has_severe = violations.iter().any(|v| {
            matches!(
                v,
                GuardrailViolation::InventedFacts(_) | GuardrailViolation::ValidationFailed(_)
            )
        });

        if has_severe {
            TicketOutcome::InternalError
        } else {
            TicketOutcome::UsefulPartial
        }
    };

    GuardrailResult {
        passed: violations.is_empty(),
        violations,
        adjusted_response: None, // Could implement auto-fixing in future
        outcome,
    }
}

/// Process a specialist response through all guardrails
pub fn process_with_guardrails(
    raw_output: &str,
    ctx: &GuardrailContext,
) -> (StrictResponse, GuardrailResult) {
    // Parse the response
    let parse_result = parse_specialist_response(raw_output);

    match parse_result {
        ParseOutcome::Success(response, validation)
        | ParseOutcome::ValidationFailed(response, validation) => {
            let guardrail_result = check_guardrails(&response, ctx, &validation);

            if guardrail_result.passed {
                (response, guardrail_result)
            } else {
                // Try to use fallback if guardrails failed
                let fallback_ctx = FallbackContext {
                    ticket_id: response.meta.ticket_id.clone(),
                    domain: ctx.domain.clone(),
                    intent: response.intent.clone(),
                    question: ctx.question.clone(),
                    probe_results: ctx.available_probes.clone(),
                    reason: FallbackReason::ValidationFailed("Guardrail check failed".to_string()),
                    elapsed_ms: response.metrics.latency_ms,
                };

                let fallback_response = generate_fallback(&fallback_ctx);
                let fallback_validation = validate_response(&fallback_response);
                let fallback_guardrails =
                    check_guardrails(&fallback_response, ctx, &fallback_validation);

                // Return fallback if it's better, otherwise original
                if is_useful_response(&fallback_response) && fallback_guardrails.passed {
                    (fallback_response, fallback_guardrails)
                } else {
                    (response, guardrail_result)
                }
            }
        }

        ParseOutcome::NoJson { .. }
        | ParseOutcome::InvalidJson { .. }
        | ParseOutcome::SchemaMismatch { .. } => {
            // Use fallback
            let reason = parse_result
                .to_fallback_reason()
                .unwrap_or(FallbackReason::ParseError(
                    "Unknown parse error".to_string(),
                ));
            let fallback_ctx = FallbackContext {
                ticket_id: String::new(),
                domain: ctx.domain.clone(),
                intent: "unknown".to_string(),
                question: ctx.question.clone(),
                probe_results: ctx.available_probes.clone(),
                reason,
                elapsed_ms: 0,
            };

            let fallback_response = generate_fallback(&fallback_ctx);
            let fallback_validation = validate_response(&fallback_response);
            let fallback_guardrails =
                check_guardrails(&fallback_response, ctx, &fallback_validation);

            (fallback_response, fallback_guardrails)
        }

        ParseOutcome::Timeout { elapsed_ms } => {
            let fallback_ctx = FallbackContext {
                ticket_id: String::new(),
                domain: ctx.domain.clone(),
                intent: "unknown".to_string(),
                question: ctx.question.clone(),
                probe_results: ctx.available_probes.clone(),
                reason: FallbackReason::Timeout,
                elapsed_ms,
            };

            let fallback_response = generate_fallback(&fallback_ctx);
            let fallback_validation = validate_response(&fallback_response);
            let fallback_guardrails =
                check_guardrails(&fallback_response, ctx, &fallback_validation);

            (fallback_response, fallback_guardrails)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_protocol::{ProbeEvidence, ResponseMeta};

    #[test]
    fn test_guardrail_pass() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed systemd services.",
            vec!["0 failed units".to_string()],
            vec![ProbeEvidence {
                id: "systemctl_failed".to_string(),
                summary: "0 failed units".to_string(),
                raw_reference: None,
            }],
            ResponseMeta {
                handled_by: "Test".to_string(),
                ticket_id: "T-1".to_string(),
                version: 1,
            },
        );

        let ctx =
            GuardrailContext::from_question("Do I have any failed services?", "services.systemd")
                .with_probe("systemctl_failed", "0 loaded units listed.");

        let validation = validate_response(&response);
        let result = check_guardrails(&response, &ctx, &validation);

        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_guardrail_fail_tutorial_for_state() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "Step 1: Run systemctl status. Step 2: Check the logs.",
            vec![],
            vec![],
            ResponseMeta {
                handled_by: "Test".to_string(),
                ticket_id: "T-1".to_string(),
                version: 1,
            },
        );

        let ctx =
            GuardrailContext::from_question("Do I have any failed services?", "services.systemd");
        let validation = validate_response(&response);
        let result = check_guardrails(&response, &ctx, &validation);

        // Should have intent mismatch violation
        assert!(!result.passed);
        assert!(result
            .violations
            .iter()
            .any(|v| matches!(v, GuardrailViolation::IntentMismatch { .. })));
    }

    #[test]
    fn test_vague_state_answer() {
        let response = StrictResponse::success(
            "system",
            "check_memory",
            "You might have enough memory.",
            vec![],
            vec![],
            ResponseMeta::default(),
        );

        assert!(is_vague_state_answer(&response));
    }
}
