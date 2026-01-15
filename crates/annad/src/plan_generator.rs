//! Plan Generator - Generate ActionPlans from LLM.
//! Phase 16: Turn fallback into real execution.
//!
//! When the LLM would have given manual instructions (blocked by sanitization),
//! we ask it to generate a structured ActionPlan instead.

use anna_shared::action_plan::ActionPlan;
use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::ollama;

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

/// Generate a plan for common system configuration tasks.
/// This uses pre-defined templates for well-known operations.
pub fn generate_template_plan(question: &str) -> Option<ActionPlan> {
    let q = question.to_lowercase();

    // GDM resolution
    if q.contains("gdm") && q.contains("resolution") {
        return Some(gdm_resolution_plan(question));
    }

    // Disable sleep/suspend
    if (q.contains("disable") || q.contains("prevent") || q.contains("stop"))
        && (q.contains("sleep") || q.contains("suspend"))
    {
        return Some(disable_sleep_plan(question));
    }

    // Lid close behavior
    if q.contains("lid") && (q.contains("close") || q.contains("closing")) {
        return Some(lid_close_plan(question));
    }

    None
}

fn gdm_resolution_plan(question: &str) -> ActionPlan {
    // Extract resolution from question (default to 1920x1080)
    let resolution = extract_resolution(question).unwrap_or_else(|| "1920x1080".to_string());

    let mut plan = ActionPlan::new(
        question,
        &format!("Set GDM login screen resolution to {}", resolution),
        "This configures the display resolution for the GDM login screen by setting up \
         a custom Wayland configuration. The change takes effect on next login.",
    );

    // Step 1: Create monitors.xml config
    plan.add_step(
        "Create GDM monitor configuration",
        &format!(
            r#"mkdir -p /var/lib/gdm/.config && cat > /var/lib/gdm/.config/monitors.xml << 'EOF'
<monitors version="2">
  <configuration>
    <logicalmonitor>
      <x>0</x>
      <y>0</y>
      <primary>yes</primary>
      <monitor>
        <monitorspec>
          <connector>*</connector>
          <vendor>unknown</vendor>
          <product>unknown</product>
          <serial>unknown</serial>
        </monitorspec>
        <mode>
          <width>{}</width>
          <height>{}</height>
          <rate>60</rate>
        </mode>
      </monitor>
    </logicalmonitor>
  </configuration>
</monitors>
EOF"#,
            resolution.split('x').next().unwrap_or("1920"),
            resolution.split('x').nth(1).unwrap_or("1080")
        ),
        true,
    );

    // Step 2: Set permissions
    plan.add_step(
        "Set proper ownership",
        "chown -R gdm:gdm /var/lib/gdm/.config",
        true,
    );

    plan.set_verification(
        "cat /var/lib/gdm/.config/monitors.xml | grep -q 'width'",
        "width",
        "Verify monitor configuration exists",
    );

    plan
}

fn disable_sleep_plan(question: &str) -> ActionPlan {
    let mut plan = ActionPlan::new(
        question,
        "Disable automatic sleep and suspend",
        "This masks the systemd sleep targets and configures logind to ignore idle timeouts. \
         The system will no longer automatically sleep or suspend.",
    );

    // Step 1: Mask sleep targets
    plan.add_step(
        "Mask systemd sleep targets",
        "systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target",
        true,
    );

    // Step 2: Configure logind
    plan.add_step(
        "Configure logind to ignore idle",
        r#"mkdir -p /etc/systemd/logind.conf.d && cat > /etc/systemd/logind.conf.d/no-idle.conf << 'EOF'
[Login]
IdleAction=ignore
IdleActionSec=0
EOF"#,
        true,
    );

    // Step 3: Restart logind
    plan.add_step("Apply logind changes", "systemctl restart systemd-logind", true);

    plan.set_verification(
        "systemctl status sleep.target | head -3",
        "masked",
        "Verify sleep target is masked",
    );

    plan
}

fn lid_close_plan(question: &str) -> ActionPlan {
    // Determine the desired action (default to ignore)
    let action = if question.to_lowercase().contains("nothing")
        || question.to_lowercase().contains("ignore")
    {
        "ignore"
    } else if question.to_lowercase().contains("lock") {
        "lock"
    } else if question.to_lowercase().contains("suspend") {
        "suspend"
    } else {
        "ignore" // Default: do nothing
    };

    let mut plan = ActionPlan::new(
        question,
        &format!("Configure lid close to {}", action),
        &format!(
            "This configures the system to {} when the laptop lid is closed. \
             Applies to both AC power and battery.",
            action
        ),
    );

    plan.add_step(
        "Configure lid close behavior",
        &format!(
            r#"mkdir -p /etc/systemd/logind.conf.d && cat > /etc/systemd/logind.conf.d/lid.conf << 'EOF'
[Login]
HandleLidSwitch={}
HandleLidSwitchExternalPower={}
HandleLidSwitchDocked={}
EOF"#,
            action, action, action
        ),
        true,
    );

    plan.add_step("Apply changes", "systemctl restart systemd-logind", true);

    plan.set_verification(
        &format!("grep -r 'HandleLidSwitch={}' /etc/systemd/logind.conf.d/", action),
        action,
        "Verify lid switch configuration",
    );

    plan
}

/// Extract resolution from a question string.
fn extract_resolution(question: &str) -> Option<String> {
    // Common patterns: "1920x1080", "2560x1440", "3840x2160"
    let re = regex::Regex::new(r"(\d{3,4})[xX×](\d{3,4})").ok()?;
    re.captures(question)
        .map(|c| format!("{}x{}", &c[1], &c[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resolution() {
        assert_eq!(
            extract_resolution("set to 1920x1080"),
            Some("1920x1080".to_string())
        );
        assert_eq!(
            extract_resolution("change to 2560X1440"),
            Some("2560x1440".to_string())
        );
        assert_eq!(extract_resolution("no resolution here"), None);
    }

    #[test]
    fn test_generate_template_plan_gdm() {
        let plan = generate_template_plan("change GDM resolution to 1920x1080");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert!(plan.summary.contains("GDM"));
        assert!(plan.requires_sudo());
    }

    #[test]
    fn test_generate_template_plan_sleep() {
        let plan = generate_template_plan("disable sleep when idle");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert!(plan.summary.contains("sleep"));
        assert!(plan.steps.len() >= 2);
    }

    #[test]
    fn test_generate_template_plan_lid() {
        let plan = generate_template_plan("do nothing when lid closes");
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert!(plan.summary.contains("lid"));
    }
}
