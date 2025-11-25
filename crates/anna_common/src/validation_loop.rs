//! Validation Loop - Multi-round LLM validation
//!
//! v6.45.0: Allows LLM to request retries when initial results are insufficient.
//! The LLM can refine its approach based on what it learned from previous attempts.

use crate::executor_core::ExecutionResult;
use crate::interpreter_core::InterpretedAnswer;
use crate::planner_core::{CommandPlan, Intent};
use serde::{Deserialize, Serialize};

/// Maximum number of validation rounds to prevent infinite loops
pub const MAX_VALIDATION_ROUNDS: usize = 3;

/// Validation decision from the LLM
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationDecision {
    /// Results are sufficient, proceed with interpretation
    Sufficient,

    /// Results are insufficient, need another round
    NeedMoreData {
        /// What was wrong with the current results
        reason: String,
        /// Suggested refinement for next attempt
        suggested_approach: String,
    },

    /// Results are ambiguous, need clarification
    Ambiguous {
        /// What is ambiguous
        reason: String,
        /// Alternative approaches to try
        alternatives: Vec<String>,
    },
}

/// Request to validate execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    /// Original user intent
    pub intent: Intent,

    /// Execution result from this round
    pub execution_result: ExecutionResult,

    /// Current round number (1-indexed)
    pub round: usize,

    /// Previous validation decisions (for context)
    pub previous_attempts: Vec<ValidationAttempt>,
}

/// Record of a validation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationAttempt {
    /// Round number
    pub round: usize,

    /// What was tried
    pub plan: CommandPlan,

    /// What happened
    pub result: ExecutionResult,

    /// Why it wasn't sufficient
    pub reason: String,
}

/// Response from validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    /// Decision about whether to proceed
    pub decision: ValidationDecision,

    /// Confidence in the decision (0.0-1.0)
    pub confidence: f64,

    /// Reasoning behind the decision
    pub reasoning: String,
}

/// Validation loop coordinator
pub struct ValidationLoop {
    max_rounds: usize,
}

impl ValidationLoop {
    pub fn new() -> Self {
        Self {
            max_rounds: MAX_VALIDATION_ROUNDS,
        }
    }

    /// Check if we should continue validating
    pub fn should_continue(&self, round: usize, decision: &ValidationDecision) -> bool {
        if round >= self.max_rounds {
            return false;
        }

        match decision {
            ValidationDecision::Sufficient => false,
            ValidationDecision::NeedMoreData { .. } => true,
            ValidationDecision::Ambiguous { .. } => true,
        }
    }

    /// Build validation request for current round
    pub fn build_request(
        &self,
        intent: &Intent,
        exec_result: &ExecutionResult,
        round: usize,
        previous_attempts: Vec<ValidationAttempt>,
    ) -> ValidationRequest {
        ValidationRequest {
            intent: intent.clone(),
            execution_result: exec_result.clone(),
            round,
            previous_attempts,
        }
    }
}

