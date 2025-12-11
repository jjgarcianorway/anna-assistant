//! Translator guardrails for intent validation (v0.0.428).
//!
//! Enforces:
//! - Correct intent classification (state vs how-to)
//! - Strict response validation
//! - No auto-invention of facts when specialist fails

use super::{
    fallback::{FallbackContext, FallbackReason, generate_fallback},
    outcome::{TicketOutcome, determine_outcome},
    parser::{ParseOutcome, parse_specialist_response},
    schema::{ResponseStatus, StrictResponse},
    validator::{validate_response, is_useful_response, ValidationResult},
};

/// Intent type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentType {
    /// User is asking about current state ("do I have X?", "is X running?")
    CheckState,
    /// User wants to know how to do something ("how do I configure X?")
    HowTo,
    /// User wants explanation ("what does X mean?")
    Explain,
    /// User is reporting a problem ("X is not working")
    Diagnose,
    /// User wants to perform an action ("install X", "restart Y")
    Action,
    /// Unknown/ambiguous intent
    Unknown,
}

/// Classify intent from question text
pub fn classify_intent(question: &str) -> IntentType {
    let lower = question.to_lowercase();

    // State-checking patterns (highest priority)
    let state_patterns = [
        "do i have",
        "is there",
        "are there",
        "am i",
        "is my",
        "show me",
        "list my",
        "what is my",
        "how much",
        "how many",
        "is it running",
        "is it installed",
        "is it enabled",
        "is it active",
        "check if",
        "currently",
        "right now",
    ];

    for pattern in &state_patterns {
        if lower.contains(pattern) {
            return IntentType::CheckState;
        }
    }

    // How-to patterns
    let howto_patterns = [
        "how do i",
        "how can i",
        "how to",
        "how should i",
        "what's the best way to",
        "steps to",
        "guide to",
        "tutorial",
        "configure",
        "set up",
        "setup",
    ];

    for pattern in &howto_patterns {
        if lower.contains(pattern) {
            return IntentType::HowTo;
        }
    }

    // Explain patterns
    let explain_patterns = [
        "what does",
        "what is",
        "what are",
        "explain",
        "meaning of",
        "difference between",
        "why does",
    ];

    for pattern in &explain_patterns {
        if lower.contains(pattern) {
            return IntentType::Explain;
        }
    }

    // Diagnose patterns
    let diagnose_patterns = [
        "not working",
        "doesn't work",
        "won't start",
        "failing",
        "failed",
        "error",
        "problem with",
        "issue with",
        "trouble with",
        "broken",
        "crash",
    ];

    for pattern in &diagnose_patterns {
        if lower.contains(pattern) {
            return IntentType::Diagnose;
        }
    }

    // Action patterns
    let action_patterns = [
        "install",
        "remove",
        "delete",
        "restart",
        "stop",
        "start",
        "enable",
        "disable",
        "update",
        "upgrade",
    ];

    for pattern in &action_patterns {
        if lower.contains(pattern) {
            return IntentType::Action;
        }
    }

    IntentType::Unknown
}

/// Validation context for translator guardrails
#[derive(Debug)]
pub struct GuardrailContext {
    /// Original user question
    pub question: String,
    /// Classified intent type
    pub intent_type: IntentType,
    /// Domain hint
    pub domain: String,
    /// Available probes
    pub available_probes: std::collections::HashMap<String, String>,
}

impl GuardrailContext {
    /// Create context from question
    pub fn from_question(question: &str, domain: &str) -> Self {
        Self {
            question: question.to_string(),
            intent_type: classify_intent(question),
            domain: domain.to_string(),
            available_probes: std::collections::HashMap::new(),
        }
    }

    /// Add probe result
    pub fn with_probe(mut self, id: &str, output: &str) -> Self {
        self.available_probes.insert(id.to_string(), output.to_string());
        self
    }
}

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

/// Types of guardrail violations
#[derive(Debug, Clone)]
pub enum GuardrailViolation {
    /// Response type doesn't match intent (e.g., how-to for state query)
    IntentMismatch { expected: IntentType, got: ResponseType },
    /// Response contains invented facts not in evidence
    InventedFacts(Vec<String>),
    /// Response is too vague for the intent type
    TooVague,
    /// Response validation failed
    ValidationFailed(Vec<String>),
    /// Summary doesn't match evidence
    SummaryMismatch,
}

