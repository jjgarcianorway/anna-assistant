//! Approval UI - User confirmation for action plans (v6.43.0)
//!
//! Shows command plan to user and asks for explicit approval before execution.

use anna_common::planner_core::{CommandPlan, PlannedCommand, SafetyLevel, StepRiskLevel};
use std::io::{self, Write};

/// Display a plan to the user and ask for approval
pub fn request_approval(plan: &CommandPlan) -> io::Result<bool> {
    println!("\n📋 Anna proposes the following plan:\n");

    // Show goal description if available
    if let Some(goal) = &plan.goal_description {
        println!("Goal: {}", goal);
    }

    // Show safety level
    let safety_symbol = match plan.safety_level {
        SafetyLevel::ReadOnly => "🔍",
        SafetyLevel::MinimalWrite => "✏️",
        SafetyLevel::Risky => "⚠️",
    };
    println!("Safety level: {} {:?}", safety_symbol, plan.safety_level);

    // Show confidence if meaningful
    if plan.confidence > 0.0 {
        let confidence_pct = (plan.confidence * 100.0) as u32;
        println!("Confidence: {}%", confidence_pct);
    }

    // Show assumptions if any
    if !plan.assumptions.is_empty() {
        println!("\nAssumptions:");
        for assumption in &plan.assumptions {
            println!("  • {}", assumption);
        }
    }

    // Show commands
    println!("\nSteps:");
    for (idx, cmd) in plan.commands.iter().enumerate() {
        render_planned_command(idx + 1, cmd);
    }

    // Show fallbacks if any
    if !plan.fallbacks.is_empty() {
        println!("\nFallback commands (if primary commands fail):");
        for (idx, cmd) in plan.fallbacks.iter().enumerate() {
            render_planned_command(idx + 1, cmd);
        }
    }

    // Reasoning
    if !plan.reasoning.is_empty() {
        println!("\nReasoning: {}", plan.reasoning);
    }

    // Prompt for approval
    print!("\nApprove and execute? [y/N]: ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    let approved = matches!(response.trim().to_lowercase().as_str(), "y" | "yes");

    if approved {
        println!("✓ Approved. Executing...\n");
    } else {
        println!("✗ Cancelled.\n");
    }

    Ok(approved)
}

/// Render a single planned command
fn render_planned_command(step_num: usize, cmd: &PlannedCommand) {
    let risk_symbol = match cmd.risk_level {
        StepRiskLevel::ReadOnly => "🔍",
        StepRiskLevel::Low => "🟢",
        StepRiskLevel::Medium => "🟡",
        StepRiskLevel::High => "🔴",
    };

    let full_command = if cmd.args.is_empty() {
        cmd.command.clone()
    } else {
        format!("{} {}", cmd.command, cmd.args.join(" "))
    };

    println!("  {}. {} {} - {}", step_num, risk_symbol, full_command, cmd.purpose);

    // Show additional details if present
    if cmd.requires_root {
        println!("     ⚠️  Requires root privileges");
    }
    if cmd.writes_files {
        println!("     📝 Writes files");
    }
    if let Some(outcome) = &cmd.expected_outcome {
        println!("     Expected: {}", outcome);
    }
}

/// Check if the plan requires approval
/// v6.43.0: Only ReadOnly operations with high confidence skip approval
pub fn requires_approval(plan: &CommandPlan) -> bool {
    // High-risk plans always require approval
    if matches!(plan.safety_level, SafetyLevel::Risky) {
        return true;
    }

    // Plans with file writes require approval
    if plan.commands.iter().any(|c| c.writes_files) {
        return true;
    }

    // Plans requiring root require approval
    if plan.commands.iter().any(|c| c.requires_root) {
        return true;
    }

    // Medium-risk steps require approval
    if plan.commands.iter().any(|c| matches!(c.risk_level, StepRiskLevel::Medium | StepRiskLevel::High)) {
        return true;
    }

    // ReadOnly operations with high confidence (> 0.9) can skip approval
    if matches!(plan.safety_level, SafetyLevel::ReadOnly) && plan.confidence > 0.9 {
        return false;
    }

    // Default: require approval for MinimalWrite operations
    matches!(plan.safety_level, SafetyLevel::MinimalWrite)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readonly_high_confidence_skips_approval() {
        let plan = CommandPlan {
            commands: vec![PlannedCommand {
                command: "pacman".to_string(),
                args: vec!["-Q".to_string(), "steam".to_string()],
                purpose: "Check if steam is installed".to_string(),
                requires_tools: vec!["pacman".to_string()],
                risk_level: StepRiskLevel::ReadOnly,
                writes_files: false,
                requires_root: false,
                expected_outcome: None,
                validation_hint: None,
            }],
            safety_level: SafetyLevel::ReadOnly,
            fallbacks: vec![],
            expected_output: String::new(),
            reasoning: String::new(),
            goal_description: Some("Check for steam".to_string()),
            assumptions: vec![],
            confidence: 0.95,
        };

        assert!(!requires_approval(&plan));
    }

    #[test]
    fn test_risky_requires_approval() {
        let plan = CommandPlan {
            commands: vec![],
            safety_level: SafetyLevel::Risky,
            fallbacks: vec![],
            expected_output: String::new(),
            reasoning: String::new(),
            goal_description: None,
            assumptions: vec![],
            confidence: 0.8,
        };

        assert!(requires_approval(&plan));
    }

    #[test]
    fn test_writes_files_requires_approval() {
        let plan = CommandPlan {
            commands: vec![PlannedCommand {
                command: "echo".to_string(),
                args: vec![],
                purpose: "Write config".to_string(),
                requires_tools: vec![],
                risk_level: StepRiskLevel::Low,
                writes_files: true,
                requires_root: false,
                expected_outcome: None,
                validation_hint: None,
            }],
            safety_level: SafetyLevel::MinimalWrite,
            fallbacks: vec![],
            expected_output: String::new(),
            reasoning: String::new(),
            goal_description: None,
            assumptions: vec![],
            confidence: 0.8,
        };

        assert!(requires_approval(&plan));
    }

    #[test]
    fn test_requires_root_requires_approval() {
        let plan = CommandPlan {
            commands: vec![PlannedCommand {
                command: "systemctl".to_string(),
                args: vec!["restart".to_string(), "annad".to_string()],
                purpose: "Restart daemon".to_string(),
                requires_tools: vec![],
                risk_level: StepRiskLevel::Medium,
                writes_files: false,
                requires_root: true,
                expected_outcome: None,
                validation_hint: None,
            }],
            safety_level: SafetyLevel::MinimalWrite,
            fallbacks: vec![],
            expected_output: String::new(),
            reasoning: String::new(),
            goal_description: None,
            assumptions: vec![],
            confidence: 0.8,
        };

        assert!(requires_approval(&plan));
    }
}
