//! Confirmation UI - User confirmation prompts for execution safety
//!
//! v6.50.0: Display plan summaries and collect user confirmation

use crate::execution_safety::{ExecutionMode, PlanSummary};
use crate::planner_core::CommandPlan;
use std::io::{self, Write};

/// Display plan summary and prompt for confirmation
///
/// Returns Ok(true) if user confirms, Ok(false) if user cancels
pub fn confirm_plan_execution(plan: &CommandPlan, is_interactive: bool) -> io::Result<bool> {
    let summary = plan.compute_plan_summary(is_interactive);

    // Display plan header
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 EXECUTION PLAN");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Display description
    println!("Goal: {}", summary.description);
    println!("Risk: {}", summary.risk_description());
    println!("Commands: {}", summary.command_count);

    // Display domains
    if !summary.domains.is_empty() {
        print!("Domains: ");
        for (i, domain) in summary.domains.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{:?}", domain);
        }
        println!();
    }

    // Backup indicator
    if summary.will_create_backups {
        println!("✓ Backups will be created before changes");
    }

    println!();

    // Display commands
    println!("Commands to execute:");
    for (i, cmd) in plan.commands.iter().enumerate() {
        let full_cmd = if cmd.args.is_empty() {
            cmd.command.clone()
        } else {
            format!("{} {}", cmd.command, cmd.args.join(" "))
        };

        let requires_root = if cmd.requires_root { "🔐 " } else { "   " };
        println!("  {}{}) {} - {}", requires_root, i + 1, full_cmd, cmd.purpose);
    }
    println!();

    // Check execution mode and prompt
    match summary.execution_mode {
        ExecutionMode::PlanOnly => {
            println!("⚠️  HIGH RISK: This plan will not be executed automatically.");
            println!("   The operations above are potentially dangerous.");
            println!("   Review carefully. If you need to proceed, execute manually.\n");
            Ok(false) // Do not execute
        }
        ExecutionMode::ConfirmRequired => {
            // Show confirmation prompt
            print!("{} ", summary.confirmation_prompt());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let confirmed = input.trim().eq_ignore_ascii_case("y")
                || input.trim().eq_ignore_ascii_case("yes");

            if !confirmed {
                println!("\n✗ Cancelled. No changes were made.\n");
            } else {
                println!("\n✓ Confirmed. Executing plan...\n");
            }

            Ok(confirmed)
        }
        ExecutionMode::Automatic => {
            // Future: automatic execution
            println!("ℹ️  Executing automatically (trusted operation)...\n");
            Ok(true)
        }
    }
}

/// Display post-validation results
pub fn display_post_validation(validation: &crate::action_episodes::PostValidation) {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 POST-EXECUTION ASSESSMENT");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Satisfaction score
    let score_percent = validation.satisfaction_score * 100.0;
    let score_indicator = if score_percent >= 90.0 {
        "✓"
    } else if score_percent >= 70.0 {
        "⚠️"
    } else {
        "✗"
    };

    println!("{} Satisfaction: {:.0}%", score_indicator, score_percent);
    println!();

    // Summary
    println!("{}", validation.summary);
    println!();

    // Concerns
    if !validation.residual_concerns.is_empty() {
        println!("⚠️  Residual Concerns:");
        for concern in &validation.residual_concerns {
            println!("   • {}", concern);
        }
        println!();
    }

    // Suggested checks
    if !validation.suggested_checks.is_empty() {
        println!("💡 Suggested verification commands:");
        for check in &validation.suggested_checks {
            println!("   $ {}", check);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner_core::{CommandPlan, PlannedCommand, SafetyLevel, StepRiskLevel};

    fn make_test_plan(command: &str, purpose: &str, risk: StepRiskLevel) -> CommandPlan {
        CommandPlan {
            commands: vec![PlannedCommand {
                command: command.to_string(),
                args: vec![],
                purpose: purpose.to_string(),
                requires_tools: vec![],
                risk_level: risk,
                writes_files: false,
                requires_root: false,
                expected_outcome: None,
                validation_hint: None,
            }],
            safety_level: SafetyLevel::ReadOnly,
            fallbacks: vec![],
            expected_output: "test".to_string(),
            reasoning: "test".to_string(),
            goal_description: Some("Test operation".to_string()),
            assumptions: vec![],
            confidence: 0.9,
        }
    }

    #[test]
    fn test_high_risk_plan_rejected() {
        let plan = make_test_plan("fdisk", "partition disk", StepRiskLevel::High);
        let summary = plan.compute_plan_summary(true);

        // High risk should be PlanOnly
        assert_eq!(summary.execution_mode, ExecutionMode::PlanOnly);
    }

    #[test]
    fn test_safe_plan_requires_confirmation() {
        let plan = make_test_plan("ls", "list files", StepRiskLevel::ReadOnly);
        let summary = plan.compute_plan_summary(true);

        // Safe with interactive should require confirmation
        assert_eq!(summary.execution_mode, ExecutionMode::ConfirmRequired);
    }
}