/// Response type classification (what kind of answer was given)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    /// Direct answer about current state
    StateAnswer,
    /// Tutorial/how-to guide
    Tutorial,
    /// Explanation
    Explanation,
    /// Diagnosis
    Diagnosis,
    /// Action confirmation
    ActionResult,
    /// Failure message
    Failure,
}

/// Classify what type of response was given
pub fn classify_response(response: &StrictResponse) -> ResponseType {
    let summary_lower = response.summary.to_lowercase();

    // Check for tutorial patterns
    let tutorial_patterns = [
        "step 1",
        "step 2",
        "first,",
        "to do this",
        "you can",
        "you should",
        "here's how",
        "follow these",
    ];

    if tutorial_patterns.iter().any(|p| summary_lower.contains(p)) {
        return ResponseType::Tutorial;
    }

    // Check for failure
    if response.status == ResponseStatus::Failure {
        return ResponseType::Failure;
    }

    // Check for state answer patterns
    let state_patterns = [
        "is installed",
        "is not installed",
        "is running",
        "is not running",
        "is enabled",
        "is disabled",
        "are no",
        "there are",
        "you have",
        "currently",
        "available",
        "active",
        "inactive",
    ];

    if state_patterns.iter().any(|p| summary_lower.contains(p)) {
        return ResponseType::StateAnswer;
    }

    // Check for diagnosis patterns
    let diagnosis_patterns = [
        "because",
        "the cause",
        "appears to be",
        "the issue is",
        "the problem is",
        "failed due to",
    ];

    if diagnosis_patterns.iter().any(|p| summary_lower.contains(p)) {
        return ResponseType::Diagnosis;
    }

    // Default to state answer if has facts
    if !response.details.key_facts.is_empty() {
        return ResponseType::StateAnswer;
    }

    ResponseType::Explanation
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
        let has_severe = violations.iter().any(|v| matches!(v,
            GuardrailViolation::InventedFacts(_)
            | GuardrailViolation::ValidationFailed(_)
        ));

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

/// Check if response type matches intent type
fn check_intent_match(intent: IntentType, response: ResponseType) -> Option<GuardrailViolation> {
    let mismatch = match intent {
        IntentType::CheckState => {
            // State query should get state answer, not tutorial
            response == ResponseType::Tutorial
        }
        IntentType::HowTo => {
            // How-to query shouldn't just get state answer
            // (Tutorial or Explanation are fine)
            false // More permissive
        }
        _ => false,
    };

    if mismatch {
        Some(GuardrailViolation::IntentMismatch {
            expected: intent,
            got: response,
        })
    } else {
        None
    }
}

/// Check for facts not backed by evidence
fn check_invented_facts(
    response: &StrictResponse,
    probes: &std::collections::HashMap<String, String>,
) -> Option<GuardrailViolation> {
    // Skip if no probes to check against
    if probes.is_empty() {
        return None;
    }

    // Extract numbers from response
    let response_numbers: Vec<String> = extract_numbers(&response.summary)
        .into_iter()
        .chain(response.details.key_facts.iter().flat_map(|f| extract_numbers(f)))
        .filter(|n| !is_common_number(n))
        .collect();

    // Extract numbers from probe outputs
    let evidence_text: String = probes.values().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
    let evidence_numbers: Vec<String> = extract_numbers(&evidence_text);

    // Find numbers in response not in evidence
    let invented: Vec<String> = response_numbers
        .into_iter()
        .filter(|n| !evidence_numbers.contains(n) && !evidence_text.contains(n))
        .collect();

    if invented.is_empty() {
        None
    } else {
        Some(GuardrailViolation::InventedFacts(invented))
    }
}

/// Extract numbers from text
fn extract_numbers(text: &str) -> Vec<String> {
    let mut numbers = vec![];
    let mut current = String::new();
    let mut in_number = false;

    for c in text.chars() {
        if c.is_ascii_digit() || (c == '.' && in_number) || (c == '%' && in_number) {
            current.push(c);
            in_number = true;
        } else {
            if in_number && !current.is_empty() {
                numbers.push(current.clone());
                current.clear();
            }
            in_number = false;
        }
    }
    if !current.is_empty() {
        numbers.push(current);
    }

    numbers
}

/// Check if a number is too common to flag
fn is_common_number(num: &str) -> bool {
    let common = ["0", "1", "2", "3", "4", "5", "10", "100", "100%", "0%"];
    common.contains(&num)
}

/// Check if state answer is too vague
fn is_vague_state_answer(response: &StrictResponse) -> bool {
    let lower = response.summary.to_lowercase();

    let vague_patterns = [
        "might",
        "could",
        "possibly",
        "perhaps",
        "typically",
        "usually",
        "generally",
        "you can try",
    ];

    vague_patterns.iter().any(|p| lower.contains(p))
}

/// Process a specialist response through all guardrails
pub fn process_with_guardrails(
    raw_output: &str,
    ctx: &GuardrailContext,
) -> (StrictResponse, GuardrailResult) {
    // Parse the response
    let parse_result = parse_specialist_response(raw_output);

    match parse_result {
        ParseOutcome::Success(response, validation) |
        ParseOutcome::ValidationFailed(response, validation) => {
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
                let fallback_guardrails = check_guardrails(&fallback_response, ctx, &fallback_validation);

                // Return fallback if it's better, otherwise original
                if is_useful_response(&fallback_response) && fallback_guardrails.passed {
                    (fallback_response, fallback_guardrails)
                } else {
                    (response, guardrail_result)
                }
            }
        }

        ParseOutcome::NoJson { .. } |
        ParseOutcome::InvalidJson { .. } |
        ParseOutcome::SchemaMismatch { .. } => {
            // Use fallback
            let reason = parse_result.to_fallback_reason().unwrap_or(FallbackReason::ParseError("Unknown parse error".to_string()));
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
            let fallback_guardrails = check_guardrails(&fallback_response, ctx, &fallback_validation);

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
            let fallback_guardrails = check_guardrails(&fallback_response, ctx, &fallback_validation);

            (fallback_response, fallback_guardrails)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_protocol::{ProbeEvidence, ResponseMeta};

    #[test]
    fn test_classify_intent_state() {
        // "do i have" triggers CheckState
        assert_eq!(classify_intent("do I have any failed services?"), IntentType::CheckState);
        // "how much" triggers CheckState
        assert_eq!(classify_intent("how much RAM do I have?"), IntentType::CheckState);
        // "is it running" needs the full phrase
        assert_eq!(classify_intent("Is nginx currently running?"), IntentType::CheckState);
    }

    #[test]
    fn test_classify_intent_howto() {
        assert_eq!(classify_intent("How do I configure nginx?"), IntentType::HowTo);
        assert_eq!(classify_intent("How to install vim?"), IntentType::HowTo);
    }

    #[test]
    fn test_classify_intent_diagnose() {
        assert_eq!(classify_intent("My wifi is not working"), IntentType::Diagnose);
        assert_eq!(classify_intent("nginx service failed"), IntentType::Diagnose);
    }

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

        let ctx = GuardrailContext::from_question("Do I have any failed services?", "services.systemd")
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

        let ctx = GuardrailContext::from_question("Do I have any failed services?", "services.systemd");
        let validation = validate_response(&response);
        let result = check_guardrails(&response, &ctx, &validation);

        // Should have intent mismatch violation
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| matches!(v, GuardrailViolation::IntentMismatch { .. })));
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

    #[test]
    fn test_response_type_classification() {
        let state_response = StrictResponse::success(
            "system", "check", "nginx is running.", vec![], vec![], ResponseMeta::default()
        );
        assert_eq!(classify_response(&state_response), ResponseType::StateAnswer);

        let tutorial_response = StrictResponse::success(
            "system", "howto", "Step 1: First, install the package.", vec![], vec![], ResponseMeta::default()
        );
        assert_eq!(classify_response(&tutorial_response), ResponseType::Tutorial);
    }
}
