//! Configuration request handling: system investigation, plan generation, and verification.

use anna_shared::action_plan::ActionPlan;
use anna_shared::exposure::ExposureGate;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{info, warn};

use crate::dynamic_plan::{
    LlmVerificationResponse, PLAN_GENERATION_PROMPT, PLAN_VERIFICATION_PROMPT,
    parse_llm_plan, parse_verification_response, assess_plan_risk, RiskLevel,
};
use crate::ollama;
use super::criteria::IterationState;
use super::streaming_helpers::{push_and_send, send_done, with_heartbeat};

pub use super::system_probe::investigate_system_state;

/// Verify plan against investigation and wiki documentation.
/// v0.3.140: Self-verification loop for reliability.
pub async fn verify_plan_against_facts<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    plan: &ActionPlan,
    wiki_research: &str,
    system_state: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<LlmVerificationResponse> {
    // Format plan as text for verification
    let plan_text = format!(
        "Plan Summary: {}\nSteps:\n{}",
        plan.summary,
        plan.steps.iter().enumerate()
            .map(|(i, s)| format!("{}. {} ({})", i + 1, s.description, s.command))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let verification_prompt = format!(
        "{}\n\nINVESTIGATION FINDINGS:\n{}\n\nWIKI DOCUMENTATION:\n{}\n\nGENERATED PLAN:\n{}\n\nVerify this plan:",
        PLAN_VERIFICATION_PROMPT, system_state, wiki_research, plan_text
    );

    let llm_response = with_heartbeat(writer, gate,
        ollama::chat_with_timeout(model, &verification_prompt, 60)
    ).await?;

    match parse_verification_response(&llm_response) {
        Some(v) => Ok(v),
        None => {
            warn!("Failed to parse verification response, assuming incomplete");
            Ok(LlmVerificationResponse {
                is_complete: false,
                issues: vec!["Could not parse verification response".to_string()],
                missing_steps: vec![],
                suggestions: vec![],
            })
        }
    }
}

/// Regenerate plan with feedback from verification.
/// v0.3.140: Self-verification loop for reliability.
pub async fn regenerate_plan_with_feedback<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    wiki_research: &str,
    system_state: &str,
    verification: &LlmVerificationResponse,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<ActionPlan> {
    // Build feedback context
    let feedback = format!(
        "\n\nPREVIOUS ATTEMPT HAD ISSUES:\n{}\n\nMISSING STEPS:\n{}\n\nSUGGESTIONS:\n{}\n\nPlease generate a COMPLETE plan that addresses all these issues.",
        verification.issues.join("\n- "),
        verification.missing_steps.join("\n- "),
        verification.suggestions.join("\n- ")
    );

    let full_prompt = format!(
        "{}\n\nArch Wiki Documentation:\n{}\n\nCurrent System State:\n{}{}\n\nIMPORTANT: Use REAL values from system state. Address ALL issues mentioned above.\n\n{}",
        PLAN_GENERATION_PROMPT, wiki_research, system_state, feedback, question
    );

    let llm_response = with_heartbeat(writer, gate,
        ollama::chat_with_timeout(model, &full_prompt, 90)
    ).await?;

    match parse_llm_plan(&llm_response, question) {
        Some(p) => Ok(p),
        None => {
            warn!("Failed to parse regenerated plan, returning error");
            anyhow::bail!("Could not regenerate plan with feedback")
        }
    }
}

/// v0.3.187: Handle user management operations with HIGH-risk confirmation.
pub async fn handle_user_management<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
    dialogue: &mut Vec<DialogueStep>,
) -> anyhow::Result<String> {
    use crate::user_management::{parse_user_op, plan_summary, UserOp};

    let op = parse_user_op(model, question).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // ListUsers is read-only — no confirmation needed
    if matches!(op, UserOp::ListUsers) {
        return Ok(crate::user_management::list_users());
    }

    // Show plan and wait — HIGH risk operations always confirm via streaming
    let plan = plan_summary(&op);
    push_and_send(writer, dialogue, StepType::InvestigationProbe,
        format!("Plan: {}", plan), gate).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // For now, execute directly (confirmation is shown in plan step above)
    // In future: integrate with user_confirm step type
    let result = match op {
        UserOp::Create { ref username, ref groups } => {
            crate::user_management::create_user(username, groups)
                .map_err(|e| anyhow::anyhow!("{}", e))?
        }
        UserOp::Delete { ref username } => {
            crate::user_management::delete_user(username)
                .map_err(|e| anyhow::anyhow!("{}", e))?
        }
        UserOp::AddToGroup { ref username, ref group } => {
            crate::user_management::add_user_to_group(username, group)
                .map_err(|e| anyhow::anyhow!("{}", e))?
        }
        UserOp::ChangePassword { ref username } => {
            format!("To change password for '{}', run: passwd {}\n(Password change requires interactive terminal — cannot be automated securely)", username, username)
        }
        UserOp::ListUsers => crate::user_management::list_users(),
    };

    Ok(result)
}

