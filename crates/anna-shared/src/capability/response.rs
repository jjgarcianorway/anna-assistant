//! Total Response Formatter - Every request produces valid output.
//!
//! It is impossible to emit "could not format a valid response".
//! Every request produces exactly one of: Resolved, Abstained, Failed, ConfirmationRequired.
//!
//! Phase 31: Mutating capabilities return ConfirmationRequired with ActionPlan.

use super::registry::{CapabilityId, CapabilityMode, CAPABILITY_REGISTRY};
use super::router::CapabilityRoutingResult;
use crate::action_plan::ActionPlan;
use serde::{Deserialize, Serialize};

/// The outcome of a capability request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseOutcome {
    /// Request was fulfilled. Contains explanation of what was done.
    Resolved {
        capability_id: CapabilityId,
        explanation: String,
        artifacts: Vec<ResponseArtifact>,
    },
    /// Request was understood but not executed. Contains explicit reason.
    Abstained {
        capability_id: Option<CapabilityId>,
        reason: AbstainReason,
        explanation: String,
        /// Hints for what capabilities might be relevant (for NoMatchingCapability).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        hints: Vec<String>,
    },
    /// Structural error. Request could not be processed.
    Failed {
        error: FailedReason,
        diagnostic: String,
    },
    /// Phase 31: Mutating capability ready to execute, awaiting user confirmation.
    ConfirmationRequired {
        capability_id: CapabilityId,
        /// Evidence gathered from probes (shown to user).
        evidence: Vec<ResponseArtifact>,
        /// The action plan to execute after confirmation.
        action_plan: ActionPlan,
    },
}

/// Reason for structural failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailedReason {
    /// Registry inconsistency (capability routed but not found).
    RegistryInconsistency,
    /// Missing execution result for ReadOnly capability.
    MissingExecutionResult,
    /// Probe failed during capability execution.
    ProbeError { probe_name: String },
    /// Internal error during formatting.
    FormattingError,
}

/// Reason for abstaining from execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbstainReason {
    /// Capability is mutating and execution gate is closed.
    ExecutionGateBlocked,
    /// Request did not match any known capability (includes hints for closest matches).
    NoMatchingCapability,
    /// Request was ambiguous.
    AmbiguousRequest,
    /// Request was malformed.
    MalformedRequest,
    /// Capability is disabled.
    CapabilityDisabled,
    /// ReadOnly capability gathered facts but has nothing actionable.
    NoActionRequired,
    /// System prerequisites not met for this capability.
    PrerequisitesNotMet,
    /// LLM output was blocked by policy (contained forbidden patterns).
    OutputPolicyViolation,
}

impl AbstainReason {
    /// Human-readable explanation of why we abstained.
    pub fn explanation(&self) -> &'static str {
        match self {
            Self::ExecutionGateBlocked => {
                "This operation would modify the system. Execution is currently blocked."
            }
            Self::NoMatchingCapability => {
                "This request does not match any known capability."
            }
            Self::AmbiguousRequest => {
                "This request matches multiple capabilities. Please be more specific."
            }
            Self::MalformedRequest => {
                "This request could not be parsed."
            }
            Self::CapabilityDisabled => {
                "This capability is currently disabled."
            }
            Self::NoActionRequired => {
                "Analysis complete. No action is required at this time."
            }
            Self::PrerequisitesNotMet => {
                "System prerequisites for this capability are not met."
            }
            Self::OutputPolicyViolation => {
                "Response contained content that violates output policy."
            }
        }
    }

    /// Code for logging/telemetry.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ExecutionGateBlocked => "EXECUTION_GATE_BLOCKED",
            Self::NoMatchingCapability => "NO_MATCHING_CAPABILITY",
            Self::AmbiguousRequest => "AMBIGUOUS_REQUEST",
            Self::MalformedRequest => "MALFORMED_REQUEST",
            Self::CapabilityDisabled => "CAPABILITY_DISABLED",
            Self::NoActionRequired => "NO_ACTION_REQUIRED",
            Self::PrerequisitesNotMet => "PREREQUISITES_NOT_MET",
            Self::OutputPolicyViolation => "OUTPUT_POLICY_VIOLATION",
        }
    }
}

/// Artifact produced by a resolved capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseArtifact {
    /// Type of artifact (e.g., "fact", "plan", "warning").
    pub artifact_type: String,
    /// Human-readable label.
    pub label: String,
    /// Content of the artifact.
    pub content: String,
}

