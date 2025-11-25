//! Trace Renderer - Visible thinking trace for CLI
//!
//! v6.41.0: Shows how Anna and the LLM work together to answer questions.
//! The trace is human-readable, compact, and placed after the main answer.

use crate::executor_core::ExecutionResult;
use crate::interpreter_core::InterpretedAnswer;
use crate::planner_core::Intent;

/// Render the complete thinking trace
pub fn render_trace(
    intent: &Intent,
    exec_result: &ExecutionResult,
    answer: &InterpretedAnswer,
) -> String {
    let mut trace = String::new();

    trace.push_str("\n🧠 Anna thinking trace\n\n");

    // Intent section
    trace.push_str("Intent:\n");
    trace.push_str(&format!("  - Goal: {:?}\n", intent.goal));
    trace.push_str(&format!("  - Domain: {:?}\n", intent.domain));

    if !intent.constraints.is_empty() {
        trace.push_str("  - Constraints: ");
        let constraints: Vec<String> = intent.constraints.iter().map(|c| format!("{:?}", c)).collect();
        trace.push_str(&constraints.join(", "));
        trace.push('\n');
    }

    // Commands executed
    trace.push_str("\nCommands executed:\n");
    if exec_result.command_results.is_empty() {
        trace.push_str("  (no commands run)\n");
    } else {
        for cmd_result in &exec_result.command_results {
            if cmd_result.success {
                trace.push_str(&format!("  [CMD] {} ✓\n", cmd_result.full_command));
            } else {
                trace.push_str(&format!("  [CMD] {} ✗\n", cmd_result.full_command));
            }
        }
    }

    // Key outputs (truncated)
    trace.push_str("\nKey outputs:\n");
    let mut showed_output = false;
    for cmd_result in &exec_result.command_results {
        if cmd_result.success && !cmd_result.stdout.is_empty() {
            // Show first 3 lines of output
            let lines: Vec<&str> = cmd_result.stdout.lines().take(3).collect();
            if !lines.is_empty() {
                trace.push_str(&format!("  {}: {}\n", cmd_result.command, lines.join(", ")));
                showed_output = true;
            }
        }
    }

    if !showed_output {
        if exec_result.success {
            trace.push_str("  (commands succeeded but produced no output)\n");
        } else {
            trace.push_str("  (all commands failed)\n");
        }
    }

    // Reasoning
    trace.push_str("\nReasoning (LLM summary):\n");
    for line in answer.reasoning.lines().take(5) {
        trace.push_str(&format!("  {}\n", line.trim()));
    }

    // Execution time
    trace.push_str(&format!(
        "\nExecution time: {}ms\n",
        exec_result.execution_time_ms
    ));

    trace
}

/// Render a compact trace (for non-TTY or when space is limited)
pub fn render_compact_trace(
    intent: &Intent,
    exec_result: &ExecutionResult,
) -> String {
    let mut trace = String::new();

    trace.push_str("[");
    trace.push_str(&format!("{:?}/{:?} → ", intent.goal, intent.domain));

    let success_count = exec_result.command_results.iter().filter(|c| c.success).count();
    let total_count = exec_result.command_results.len();

    trace.push_str(&format!("{}/{} commands succeeded", success_count, total_count));
    trace.push_str(&format!(" in {}ms", exec_result.execution_time_ms));
    trace.push_str("]");

    trace
}

/// Check if trace should be shown (only in TTY mode)
pub fn should_show_trace() -> bool {
    // Check if stdout is a TTY
    atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor_core::{CommandResult, ExecutionResult};
    use crate::interpreter_core::{ConfidenceLevel, InterpretedAnswer};
    use crate::planner_core::{CommandPlan, DomainType, GoalType, Intent, SafetyLevel};

    #[test]
    fn test_render_trace() {
        let intent = Intent {
            goal: GoalType::Inspect,
            domain: DomainType::Packages,
            constraints: vec![],
            query: "do I have games?".to_string(),
        };

        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
                goal_description: Some("Check for games".to_string()),
                assumptions: vec![],
                confidence: 0.9,
                policy_decision: None,
            },
            command_results: vec![CommandResult {
                command: "pacman".to_string(),
                full_command: "pacman -Q steam".to_string(),
                exit_code: 0,
                stdout: "steam 1.0.0.79-2\n".to_string(),
                stderr: String::new(),
                success: true,
                time_ms: 10,
                evidence: crate::executor_core::EvidenceItem { command: "test".to_string(), exit_code: 0, stderr_snippet: String::new(), kind: crate::executor_core::EvidenceKind::Positive, summary: None },
            }],
            success: true,
            execution_time_ms: 10,
        };

        let answer = InterpretedAnswer {
            answer: "Yes, you have steam installed.".to_string(),
            details: None,
            confidence: ConfidenceLevel::High,
            reasoning: "Pacman query returned steam package.".to_string(),
            source: "Command output".to_string(),
            achieved_goal: true,
            validation_confidence: 0.95,
            followup_suggestions: vec![],
            short_summary: Some("Steam is installed".to_string()),
        };

        let trace = render_trace(&intent, &exec_result, &answer);

        assert!(trace.contains("Anna thinking trace"));
        assert!(trace.contains("Intent:"));
        assert!(trace.contains("Commands executed:"));
        assert!(trace.contains("pacman -Q steam"));
        assert!(trace.contains("Reasoning"));
    }

    #[test]
    fn test_render_compact_trace() {
        let intent = Intent {
            goal: GoalType::Check,
            domain: DomainType::Hardware,
            constraints: vec![],
            query: "test".to_string(),
        };

        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
                goal_description: Some("Hardware check test".to_string()),
                assumptions: vec![],
                confidence: 0.8,
                policy_decision: None,
            },
            command_results: vec![
                CommandResult {
                    command: "cmd1".to_string(),
                    full_command: "cmd1".to_string(),
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                    success: true,
                    time_ms: 5,
                evidence: crate::executor_core::EvidenceItem { command: "test".to_string(), exit_code: 0, stderr_snippet: String::new(), kind: crate::executor_core::EvidenceKind::Positive, summary: None },
                },
                CommandResult {
                    command: "cmd2".to_string(),
                    full_command: "cmd2".to_string(),
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: String::new(),
                    success: false,
                    time_ms: 5,
                evidence: crate::executor_core::EvidenceItem { command: "test".to_string(), exit_code: 0, stderr_snippet: String::new(), kind: crate::executor_core::EvidenceKind::Positive, summary: None },
                },
            ],
            success: false,
            execution_time_ms: 10,
        };

        let trace = render_compact_trace(&intent, &exec_result);
        assert!(trace.contains("Check/Hardware"));
        assert!(trace.contains("1/2 commands succeeded"));
        assert!(trace.contains("10ms"));
    }
}