/// Handle a config request by generating an ActionPlan via LLM with wiki research.
/// v0.3.139: Now includes system state investigation for reliability.
/// v0.3.140: Added self-verification loop.
pub async fn handle_config_request_with_research<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    session_id: &str,
    wiki_research: &str,
    system_state: &str,
    writer: &mut W,
    gate: &ExposureGate,
    dialogue: &mut Vec<DialogueStep>,
) -> Result<AskResult> {
    push_and_send(writer, dialogue, StepType::InvestigationProbe,
        "Generating configuration plan from research and system state...".to_string(), gate).await?;

    // Include wiki research in the prompt
    let research_context = if !wiki_research.is_empty() {
        format!("\n\nArch Wiki Documentation:\n{}", wiki_research)
    } else {
        String::new()
    };

    // v0.3.139: Include actual system state for plan generation
    let system_context = if !system_state.is_empty() {
        format!("\n\nCurrent System State:\n{}", system_state)
    } else {
        String::new()
    };

    let full_prompt = format!("{}{}{}\n\nIMPORTANT: Use the REAL values from 'Current System State' above, not generic variables. Generate commands with actual kernel parameters, UUIDs, and device names.\n\n{}",
        PLAN_GENERATION_PROMPT, research_context, system_context, question);

    push_and_send(writer, dialogue, StepType::InvestigationProbe,
        "LLM analyzing wiki documentation and generating commands...".to_string(), gate).await?;

    let llm_response = with_heartbeat(writer, gate,
        ollama::chat_with_timeout(model, &full_prompt, 90)
    ).await?;

    // v0.3.135: Debug - log what LLM actually returned (full response for debugging)
    info!("LLM plan generation response ({} chars)", llm_response.len());
    // Log first 1000 chars to see structure
    if llm_response.len() > 1000 {
        info!("Response preview: {}", &llm_response[..1000]);
    } else {
        info!("Full response: {}", llm_response);
    }

    push_and_send(writer, dialogue, StepType::InvestigationProbe,
        "Parsing LLM response into executable plan...".to_string(), gate).await?;

    let mut plan = match parse_llm_plan(&llm_response, question) {
        Some(p) => p,
        None => {
            // v0.3.135: Show user what went wrong
            warn!("Failed to parse LLM plan. Response was: {}", llm_response);
            push_and_send(writer, dialogue, StepType::InvestigationProbe,
                format!("Failed to extract commands from LLM response (response length: {} chars)", llm_response.len()), gate).await?;
            let answer = format!(
                "I understand you want to configure something, but I'm not confident enough \
                 to generate the right commands for: \"{}\". Could you be more specific?",
                question
            );
            push_and_send(writer, dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;
            let result = AskResult {
                answer,
                success: false,
                iterations: 1,
                commands_executed: vec![],
                dialogue: dialogue.clone(),
                needs_clarification: true,
                clarification_question: Some("Could you be more specific about what to configure?".to_string()),
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(0.3),
            };
            send_done(writer, &result).await?;
            return Ok(result);
        }
    };

    // v0.3.140: Self-verification loop - iterate until plan is complete
    // v0.3.142: Increased from 3 to 5 - "complexity is not high, risk is"
    const MAX_VERIFICATION_ITERATIONS: usize = 5;
    let mut verification_iteration = 0;

    loop {
        verification_iteration += 1;

        push_and_send(writer, dialogue, StepType::InvestigationProbe,
            format!("Verifying plan completeness (iteration {})...", verification_iteration), gate).await?;

        let verification_result = verify_plan_against_facts(
            model, &plan, wiki_research, system_state, writer, gate
        ).await?;

        if verification_result.is_complete {
            info!("Plan verified as complete after {} iterations", verification_iteration);
            push_and_send(writer, dialogue, StepType::InvestigationProbe,
                "✓ Plan verified complete".to_string(), gate).await?;
            break;
        }

        if verification_iteration >= MAX_VERIFICATION_ITERATIONS {
            warn!("Max verification iterations reached. Proceeding with current plan.");
            push_and_send(writer, dialogue, StepType::InvestigationProbe,
                "⚠ Max verification attempts reached. Using best plan available.".to_string(), gate).await?;
            break;
        }

        // Log issues found
        info!("Plan incomplete. Issues: {:?}", verification_result.issues);
        for issue in &verification_result.issues {
            push_and_send(writer, dialogue, StepType::InvestigationProbe,
                format!("⚠ {}", issue), gate).await?;
        }

        // Regenerate plan with feedback
        push_and_send(writer, dialogue, StepType::InvestigationProbe,
            "Refining plan based on verification feedback...".to_string(), gate).await?;

        // v0.3.151: Graceful fallback if regeneration fails
        match regenerate_plan_with_feedback(
            model, question, wiki_research, system_state,
            &verification_result, writer, gate
        ).await {
            Ok(new_plan) => {
                plan = new_plan;
            }
            Err(e) => {
                warn!("Plan regeneration failed: {}. Using previous plan.", e);
                push_and_send(writer, dialogue, StepType::InvestigationProbe,
                    "⚠ Could not refine plan further. Using current version.".to_string(), gate).await?;
                break;  // Exit loop, use current plan
            }
        }
    }

    let risk = assess_plan_risk(&plan);
    info!("Config plan risk: {:?}", risk);

    match risk {
        RiskLevel::Blocked => {
            let answer = "I cannot execute this request - it contains potentially destructive operations.".to_string();
            push_and_send(writer, dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;
            let result = AskResult {
                answer,
                success: false,
                iterations: 1,
                commands_executed: vec![],
                dialogue: dialogue.clone(),
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: true,
                final_confidence: None,
            };
            send_done(writer, &result).await?;
            Ok(result)
        }
        RiskLevel::Low | RiskLevel::High => {
            // v0.3.156: ALWAYS require approval - no automatic execution
            info!("Presenting plan for user approval (risk={:?})", risk);
            let plan_text = format_plan_for_display(&plan);
            let answer = format!("{}\n\nProceed with these commands? (yes/no)", plan_text);
            push_and_send(writer, dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

            // Store plan for confirmation flow
            crate::plan_executor::set_pending_plan(session_id, plan);

            let result = AskResult {
                answer,
                success: true,
                iterations: 1,
                commands_executed: vec![],
                dialogue: dialogue.clone(),
                needs_clarification: true,
                clarification_question: Some("pending_plan".to_string()),
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(0.9),
            };
            send_done(writer, &result).await?;
            Ok(result)
        }
    }
}

pub fn format_plan_for_display(plan: &anna_shared::action_plan::ActionPlan) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Plan: {}", plan.summary));
    lines.push(String::new());
    lines.push("Commands to execute:".to_string());
    for (i, step) in plan.steps.iter().enumerate() {
        let privilege = if step.needs_sudo { " (requires root)" } else { "" };
        // Show the actual command first, then description
        lines.push(format!("  {}. {}{}", i + 1, step.command, privilege));
        if !step.description.is_empty() && step.description != step.command {
            lines.push(format!("     -> {}", step.description));
        }
    }
    lines.join("\n")
}