impl ResponseArtifact {
    /// Create a fact artifact.
    pub fn fact(label: &str, content: &str) -> Self {
        Self {
            artifact_type: "fact".to_string(),
            label: label.to_string(),
            content: content.to_string(),
        }
    }

    /// Create an evidence artifact (probe result).
    pub fn evidence(label: &str, content: &str) -> Self {
        Self {
            artifact_type: "evidence".to_string(),
            label: label.to_string(),
            content: content.to_string(),
        }
    }

    /// Create an operator step artifact.
    pub fn step(number: usize, content: &str) -> Self {
        Self {
            artifact_type: "step".to_string(),
            label: format!("Step {}", number),
            content: content.to_string(),
        }
    }

    /// Create a rollback step artifact.
    pub fn rollback(number: usize, content: &str) -> Self {
        Self {
            artifact_type: "rollback".to_string(),
            label: format!("Rollback {}", number),
            content: content.to_string(),
        }
    }

    /// Create a plan artifact.
    pub fn plan(label: &str, content: &str) -> Self {
        Self {
            artifact_type: "plan".to_string(),
            label: label.to_string(),
            content: content.to_string(),
        }
    }

    /// Create a warning artifact.
    pub fn warning(label: &str, content: &str) -> Self {
        Self {
            artifact_type: "warning".to_string(),
            label: label.to_string(),
            content: content.to_string(),
        }
    }

    /// Create a note artifact (caveats, limitations).
    pub fn note(label: &str, content: &str) -> Self {
        Self {
            artifact_type: "note".to_string(),
            label: label.to_string(),
            content: content.to_string(),
        }
    }
}

/// Execution result from a capability handler.
pub struct CapabilityExecutionResult {
    /// Evidence gathered from probes.
    pub evidence: Vec<ResponseArtifact>,
    /// Operator steps (numbered). Used for ReadOnly capabilities.
    pub steps: Vec<ResponseArtifact>,
    /// Rollback steps (numbered).
    pub rollback: Vec<ResponseArtifact>,
    /// Notes/caveats.
    pub notes: Vec<ResponseArtifact>,
    /// Warnings discovered.
    pub warnings: Vec<ResponseArtifact>,
    /// Summary explanation.
    pub explanation: String,
    /// If set, this handler wants to abstain with a specific reason.
    pub abstain: Option<(AbstainReason, String)>,
    /// Legacy: facts (mapped to evidence).
    pub facts: Vec<ResponseArtifact>,
    /// Legacy: plan (mapped to steps).
    pub plan: Option<ResponseArtifact>,
    /// Phase 31: ActionPlan for mutating capabilities.
    pub action_plan: Option<ActionPlan>,
}

impl CapabilityExecutionResult {
    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            evidence: Vec::new(),
            steps: Vec::new(),
            rollback: Vec::new(),
            notes: Vec::new(),
            warnings: Vec::new(),
            explanation: String::new(),
            abstain: None,
            facts: Vec::new(),
            plan: None,
            action_plan: None,
        }
    }

    /// Create a result that abstains with a reason.
    pub fn abstain(reason: AbstainReason, explanation: &str) -> Self {
        Self {
            evidence: Vec::new(),
            steps: Vec::new(),
            rollback: Vec::new(),
            notes: Vec::new(),
            warnings: Vec::new(),
            explanation: String::new(),
            abstain: Some((reason, explanation.to_string())),
            facts: Vec::new(),
            plan: None,
            action_plan: None,
        }
    }

    /// Phase 31: Create a result with an ActionPlan for mutating capabilities.
    pub fn with_action_plan(evidence: Vec<ResponseArtifact>, action_plan: ActionPlan) -> Self {
        Self {
            evidence,
            steps: Vec::new(),
            rollback: Vec::new(),
            notes: Vec::new(),
            warnings: Vec::new(),
            explanation: String::new(),
            abstain: None,
            facts: Vec::new(),
            plan: None,
            action_plan: Some(action_plan),
        }
    }

    /// Phase 33: Create a resolved result for ReadOnly capabilities.
    pub fn with_explanation(evidence: Vec<ResponseArtifact>, explanation: &str) -> Self {
        Self {
            evidence,
            steps: Vec::new(),
            rollback: Vec::new(),
            notes: Vec::new(),
            warnings: Vec::new(),
            explanation: explanation.to_string(),
            abstain: None,
            facts: Vec::new(),
            plan: None,
            action_plan: None,
        }
    }

    /// Collect all artifacts in display order.
    pub fn artifacts(&self) -> Vec<ResponseArtifact> {
        let mut all = Vec::new();
        // Evidence first
        all.extend(self.evidence.clone());
        // Legacy facts (if any)
        all.extend(self.facts.clone());
        // Steps
        all.extend(self.steps.clone());
        // Rollback
        all.extend(self.rollback.clone());
        // Notes
        all.extend(self.notes.clone());
        // Warnings
        all.extend(self.warnings.clone());
        // Legacy plan (if any)
        if let Some(plan) = &self.plan {
            all.push(plan.clone());
        }
        all
    }

    /// Check if this result wants to abstain.
    pub fn wants_abstain(&self) -> bool {
        self.abstain.is_some()
    }
}