impl Default for ValidationLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Build validation prompt for LLM
pub fn build_validation_prompt(request: &ValidationRequest) -> String {
    let mut prompt = format!(
        "# Validation Task\n\n\
        You are validating the results of command execution for the query: \"{}\"\n\n\
        Round: {}/{}\n\n",
        request.intent.query, request.round, MAX_VALIDATION_ROUNDS
    );

    // Show current execution results
    prompt.push_str("## Current Execution Results\n\n");
    for (i, cmd_result) in request.execution_result.command_results.iter().enumerate() {
        prompt.push_str(&format!(
            "Command {}: {}\n\
            Evidence: {:?}\n\
            Exit code: {}\n",
            i + 1,
            cmd_result.full_command,
            cmd_result.evidence.kind,
            cmd_result.exit_code
        ));

        if !cmd_result.stdout.is_empty() {
            let output_preview: Vec<&str> = cmd_result.stdout.lines().take(10).collect();
            prompt.push_str(&format!("Output:\n{}\n", output_preview.join("\n")));
        }

        if !cmd_result.stderr.is_empty() && !cmd_result.success {
            prompt.push_str(&format!("Errors: {}\n", cmd_result.stderr.lines().next().unwrap_or("")));
        }

        prompt.push('\n');
    }

    // Show previous attempts if any
    if !request.previous_attempts.is_empty() {
        prompt.push_str("## Previous Attempts\n\n");
        for attempt in &request.previous_attempts {
            prompt.push_str(&format!(
                "Round {}: {} ({})\n",
                attempt.round,
                attempt.plan.commands.first()
                    .map(|c| c.command.as_str())
                    .unwrap_or("no commands"),
                attempt.reason
            ));
        }
        prompt.push('\n');
    }

    prompt.push_str(&format!(
        "## Your Task\n\
        Decide if the current results are sufficient to answer the user's question.\n\n\
        Return your decision as JSON:\n\
        {{\n\
          \"decision\": \"Sufficient\" | {{\"NeedMoreData\": {{\"reason\": \"...\", \"suggested_approach\": \"...\"}}}} | {{\"Ambiguous\": {{\"reason\": \"...\", \"alternatives\": [...]}}}},\n\
          \"confidence\": 0.0-1.0,\n\
          \"reasoning\": \"Brief explanation\"\n\
        }}\n\n\
        Guidelines:\n\
        - If you have enough data to answer confidently, choose \"Sufficient\"\n\
        - If commands failed due to missing tools or permissions, choose \"Sufficient\" (we can't fix that)\n\
        - If you need different commands to get missing data, choose \"NeedMoreData\"\n\
        - If results are contradictory, choose \"Ambiguous\"\n\
        - Round {} is the last chance - prefer \"Sufficient\" to avoid endless loops\n",
        MAX_VALIDATION_ROUNDS
    ));

    prompt
}

