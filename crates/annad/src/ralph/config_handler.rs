//! Configuration request handler for non-streaming mode.
//! Simpler version of streaming config handling for annactl.

use anna_shared::action_plan::ActionPlan;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::{anyhow, Result};
use tracing::{debug, info, warn};
use std::process::Command;

use super::criteria::CompletionCriteria;

/// Investigate system state synchronously (no streaming output).
/// Returns formatted system info string for plan generation.
/// Includes DE/WM detection and modular config file resolution.
pub fn investigate_system_state_sync() -> Result<String> {
    let mut system_info = String::new();

    // Critical commands for plan generation
    let investigation_commands: Vec<(&str, &str)> = vec![
        ("cat /proc/cmdline", "Current kernel parameters"),
        ("[ -d /sys/firmware/efi ] && echo 'UEFI' || echo 'BIOS'", "Boot mode"),
        ("efibootmgr 2>/dev/null || echo 'N/A'", "Current boot entries"),
        ("findmnt -n -o UUID /", "Root filesystem UUID"),
        ("findmnt -n -o SOURCE,FSTYPE /", "Root filesystem type"),
        ("lsblk -ndo pkname $(findmnt -n -o SOURCE /boot 2>/dev/null) 2>/dev/null || echo 'N/A'", "Boot device"),
        ("uname -r", "Kernel version"),
        ("cat /etc/os-release | grep PRETTY_NAME", "OS version"),
        ("[ -d /boot/grub ] && echo 'GRUB detected' || echo 'No GRUB'", "GRUB check"),
        ("[ -d /boot/loader ] && echo 'systemd-boot detected' || echo 'No systemd-boot'", "systemd-boot check"),
        ("ls -la /boot/ 2>/dev/null | head -20", "Boot directory"),
    ];

    for (cmd, description) in investigation_commands {
        match Command::new("sh").arg("-c").arg(cmd).output() {
            Ok(result) if result.status.success() => {
                let output_clean = String::from_utf8_lossy(&result.stdout).trim().to_string();
                system_info.push_str(&format!("{}: {}\n", description, output_clean));
            }
            _ => {
                debug!("Investigation command failed: {}", cmd);
                system_info.push_str(&format!("{}: (unavailable)\n", description));
            }
        }
    }

    info!("System investigation complete ({} bytes)", system_info.len());
    Ok(system_info)
}

/// Investigate DE/WM context for configuration requests.
/// Returns a rich report including:
/// - Detected DE/WM and session type
/// - The correct change method (gsettings, config file, etc.)
/// - All config files including those sourced/included from main config
/// - Current value of the relevant setting if found
pub fn investigate_de_config(question: &str, username: &str) -> String {
    let de_ctx = crate::de_config::DesktopContext::detect(username);
    crate::de_config::build_config_investigation(&de_ctx, question)
}

/// Generate configuration plan using LLM.
pub async fn generate_config_plan(
    model: &str,
    question: &str,
    system_state: &str,
) -> Result<ActionPlan> {
    use crate::dynamic_plan::{PLAN_GENERATION_PROMPT, parse_llm_plan};
    use crate::ollama;

    let system_context = if !system_state.is_empty() {
        format!("\n\nCurrent System State:\n{}", system_state)
    } else {
        String::new()
    };

    let full_prompt = format!(
        "{}{}\n\nIMPORTANT: Use REAL values from system state, not variables. Generate specific commands.\n\n{}",
        PLAN_GENERATION_PROMPT, system_context, question
    );

    info!("Generating configuration plan for: {}", question);
    let llm_response = ollama::chat_with_timeout(model, &full_prompt, 90).await?;

    match parse_llm_plan(&llm_response, question) {
        Some(plan) => {
            info!("Generated plan with {} steps", plan.steps.len());
            Ok(plan)
        }
        None => {
            warn!("Failed to parse LLM plan response");
            Err(anyhow!("Could not generate configuration plan"))
        }
    }
}

/// Execute configuration plan synchronously.
/// Returns dialogue steps for the result.
pub async fn execute_config_plan(
    model: &str,
    plan: &ActionPlan,
    question: &str,
    criteria: &CompletionCriteria,
) -> Result<AskResult> {
    use crate::dynamic_plan::{assess_plan_risk, RiskLevel};
    use crate::plan_executor;

    let mut dialogue = Vec::new();

    // Record the question
    dialogue.push(DialogueStep {
        step_type: StepType::UserQuestion,
        content: question.to_string(),
    });

    // Risk assessment
    let risk = assess_plan_risk(plan);
    info!("Plan risk level: {:?}", risk);

    match risk {
        RiskLevel::Blocked => {
            let answer = "Cannot execute - contains potentially destructive operations.".to_string();
            dialogue.push(DialogueStep {
                step_type: StepType::FinalAnswer,
                content: answer.clone(),
            });
            return Ok(AskResult {
                answer,
                success: false,
                iterations: 1,
                commands_executed: vec![],
                dialogue,
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: true,
                final_confidence: Some(0.0),
            });
        }
        RiskLevel::High => {
            info!("High risk plan - would require user confirmation in interactive mode");
            // Non-streaming mode: proceed with caution
        }
        RiskLevel::Low => {
            debug!("Low risk level, proceeding with execution");
        }
    }

    // Show plan summary
    dialogue.push(DialogueStep {
        step_type: StepType::InvestigationProbe,
        content: format!("Configuration Plan: {}", plan.summary),
    });

    for (i, step) in plan.steps.iter().enumerate() {
        dialogue.push(DialogueStep {
            step_type: StepType::InvestigationProbe,
            content: format!("Step {}: {}", i + 1, step.description),
        });
    }

    // Execute plan
    info!("Executing plan with {} steps", plan.steps.len());
    let execution_result = plan_executor::execute_plan(plan);

    // Generate final answer based on execution
    let success = execution_result.success;
    let commands_executed: Vec<String> = plan.steps.iter()
        .map(|s| s.command.clone())
        .collect();

    let answer = if success {
        format!(
            "Configuration complete: {}\n\nExecuted {} steps successfully.",
            plan.summary,
            plan.steps.len()
        )
    } else {
        let succeeded_count = execution_result.step_results.iter().filter(|s| s.success).count();
        let failed_count = plan.steps.len() - succeeded_count;
        format!(
            "Configuration partially complete: {}\n\n{}/{} steps succeeded. {} failed.",
            plan.summary,
            succeeded_count,
            plan.steps.len(),
            failed_count
        )
    };

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: answer.clone(),
    });

    Ok(AskResult {
        answer,
        success,
        iterations: 1,
        commands_executed,
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(if success { 0.9 } else { 0.5 }),
    })
}

/// Handle configuration request in non-streaming mode.
/// Simplified version without wiki research or verification loops.
pub async fn handle_config_request_sync(
    model: &str,
    question: &str,
    criteria: &CompletionCriteria,
) -> Result<AskResult> {
    info!("Handling config request (non-streaming): {}", question);

    // Step 1: Investigate system state
    let system_state = match investigate_system_state_sync() {
        Ok(state) => state,
        Err(e) => {
            warn!("System investigation failed: {}", e);
            String::new() // Continue with empty state
        }
    };

    // Step 2: Generate plan
    let plan = generate_config_plan(model, question, &system_state).await?;

    // Step 3: Execute plan
    execute_config_plan(model, &plan, question, criteria).await
}
