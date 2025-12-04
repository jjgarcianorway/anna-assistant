//! Reliability Gate for Anna v0.0.81
//!
//! Enforces Junior reliability policing:
//! - Mutations require >= 90% reliability to proceed
//! - Blocks confirmation prompt if reliability too low
//! - Provides clear feedback on why blocked

use serde::{Deserialize, Serialize};

/// Minimum reliability score to allow mutation confirmation
pub const MIN_RELIABILITY_FOR_MUTATION: u8 = 90;

/// Minimum reliability score for read-only queries
pub const MIN_RELIABILITY_FOR_READ_ONLY: u8 = 70;

/// Minimum reliability score for diagnostic flows
pub const MIN_RELIABILITY_FOR_DIAGNOSIS: u8 = 80;

/// Reliability gate decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityGateResult {
    /// Whether the action can proceed
    pub can_proceed: bool,
    /// The reliability score
    pub reliability_score: u8,
    /// Minimum required score for this action type
    pub required_score: u8,
    /// Human-readable reason
    pub reason: String,
    /// Suggestions to improve reliability
    pub suggestions: Vec<String>,
}

/// Action type for reliability gating
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    /// Read-only system queries
    ReadOnly,
    /// Diagnostic flows (doctor)
    Diagnosis,
    /// System mutations (service/package/config changes)
    Mutation,
}

impl ActionType {
    /// Get the minimum reliability score for this action type
    pub fn min_reliability(&self) -> u8 {
        match self {
            ActionType::ReadOnly => MIN_RELIABILITY_FOR_READ_ONLY,
            ActionType::Diagnosis => MIN_RELIABILITY_FOR_DIAGNOSIS,
            ActionType::Mutation => MIN_RELIABILITY_FOR_MUTATION,
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ActionType::ReadOnly => "read-only query",
            ActionType::Diagnosis => "diagnostic flow",
            ActionType::Mutation => "system mutation",
        }
    }
}

/// Check if an action can proceed based on reliability score
pub fn check_reliability_gate(
    reliability_score: u8,
    action_type: ActionType,
    missing_evidence: &[String],
    critique: &str,
) -> ReliabilityGateResult {
    let required_score = action_type.min_reliability();
    let can_proceed = reliability_score >= required_score;

    let mut suggestions = Vec::new();

    if !can_proceed {
        // Build suggestions based on what's missing
        if !missing_evidence.is_empty() {
            suggestions.push(format!(
                "Gather missing evidence: {}",
                missing_evidence.join(", ")
            ));
        }

        if !critique.is_empty() {
            // Parse critique for actionable items
            for part in critique.split(';') {
                let part = part.trim();
                if part.contains("missing") || part.contains("need") {
                    suggestions.push(part.to_string());
                }
            }
        }

        // Generic suggestions by action type
        match action_type {
            ActionType::Mutation => {
                suggestions.push(
                    "For mutations, I need high confidence to avoid breaking your system"
                        .to_string(),
                );
                suggestions.push(
                    "Consider running a diagnostic first to gather more evidence".to_string(),
                );
            }
            ActionType::Diagnosis => {
                suggestions.push("Try running specific diagnostic tools for more evidence".to_string());
            }
            ActionType::ReadOnly => {
                suggestions.push("Try rephrasing the question with more specific terms".to_string());
            }
        }
    }

    let reason = if can_proceed {
        format!(
            "Reliability {}% meets minimum {}% for {}",
            reliability_score,
            required_score,
            action_type.description()
        )
    } else {
        format!(
            "Reliability {}% is below minimum {}% required for {}",
            reliability_score,
            required_score,
            action_type.description()
        )
    };

    ReliabilityGateResult {
        can_proceed,
        reliability_score,
        required_score,
        reason,
        suggestions,
    }
}

/// Format a blocked gate result for human display
pub fn format_gate_blocked(result: &ReliabilityGateResult) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "Cannot proceed: reliability {}% < {}% required",
        result.reliability_score, result.required_score
    ));

    if !result.suggestions.is_empty() {
        lines.push(String::new());
        lines.push("Suggestions:".to_string());
        for suggestion in &result.suggestions {
            lines.push(format!("  - {}", suggestion));
        }
    }

    lines.join("\n")
}

/// Format gate result for transcript output
pub fn format_gate_for_transcript(result: &ReliabilityGateResult, is_debug: bool) -> String {
    if result.can_proceed {
        if is_debug {
            format!(
                "[GATE] Passed: {}% >= {}%",
                result.reliability_score, result.required_score
            )
        } else {
            String::new() // Silent pass in human mode
        }
    } else if is_debug {
        format!(
            "[GATE] BLOCKED: {}% < {}% - {}",
            result.reliability_score, result.required_score, result.reason
        )
    } else {
        format!(
            "I can't proceed with this action yet. My confidence is {}%, but I need at least {}% to make changes safely.",
            result.reliability_score, result.required_score
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_gate_blocks_low_reliability() {
        let result = check_reliability_gate(85, ActionType::Mutation, &[], "");
        assert!(!result.can_proceed);
        assert_eq!(result.required_score, 90);
    }

    #[test]
    fn test_mutation_gate_passes_high_reliability() {
        let result = check_reliability_gate(95, ActionType::Mutation, &[], "");
        assert!(result.can_proceed);
    }

    #[test]
    fn test_mutation_gate_at_threshold() {
        let result = check_reliability_gate(90, ActionType::Mutation, &[], "");
        assert!(result.can_proceed);
    }

    #[test]
    fn test_read_only_lower_threshold() {
        let result = check_reliability_gate(75, ActionType::ReadOnly, &[], "");
        assert!(result.can_proceed);
    }

    #[test]
    fn test_suggestions_for_missing_evidence() {
        let missing = vec!["disk info".to_string(), "service status".to_string()];
        let result = check_reliability_gate(80, ActionType::Mutation, &missing, "");
        assert!(!result.can_proceed);
        assert!(!result.suggestions.is_empty());
        let suggestions_str = result.suggestions.join(" ");
        assert!(suggestions_str.contains("disk info"));
    }

    #[test]
    fn test_format_blocked_message() {
        let result = check_reliability_gate(80, ActionType::Mutation, &[], "");
        let formatted = format_gate_blocked(&result);
        assert!(formatted.contains("80%"));
        assert!(formatted.contains("90%"));
    }
}