/// Parse validation response from LLM JSON
pub fn parse_validation_response(json: serde_json::Value) -> Result<ValidationResponse, String> {
    let decision_value = json.get("decision")
        .ok_or_else(|| "Missing 'decision' field".to_string())?;

    let decision = if let Some(s) = decision_value.as_str() {
        if s == "Sufficient" {
            ValidationDecision::Sufficient
        } else {
            return Err(format!("Unknown decision string: {}", s));
        }
    } else if let Some(obj) = decision_value.as_object() {
        if let Some(need_more) = obj.get("NeedMoreData") {
            ValidationDecision::NeedMoreData {
                reason: need_more.get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No reason provided")
                    .to_string(),
                suggested_approach: need_more.get("suggested_approach")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }
        } else if let Some(ambiguous) = obj.get("Ambiguous") {
            let alternatives = ambiguous.get("alternatives")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            ValidationDecision::Ambiguous {
                reason: ambiguous.get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No reason provided")
                    .to_string(),
                alternatives,
            }
        } else {
            return Err("Unknown decision variant".to_string());
        }
    } else {
        return Err("Invalid decision format".to_string());
    };

    let confidence = json.get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5);

    let reasoning = json.get("reasoning")
        .and_then(|v| v.as_str())
        .unwrap_or("No reasoning provided")
        .to_string();

    Ok(ValidationResponse {
        decision,
        confidence,
        reasoning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor_core::{CommandResult, EvidenceItem, EvidenceKind};
    use crate::planner_core::{DomainType, GoalType, SafetyLevel};

    fn test_intent() -> Intent {
        Intent {
            goal: GoalType::Inspect,
            domain: DomainType::Packages,
            constraints: vec![],
            query: "do I have games?".to_string(),
        }
    }

    fn test_exec_result() -> ExecutionResult {
        ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
                goal_description: None,
                assumptions: vec![],
                confidence: 0.9,
            },
            command_results: vec![CommandResult {
                command: "pacman".to_string(),
                full_command: "pacman -Qs games".to_string(),
                exit_code: 0,
                stdout: "steam 1.0.0\n".to_string(),
                stderr: String::new(),
                success: true,
                time_ms: 10,
                evidence: EvidenceItem {
                    command: "pacman -Qs games".to_string(),
                    exit_code: 0,
                    stderr_snippet: String::new(),
                    kind: EvidenceKind::Positive,
                    summary: Some("Found packages".to_string()),
                },
            }],
            success: true,
            execution_time_ms: 10,
        }
    }

    #[test]
    fn test_validation_loop_max_rounds() {
        let loop_coordinator = ValidationLoop::new();
        assert_eq!(loop_coordinator.max_rounds, MAX_VALIDATION_ROUNDS);

        // Should not continue past max rounds
        let decision = ValidationDecision::NeedMoreData {
            reason: "Need more".to_string(),
            suggested_approach: "Try something else".to_string(),
        };
        assert!(!loop_coordinator.should_continue(MAX_VALIDATION_ROUNDS, &decision));
    }

    #[test]
    fn test_should_continue_with_sufficient() {
        let loop_coordinator = ValidationLoop::new();
        let decision = ValidationDecision::Sufficient;

        // Should not continue if sufficient
        assert!(!loop_coordinator.should_continue(1, &decision));
    }

    #[test]
    fn test_should_continue_with_need_more_data() {
        let loop_coordinator = ValidationLoop::new();
        let decision = ValidationDecision::NeedMoreData {
            reason: "Missing data".to_string(),
            suggested_approach: "Try different command".to_string(),
        };

        // Should continue if round < max_rounds
        assert!(loop_coordinator.should_continue(1, &decision));
        assert!(loop_coordinator.should_continue(2, &decision));
        assert!(!loop_coordinator.should_continue(3, &decision));
    }

    #[test]
    fn test_build_validation_request() {
        let loop_coordinator = ValidationLoop::new();
        let intent = test_intent();
        let exec_result = test_exec_result();

        let request = loop_coordinator.build_request(&intent, &exec_result, 1, vec![]);

        assert_eq!(request.round, 1);
        assert_eq!(request.intent.query, "do I have games?");
        assert!(request.previous_attempts.is_empty());
    }

    #[test]
    fn test_build_validation_prompt() {
        let intent = test_intent();
        let exec_result = test_exec_result();
        let request = ValidationRequest {
            intent,
            execution_result: exec_result,
            round: 1,
            previous_attempts: vec![],
        };

        let prompt = build_validation_prompt(&request);

        assert!(prompt.contains("do I have games?"));
        assert!(prompt.contains("Round: 1"));
        assert!(prompt.contains("pacman -Qs games"));
        assert!(prompt.contains("Positive"));
    }

    #[test]
    fn test_parse_sufficient_response() {
        let json = serde_json::json!({
            "decision": "Sufficient",
            "confidence": 0.95,
            "reasoning": "We have all the data needed"
        });

        let response = parse_validation_response(json).unwrap();
        assert_eq!(response.decision, ValidationDecision::Sufficient);
        assert_eq!(response.confidence, 0.95);
        assert!(response.reasoning.contains("all the data"));
    }

    #[test]
    fn test_parse_need_more_data_response() {
        let json = serde_json::json!({
            "decision": {
                "NeedMoreData": {
                    "reason": "Missing file list",
                    "suggested_approach": "Run ls command"
                }
            },
            "confidence": 0.7,
            "reasoning": "Need directory contents"
        });

        let response = parse_validation_response(json).unwrap();
        match response.decision {
            ValidationDecision::NeedMoreData { reason, suggested_approach } => {
                assert_eq!(reason, "Missing file list");
                assert_eq!(suggested_approach, "Run ls command");
            }
            _ => panic!("Wrong decision type"),
        }
    }

    #[test]
    fn test_parse_ambiguous_response() {
        let json = serde_json::json!({
            "decision": {
                "Ambiguous": {
                    "reason": "Contradictory results",
                    "alternatives": ["Try method A", "Try method B"]
                }
            },
            "confidence": 0.6,
            "reasoning": "Results don't match"
        });

        let response = parse_validation_response(json).unwrap();
        match response.decision {
            ValidationDecision::Ambiguous { reason, alternatives } => {
                assert_eq!(reason, "Contradictory results");
                assert_eq!(alternatives.len(), 2);
            }
            _ => panic!("Wrong decision type"),
        }
    }
}