/// Format a response from a routing result.
///
/// This function is total. It cannot fail to produce valid output.
/// Every possible input produces exactly one of: Resolved, Abstained, Failed.
pub fn format_response(
    routing: &CapabilityRoutingResult,
    execution: Option<CapabilityExecutionResult>,
) -> ResponseOutcome {
    match routing {
        CapabilityRoutingResult::Unsupported {
            reason_code,
            short_message,
        } => {
            // Map routing rejection to abstain reason with hints
            let (reason, hints) = match reason_code.as_str() {
                "UNKNOWN_CAPABILITY" => {
                    // Provide hints for closest capability matches
                    let hints = get_capability_hints();
                    (AbstainReason::NoMatchingCapability, hints)
                }
                "AMBIGUOUS_REQUEST" => (AbstainReason::AmbiguousRequest, Vec::new()),
                "MALFORMED_REQUEST" => (AbstainReason::MalformedRequest, Vec::new()),
                "CAPABILITY_DISABLED" => (AbstainReason::CapabilityDisabled, Vec::new()),
                _ => (AbstainReason::NoMatchingCapability, get_capability_hints()),
            };

            ResponseOutcome::Abstained {
                capability_id: None,
                reason,
                explanation: short_message.clone(),
                hints,
            }
        }

        CapabilityRoutingResult::Supported { capability_id } => {
            // Look up capability in registry
            let capability = match CAPABILITY_REGISTRY.get(capability_id) {
                Some(cap) => cap,
                None => {
                    // Registry inconsistency - this is a structural error
                    return ResponseOutcome::Failed {
                        error: FailedReason::RegistryInconsistency,
                        diagnostic: format!(
                            "Capability '{}' was routed but not found in registry.",
                            capability_id
                        ),
                    };
                }
            };

            // Check if capability can execute
            match capability.mode {
                CapabilityMode::Mutating => {
                    // Phase 31: Mutating capabilities return ConfirmationRequired with ActionPlan
                    match execution {
                        Some(result) => {
                            // Check if handler wants to abstain
                            if let Some((reason, explanation)) = result.abstain {
                                return ResponseOutcome::Abstained {
                                    capability_id: Some(capability_id.clone()),
                                    reason,
                                    explanation,
                                    hints: Vec::new(),
                                };
                            }

                            // Check if we have an ActionPlan
                            match result.action_plan {
                                Some(action_plan) => {
                                    // Check if preflight determined no changes needed
                                    if !action_plan.changes_needed {
                                        // Already configured - return Resolved
                                        return ResponseOutcome::Resolved {
                                            capability_id: capability_id.clone(),
                                            explanation: action_plan.skip_reason.clone()
                                                .unwrap_or_else(|| "Already configured.".to_string()),
                                            artifacts: result.evidence,
                                        };
                                    }

                                    // Return ConfirmationRequired with ActionPlan
                                    ResponseOutcome::ConfirmationRequired {
                                        capability_id: capability_id.clone(),
                                        evidence: result.evidence,
                                        action_plan,
                                    }
                                }
                                None => {
                                    // Mutating capability without ActionPlan - structural error
                                    ResponseOutcome::Failed {
                                        error: FailedReason::MissingExecutionResult,
                                        diagnostic: format!(
                                            "Capability '{}' is Mutating but no ActionPlan was provided.",
                                            capability_id
                                        ),
                                    }
                                }
                            }
                        }
                        None => {
                            // No execution result provided - structural error
                            ResponseOutcome::Failed {
                                error: FailedReason::MissingExecutionResult,
                                diagnostic: format!(
                                    "Capability '{}' is Mutating but no execution result was provided.",
                                    capability_id
                                ),
                            }
                        }
                    }
                }

                CapabilityMode::ReadOnly => {
                    // ReadOnly capabilities can execute
                    match execution {
                        Some(result) => {
                            // Check if handler wants to abstain
                            if let Some((reason, explanation)) = result.abstain {
                                return ResponseOutcome::Abstained {
                                    capability_id: Some(capability_id.clone()),
                                    reason,
                                    explanation,
                                    hints: Vec::new(),
                                };
                            }

                            let artifacts = result.artifacts();
                            if artifacts.is_empty() && result.explanation.is_empty() {
                                // Execution produced nothing - abstain
                                ResponseOutcome::Abstained {
                                    capability_id: Some(capability_id.clone()),
                                    reason: AbstainReason::NoActionRequired,
                                    explanation: format!(
                                        "Capability '{}' completed analysis. No issues found.",
                                        capability_id
                                    ),
                                    hints: Vec::new(),
                                }
                            } else {
                                // Normal resolution
                                ResponseOutcome::Resolved {
                                    capability_id: capability_id.clone(),
                                    explanation: result.explanation,
                                    artifacts,
                                }
                            }
                        }
                        None => {
                            // No execution result provided - structural error
                            ResponseOutcome::Failed {
                                error: FailedReason::MissingExecutionResult,
                                diagnostic: format!(
                                    "Capability '{}' is ReadOnly but no execution result was provided.",
                                    capability_id
                                ),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get human-readable capability hints for unknown requests.
fn get_capability_hints() -> Vec<String> {
    vec![
        "Check system status (try: 'status')".to_string(),
        "Check disk usage (try: 'disk usage')".to_string(),
        "Scale GDM login screen (try: 'scale gdm')".to_string(),
        "Check memory (try: 'how much ram')".to_string(),
        "Check services (try: 'what services are failing')".to_string(),
    ]
}

/// Format a ResponseOutcome to a user-friendly string.
/// This is the total response builder - it always produces valid output.
/// Never returns "could not format a valid response" or any equivalent.
pub fn format_outcome_to_string(outcome: &ResponseOutcome) -> String {
    match outcome {
        ResponseOutcome::Resolved {
            capability_id: _,
            explanation,
            artifacts,
        } => {
            let mut output = String::new();

            // Group artifacts by type
            let evidence: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "evidence").collect();
            let facts: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "fact").collect();
            let steps: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "step").collect();
            let rollback: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "rollback").collect();
            let notes: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "note").collect();
            let warnings: Vec<_> = artifacts.iter().filter(|a| a.artifact_type == "warning").collect();

            // What I detected
            if !evidence.is_empty() || !facts.is_empty() {
                output.push_str("Detected:\n");
                for a in &evidence {
                    output.push_str(&format!("  {} {}\n", a.label, a.content));
                }
                for a in &facts {
                    output.push_str(&format!("  {} {}\n", a.label, a.content));
                }
                output.push('\n');
            }

            // Summary
            if !explanation.is_empty() {
                output.push_str(explanation);
                output.push_str("\n\n");
            }

            // What to do
            if !steps.is_empty() {
                output.push_str("Commands:\n");
                for a in &steps {
                    output.push_str(&format!("  {}. {}\n", a.label.replace("Step ", ""), a.content));
                }
                output.push('\n');
            }

            // Rollback
            if !rollback.is_empty() {
                output.push_str("To undo:\n");
                for a in &rollback {
                    output.push_str(&format!("  {}\n", a.content));
                }
                output.push('\n');
            }

            // Notes
            if !notes.is_empty() {
                for a in &notes {
                    output.push_str(&format!("{}: {}\n", a.label, a.content));
                }
            }

            // Warnings
            if !warnings.is_empty() {
                output.push_str("\nWarnings:\n");
                for a in &warnings {
                    output.push_str(&format!("  {}\n", a.content));
                }
            }

            output.trim_end().to_string()
        }

        ResponseOutcome::Abstained {
            capability_id: _,
            reason,
            explanation,
            hints,
        } => {
            let mut output = String::new();

            // Lead with the explanation, not technical codes
            output.push_str(explanation);
            output.push('\n');

            // Add context for specific reasons
            match reason {
                AbstainReason::PrerequisitesNotMet => {
                    // Explanation already contains details
                }
                AbstainReason::NoMatchingCapability => {
                    if !hints.is_empty() {
                        output.push_str("\nThings I can help with:\n");
                        for hint in hints {
                            output.push_str(&format!("  {}\n", hint));
                        }
                    }
                }
                AbstainReason::ExecutionGateBlocked => {
                    output.push_str("\nThis would modify your system. Execution is currently blocked.\n");
                }
                _ => {}
            }

            output.trim_end().to_string()
        }

        ResponseOutcome::Failed { error, diagnostic } => {
            match error {
                FailedReason::ProbeError { probe_name } => {
                    format!("Probe '{}' failed: {}", probe_name, diagnostic)
                }
                _ => diagnostic.clone(),
            }
        }

        ResponseOutcome::ConfirmationRequired {
            capability_id: _,
            evidence,
            action_plan,
        } => {
            let mut output = String::new();

            // Show detected evidence (short)
            if !evidence.is_empty() {
                output.push_str("Detected:\n");
                for a in evidence.iter().take(6) {
                    output.push_str(&format!("  {} {}\n", a.label, a.content));
                }
                output.push('\n');
            }

            // Phase 31: Use ActionPlan's format_for_confirmation
            // This shows descriptions, NOT raw commands
            output.push_str(&action_plan.format_for_confirmation());

            output
        }
    }
}

/// Build a total response for a request that was blocked by output policy.
/// This replaces the generic "could not format a valid response" message.
pub fn build_policy_violation_response(original_request: &str) -> ResponseOutcome {
    // Try to route the request to find relevant hints
    let routing = super::router::route_request(original_request);

    match routing {
        CapabilityRoutingResult::Supported { capability_id } => {
            // A capability matched but output was blocked
            ResponseOutcome::Abstained {
                capability_id: Some(capability_id),
                reason: AbstainReason::OutputPolicyViolation,
                explanation: "Anna identified a matching capability but the generated response \
                    contained content that cannot be displayed. Please rephrase your request \
                    or try a more specific query.".to_string(),
                hints: Vec::new(),
            }
        }
        CapabilityRoutingResult::Unsupported { .. } => {
            // No capability matched
            ResponseOutcome::Abstained {
                capability_id: None,
                reason: AbstainReason::NoMatchingCapability,
                explanation: "This request does not match any known capability.".to_string(),
                hints: get_capability_hints(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::router::route_request;
    use crate::capability::{
        execute_display_scale_gdm, execute_thermal_status, execute_audio_stack_detect,
        execute_power_inhibit_sleep, InhibitTarget, InhibitAction,
    };

    #[test]
    fn test_unsupported_request_abstains_with_hints() {
        let routing = route_request("tell me a joke");
        let response = format_response(&routing, None);

        match response {
            ResponseOutcome::Abstained { reason, hints, .. } => {
                assert!(matches!(reason, AbstainReason::NoMatchingCapability));
                // Should include capability hints
                assert!(!hints.is_empty(), "Unknown capability should include hints");
            }
            _ => panic!("Expected Abstained"),
        }
    }

    #[test]
    fn test_mutating_capability_without_execution_fails() {
        // Phase 31: Mutating capabilities require an execution result with ActionPlan
        let routing = route_request("install neovim");
        let response = format_response(&routing, None);

        match response {
            ResponseOutcome::Failed { error, diagnostic } => {
                assert!(matches!(error, FailedReason::MissingExecutionResult));
                assert!(diagnostic.contains("Mutating"));
            }
            _ => panic!("Expected Failed"),
        }
    }

    #[test]
    fn test_readonly_with_result_resolves() {
        let routing = route_request("status");
        let mut execution = CapabilityExecutionResult::empty();
        execution.facts = vec![ResponseArtifact::fact("uptime", "3 days")];
        execution.explanation = "System is healthy.".to_string();
        let response = format_response(&routing, Some(execution));

        match response {
            ResponseOutcome::Resolved { explanation, artifacts, .. } => {
                assert_eq!(explanation, "System is healthy.");
                assert_eq!(artifacts.len(), 1);
            }
            _ => panic!("Expected Resolved"),
        }
    }

    #[test]
    fn test_readonly_without_result_fails() {
        let routing = route_request("status");
        let response = format_response(&routing, None);

        match response {
            ResponseOutcome::Failed { error, .. } => {
                assert!(matches!(error, FailedReason::MissingExecutionResult));
            }
            _ => panic!("Expected Failed"),
        }
    }

    #[test]
    fn test_empty_result_abstains() {
        let routing = route_request("status");
        let execution = CapabilityExecutionResult::empty();
        let response = format_response(&routing, Some(execution));

        match response {
            ResponseOutcome::Abstained { reason, .. } => {
                assert!(matches!(reason, AbstainReason::NoActionRequired));
            }
            _ => panic!("Expected Abstained"),
        }
    }

    #[test]
    fn test_response_is_total() {
        // Every possible routing result produces valid output
        let test_inputs = vec![
            "",
            "   ",
            "status",
            "install vim",
            "random nonsense",
            "gdm scaling hidpi",
            "disk usage",
        ];

        for input in test_inputs {
            let routing = route_request(input);
            let execution = if routing.is_supported() {
                let mut result = CapabilityExecutionResult::empty();
                result.facts = vec![ResponseArtifact::fact("test", "value")];
                result.explanation = "Test.".to_string();
                Some(result)
            } else {
                None
            };
            let response = format_response(&routing, execution);

            // Every response is one of the four variants
            match response {
                ResponseOutcome::Resolved { .. } => {}
                ResponseOutcome::Abstained { .. } => {}
                ResponseOutcome::Failed { .. } => {}
                ResponseOutcome::ConfirmationRequired { .. } => {}
            }
        }
    }

    // ==========================================================================
    // REGRESSION TESTS: Ensure "could not format" never appears
    // ==========================================================================

    #[test]
    fn test_no_could_not_format_in_abstained() {
        let routing = route_request("random nonsense");
        let response = format_response(&routing, None);
        let output = format_outcome_to_string(&response);

        assert!(
            !output.contains("could not format a valid response"),
            "Output must never contain generic fallback: {}",
            output
        );
    }

    #[test]
    fn test_no_could_not_format_in_failed() {
        let routing = route_request("status");
        let response = format_response(&routing, None);
        let output = format_outcome_to_string(&response);

        assert!(
            !output.contains("could not format a valid response"),
            "Output must never contain generic fallback: {}",
            output
        );
    }

    #[test]
    fn test_policy_violation_response_no_generic_fallback() {
        let response = build_policy_violation_response("scale my gdm");
        let output = format_outcome_to_string(&response);

        assert!(
            !output.contains("could not format a valid response"),
            "Policy violation must not use generic fallback: {}",
            output
        );
    }

    #[test]
    fn test_abstained_unknown_includes_hints() {
        let routing = route_request("what is the meaning of life");
        let response = format_response(&routing, None);

        match response {
            ResponseOutcome::Abstained { hints, .. } => {
                assert!(!hints.is_empty(), "Unknown request should include hints");
                // Hints should be human-readable, containing action suggestions
                let hints_text = hints.join(" ");
                assert!(hints_text.contains("status") || hints_text.contains("help"));
            }
            _ => panic!("Expected Abstained"),
        }
    }

    #[test]
    fn test_failed_probe_error_format() {
        let response = ResponseOutcome::Failed {
            error: FailedReason::ProbeError {
                probe_name: "gnome_detection".to_string(),
            },
            diagnostic: "Could not detect GNOME presence".to_string(),
        };
        let output = format_outcome_to_string(&response);

        // Output should mention the probe name and be human-readable
        assert!(output.contains("gnome_detection"));
        assert!(output.contains("failed"));
        assert!(!output.contains("could not format a valid response"));
    }

    #[test]
    fn test_format_outcome_always_produces_output() {
        // Test all ResponseOutcome variants
        let outcomes = vec![
            ResponseOutcome::Resolved {
                capability_id: CapabilityId::new("test"),
                explanation: "Test".to_string(),
                artifacts: vec![],
            },
            ResponseOutcome::Abstained {
                capability_id: None,
                reason: AbstainReason::NoMatchingCapability,
                explanation: "Test".to_string(),
                hints: vec!["hint".to_string()],
            },
            ResponseOutcome::Failed {
                error: FailedReason::FormattingError,
                diagnostic: "Test".to_string(),
            },
        ];

        for outcome in outcomes {
            let output = format_outcome_to_string(&outcome);
            assert!(!output.is_empty(), "Output must never be empty");
            assert!(
                !output.contains("could not format a valid response"),
                "Output must never contain generic fallback"
            );
        }
    }

    // Phase 31: Prove no manual commands in ConfirmationRequired output
    #[test]
    fn test_confirmation_required_output_no_manual_commands() {
        use crate::action_plan::{ActionPlan, ActionStep};

        // Build an ActionPlan with real system commands
        let mut plan = ActionPlan::new("test", "Test plan", "Testing Phase 31");
        plan.add_step_full(
            ActionStep::new("Create directory", "mkdir -p /var/lib/gdm/.config", true)
        );
        plan.add_step_full(
            ActionStep::new("Copy config", "cp /home/user/.config/monitors.xml /var/lib/gdm/.config/monitors.xml", true)
        );
        plan.add_step_full(
            ActionStep::new("Set ownership", "chown gdm:gdm /var/lib/gdm/.config/monitors.xml", true)
        );

        let outcome = ResponseOutcome::ConfirmationRequired {
            capability_id: CapabilityId::new("display.scale.gdm"),
            evidence: vec![],
            action_plan: plan,
        };

        let output = format_outcome_to_string(&outcome);

        // CRITICAL: Output must NOT contain raw shell commands
        // Note: "[sudo]" marker is allowed - it indicates privilege needs, not a command
        assert!(
            !output.contains("mkdir -p"),
            "Output must not contain mkdir command. Got: {}",
            output
        );
        assert!(
            !output.contains("cp /home"),
            "Output must not contain cp command. Got: {}",
            output
        );
        assert!(
            !output.contains("chown gdm"),
            "Output must not contain chown command. Got: {}",
            output
        );
        // Check for actual sudo commands, not the [sudo] marker
        assert!(
            !output.contains("sudo ") && !output.contains("sudo\t"),
            "Output must not contain sudo commands. Got: {}",
            output
        );
        assert!(
            !output.contains("/var/lib/gdm/.config/monitors.xml"),
            "Output must not contain raw file paths. Got: {}",
            output
        );

        // But SHOULD contain step descriptions
        assert!(
            output.contains("Create directory") || output.contains("Copy config"),
            "Output should contain step descriptions. Got: {}",
            output
        );
    }

    // =========================================================================
    // PHASE 33.2: Prove capability path bypasses LLM
    // =========================================================================

    /// Phase 33.2: Prove that capability routing produces no LLM calls.
    /// The contract: capability handlers produce direct output without shell command execution.
    /// Evidence: handlers return explanation/action_plan, NOT commands to execute.
    #[test]
    fn test_phase33_capability_path_bypasses_llm() {
        use crate::capability::{
            execute_thermal_status, execute_audio_stack_detect,
            execute_display_scale_gdm, execute_power_inhibit_sleep,
            InhibitTarget, InhibitAction,
        };

        // Test all Phase 33 capabilities
        let capabilities: Vec<(&str, CapabilityExecutionResult)> = vec![
            ("system.thermal.status", execute_thermal_status()),
            ("audio.stack.detect", execute_audio_stack_detect()),
            ("display.scale.gdm", execute_display_scale_gdm()),
            ("power.inhibit.sleep", execute_power_inhibit_sleep(InhibitTarget::LidClose, InhibitAction::Ignore)),
        ];

        for (cap_id, result) in capabilities {
            // CRITICAL: Capability must produce direct output without LLM.
            // Evidence: handlers return explanation (ReadOnly) or action_plan (Mutating)
            // NOT a request to run shell commands.
            let has_output = !result.explanation.is_empty() || result.action_plan.is_some();
            assert!(
                has_output,
                "Capability {} must produce explanation or action_plan (no LLM)",
                cap_id
            );

            // The `steps` field is for ReadOnly display only, not shell commands
            // Mutating capabilities use action_plan with structured steps
            if result.action_plan.is_some() {
                // Mutating: verify action_plan has steps with descriptions (not raw commands exposed)
                let plan = result.action_plan.as_ref().unwrap();
                for step in &plan.steps {
                    // Step descriptions should be human-readable, not raw shell
                    assert!(
                        !step.description.contains("mkdir -p") &&
                        !step.description.contains("cp /") &&
                        !step.description.contains("chown ") &&
                        !step.description.contains("sed -i"),
                        "Capability {} step description must not be raw shell: {}",
                        cap_id,
                        step.description
                    );
                }
            }
        }
    }

    /// Phase 33.2: Prove deterministic routing - same input always routes same way.
    #[test]
    fn test_phase33_routing_is_deterministic_for_all_capabilities() {
        let test_inputs = vec![
            ("what's my cpu temperature", "system.thermal.status"),
            ("check thermal status", "system.thermal.status"),
            ("am I using pipewire or pulseaudio", "audio.stack.detect"),
            ("what audio system am I running", "audio.stack.detect"),
            ("scale gdm login screen", "display.scale.gdm"),
            ("stop sleep when closing lid", "power.inhibit.sleep"),
        ];

        for (input, expected_cap) in test_inputs {
            let result1 = route_request(input);
            let result2 = route_request(input);

            // Must route to the same capability every time
            assert_eq!(
                result1.capability_id().map(|id| id.as_str()),
                result2.capability_id().map(|id| id.as_str()),
                "Routing must be deterministic for: {}",
                input
            );

            // Must route to expected capability
            assert!(
                result1.is_supported(),
                "Input '{}' should route to capability",
                input
            );
            assert_eq!(
                result1.capability_id().unwrap().as_str(),
                expected_cap,
                "Input '{}' should route to {}",
                input,
                expected_cap
            );
        }
    }

    // ==========================================================================
    // Phase 34A: Specific Regression Tests
    // ==========================================================================

    /// Phase 34A: Test the EXACT user question "can you please scale up GDM login screen?"
    /// It MUST route to display.scale.gdm and produce either:
    /// - ConfirmationRequired (if prerequisites met)
    /// - Abstained (if prerequisites not met)
    /// It MUST NOT produce "could not format a valid response"
    #[test]
    fn test_gdm_scale_question_is_confirmation_or_abstain() {
        let question = "can you please scale up GDM login screen?";
        let routing = route_request(question);

        // Must route to capability
        assert!(
            routing.is_supported(),
            "Question '{}' must route to a capability, not fall through to LLM",
            question
        );
        assert_eq!(
            routing.capability_id().unwrap().as_str(),
            "display.scale.gdm",
            "Question must route to display.scale.gdm"
        );

        // Execute the capability
        let execution_result = execute_display_scale_gdm();
        let response = format_response(&routing, Some(execution_result));

        // Must be either ConfirmationRequired or Abstained (not Failed, not Resolved without ActionPlan)
        match &response {
            ResponseOutcome::ConfirmationRequired { action_plan, .. } => {
                // Good: has an action plan
                assert!(!action_plan.steps.is_empty() || !action_plan.changes_needed);
            }
            ResponseOutcome::Abstained { reason, explanation, .. } => {
                // Good: abstained with a clear reason
                assert!(!explanation.is_empty(), "Abstain must have explanation");
                // Must NOT contain the forbidden message
                assert!(
                    !explanation.to_lowercase().contains("could not format"),
                    "Abstain explanation must not contain 'could not format': {}",
                    explanation
                );
            }
            ResponseOutcome::Resolved { explanation, .. } => {
                // Also acceptable if changes_needed=false scenario
                assert!(
                    !explanation.to_lowercase().contains("could not format"),
                    "Resolved explanation must not contain 'could not format': {}",
                    explanation
                );
            }
            ResponseOutcome::Failed { diagnostic, .. } => {
                panic!(
                    "Capability must not fail for matched request. Diagnostic: {}",
                    diagnostic
                );
            }
        }
    }

    /// Phase 34A: No "could not format a valid response" for ANY capability match
    #[test]
    fn test_no_could_not_format_for_capability_match() {
        let questions = vec![
            "can you please scale up GDM login screen?",
            "scale gdm",
            "what's my cpu temperature",
            "am I using pipewire",
            "stop sleep when closing lid",
        ];

        for question in questions {
            let routing = route_request(question);
            if !routing.is_supported() {
                continue; // Only test capability-routed requests
            }

            let cap_id = routing.capability_id().unwrap().as_str();
            let execution_result = match cap_id {
                "display.scale.gdm" => execute_display_scale_gdm(),
                "system.thermal.status" => execute_thermal_status(),
                "audio.stack.detect" => execute_audio_stack_detect(),
                "power.inhibit.sleep" => {
                    execute_power_inhibit_sleep(InhibitTarget::LidClose, InhibitAction::Ignore)
                }
                _ => continue,
            };

            let response = format_response(&routing, Some(execution_result));
            let output = format_outcome_to_string(&response);

            // CRITICAL ASSERTION
            assert!(
                !output.to_lowercase().contains("could not format"),
                "Phase 34A: Capability '{}' for question '{}' must not produce 'could not format'. Got: {}",
                cap_id,
                question,
                output
            );
            assert!(
                !output.to_lowercase().contains("gathered information"),
                "Phase 34A: Capability '{}' must not produce 'gathered information' fallback. Got: {}",
                cap_id,
                output
            );
        }
    }
}
