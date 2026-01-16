//! Total Response Formatter - Every request produces valid output.
//!
//! It is impossible to emit "could not format a valid response".
//! Every request produces exactly one of: Resolved, Abstained, Failed.

use super::registry::{CapabilityId, CapabilityMode, CAPABILITY_REGISTRY};
use super::router::CapabilityRoutingResult;
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
    },
    /// Structural error. Request could not be processed.
    Failed {
        error_code: String,
        error_message: String,
    },
}

/// Reason for abstaining from execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbstainReason {
    /// Capability is mutating and execution gate is closed.
    ExecutionGateBlocked,
    /// Request did not match any capability.
    UnknownCapability,
    /// Request was ambiguous.
    AmbiguousRequest,
    /// Request was malformed.
    MalformedRequest,
    /// Capability is disabled.
    CapabilityDisabled,
    /// ReadOnly capability gathered facts but has nothing actionable.
    NoActionRequired,
}

impl AbstainReason {
    /// Human-readable explanation of why we abstained.
    pub fn explanation(&self) -> &'static str {
        match self {
            Self::ExecutionGateBlocked => {
                "This operation would modify the system. Execution is currently blocked."
            }
            Self::UnknownCapability => {
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
        }
    }

    /// Code for logging/telemetry.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ExecutionGateBlocked => "EXECUTION_GATE_BLOCKED",
            Self::UnknownCapability => "UNKNOWN_CAPABILITY",
            Self::AmbiguousRequest => "AMBIGUOUS_REQUEST",
            Self::MalformedRequest => "MALFORMED_REQUEST",
            Self::CapabilityDisabled => "CAPABILITY_DISABLED",
            Self::NoActionRequired => "NO_ACTION_REQUIRED",
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
}

/// Execution result from a capability handler.
pub struct CapabilityExecutionResult {
    /// Facts gathered during execution.
    pub facts: Vec<ResponseArtifact>,
    /// Plan proposed (if any).
    pub plan: Option<ResponseArtifact>,
    /// Warnings discovered.
    pub warnings: Vec<ResponseArtifact>,
    /// Summary explanation.
    pub explanation: String,
}

impl CapabilityExecutionResult {
    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            facts: Vec::new(),
            plan: None,
            warnings: Vec::new(),
            explanation: String::new(),
        }
    }

    /// Collect all artifacts.
    pub fn artifacts(&self) -> Vec<ResponseArtifact> {
        let mut all = self.facts.clone();
        if let Some(plan) = &self.plan {
            all.push(plan.clone());
        }
        all.extend(self.warnings.clone());
        all
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
            // Map routing rejection to abstain reason
            let reason = match reason_code.as_str() {
                "UNKNOWN_CAPABILITY" => AbstainReason::UnknownCapability,
                "AMBIGUOUS_REQUEST" => AbstainReason::AmbiguousRequest,
                "MALFORMED_REQUEST" => AbstainReason::MalformedRequest,
                "CAPABILITY_DISABLED" => AbstainReason::CapabilityDisabled,
                _ => AbstainReason::UnknownCapability,
            };

            ResponseOutcome::Abstained {
                capability_id: None,
                reason,
                explanation: short_message.clone(),
            }
        }

        CapabilityRoutingResult::Supported { capability_id } => {
            // Look up capability in registry
            let capability = match CAPABILITY_REGISTRY.get(capability_id) {
                Some(cap) => cap,
                None => {
                    // Registry inconsistency - this is a structural error
                    return ResponseOutcome::Failed {
                        error_code: "REGISTRY_INCONSISTENCY".to_string(),
                        error_message: format!(
                            "Capability '{}' was routed but not found in registry.",
                            capability_id
                        ),
                    };
                }
            };

            // Check if capability can execute
            match capability.mode {
                CapabilityMode::Mutating => {
                    // Mutating capabilities are always blocked
                    ResponseOutcome::Abstained {
                        capability_id: Some(capability_id.clone()),
                        reason: AbstainReason::ExecutionGateBlocked,
                        explanation: format!(
                            "Capability '{}' requires system modification. {}",
                            capability_id,
                            AbstainReason::ExecutionGateBlocked.explanation()
                        ),
                    }
                }

                CapabilityMode::ReadOnly => {
                    // ReadOnly capabilities can execute
                    match execution {
                        Some(result) => {
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
                                error_code: "MISSING_EXECUTION_RESULT".to_string(),
                                error_message: format!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::router::route_request;

    #[test]
    fn test_unsupported_request_abstains() {
        let routing = route_request("tell me a joke");
        let response = format_response(&routing, None);

        match response {
            ResponseOutcome::Abstained { reason, .. } => {
                assert!(matches!(reason, AbstainReason::UnknownCapability));
            }
            _ => panic!("Expected Abstained"),
        }
    }

    #[test]
    fn test_mutating_capability_abstains() {
        let routing = route_request("install neovim");
        let response = format_response(&routing, None);

        match response {
            ResponseOutcome::Abstained { reason, capability_id, .. } => {
                assert!(matches!(reason, AbstainReason::ExecutionGateBlocked));
                assert!(capability_id.is_some());
            }
            _ => panic!("Expected Abstained"),
        }
    }

    #[test]
    fn test_readonly_with_result_resolves() {
        let routing = route_request("status");
        let execution = CapabilityExecutionResult {
            facts: vec![ResponseArtifact::fact("uptime", "3 days")],
            plan: None,
            warnings: Vec::new(),
            explanation: "System is healthy.".to_string(),
        };
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
            ResponseOutcome::Failed { error_code, .. } => {
                assert_eq!(error_code, "MISSING_EXECUTION_RESULT");
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
                Some(CapabilityExecutionResult {
                    facts: vec![ResponseArtifact::fact("test", "value")],
                    plan: None,
                    warnings: Vec::new(),
                    explanation: "Test.".to_string(),
                })
            } else {
                None
            };
            let response = format_response(&routing, execution);

            // Every response is one of the three variants
            match response {
                ResponseOutcome::Resolved { .. } => {}
                ResponseOutcome::Abstained { .. } => {}
                ResponseOutcome::Failed { .. } => {}
            }
        }
    }
}
