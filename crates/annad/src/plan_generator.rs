//! Plan Generator - Generate ActionPlans from LLM.
//! Phase 16: Turn fallback into real execution.
//! Phase 17: Preflight checks, verification, and rollback.
//!
//! When the LLM would have given manual instructions (blocked by sanitization),
//! we ask it to generate a structured ActionPlan instead.

use anna_shared::action_plan::ActionPlan;
use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::ollama;

// Template plans removed - LLM generates all plans dynamically now.

/// Request structure for plan generation.
#[derive(Debug, Deserialize)]
struct PlanResponse {
    summary: String,
    explanation: String,
    steps: Vec<StepResponse>,
    verification_command: Option<String>,
    verification_pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StepResponse {
    description: String,
    command: String,
    needs_sudo: bool,
}

/// Generate an action plan from the investigation state.
pub async fn generate_plan(
    model: &str,
    question: &str,
    investigation_context: &str,
) -> Result<ActionPlan> {
    info!("Generating action plan for: {}", question);

    let prompt = format!(
        r#"You are Anna, an Arch Linux system assistant. The user asked: "{question}"

Based on your investigation:
{investigation_context}

Generate a STRUCTURED ACTION PLAN to accomplish this task. You will execute the commands yourself - the user will only confirm yes/no.

Respond with ONLY valid JSON in this exact format:
{{
  "summary": "One-line description of what you'll do",
  "explanation": "2-3 sentences explaining why this approach works",
  "steps": [
    {{
      "description": "Human-readable step description",
      "command": "Exact shell command to run",
      "needs_sudo": true or false
    }}
  ],
  "verification_command": "Optional command to verify success",
  "verification_pattern": "Text that should appear in verification output if successful"
}}

RULES:
- Each step must have exactly ONE command
- Use needs_sudo: true for any privileged operations
- Commands must be non-interactive (no prompts, use -y flags where needed)
- Keep the plan minimal - only essential steps
- Include verification if possible

JSON only, no markdown, no explanation outside the JSON:"#
    );

    let response = ollama::chat_with_timeout(model, &prompt, 30).await?;
    debug!("Plan generation response: {}", response);

    // Parse the JSON response
    let plan_response: PlanResponse = serde_json::from_str(&response).map_err(|e| {
        warn!("Failed to parse plan JSON: {}", e);
        anyhow::anyhow!("Invalid plan format: {}", e)
    })?;

    // Build the ActionPlan
    let mut plan = ActionPlan::new(question, &plan_response.summary, &plan_response.explanation);

    for step in plan_response.steps {
        plan.add_step(&step.description, &step.command, step.needs_sudo);
    }

    // Add verification if provided
    if let (Some(cmd), Some(pattern)) = (
        plan_response.verification_command,
        plan_response.verification_pattern,
    ) {
        plan.set_verification(&cmd, &pattern, "Verify changes were applied");
    }

    info!(
        "Generated plan with {} steps, requires_sudo={}",
        plan.steps.len(),
        plan.requires_sudo()
    );

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_response_parsing() {
        let json = r#"{
            "summary": "Install package",
            "explanation": "Using pacman to install",
            "steps": [{"description": "Install", "command": "pacman -S pkg", "needs_sudo": true}],
            "verification_command": "pacman -Q pkg",
            "verification_pattern": "pkg"
        }"#;
        let resp: PlanResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.steps.len(), 1);
        assert!(resp.steps[0].needs_sudo);
    }
}
