//! Dynamic Plan Generation - LLM generates ActionPlans for system config requests.
//! Phase 37: Instead of hardcoded handlers, LLM figures out what to do.

use anna_shared::action_plan::{ActionPlan, ActionStep};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Risk level for a generated plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Safe to execute immediately (config files, dconf, reversible changes)
    Low,
    /// Requires confirmation (packages, services, system files)
    High,
    /// Cannot execute (destructive, security-sensitive)
    Blocked,
}

/// Paths/patterns that are LOW risk (config tweaks, reversible).
const LOW_RISK_PATHS: &[&str] = &[
    // Config directories
    "/etc/dconf/",
    "/etc/systemd/logind.conf",
    "/etc/systemd/logind.conf.d/",
    "/var/lib/gdm/.config/",
    "/var/lib/gdm3/.config/",
    // GNOME/dconf commands
    "dconf update",
    "dconf write",
    "gsettings set",
    "gsettings reset",
    // Systemd commands (non-service)
    "systemctl daemon-reload",
    "loginctl",
    // Directory/file setup for configs
    "mkdir -p /var/lib/gdm",
    "mkdir -p /etc/dconf",
    "mkdir -p /etc/systemd/logind.conf.d",
    // Display settings
    "xrandr",
    "wlr-randr",
    // Safe file operations on known config paths
    "cp ~/.config/monitors.xml /var/lib/gdm",
    "chown gdm:gdm /var/lib/gdm",
    // NetworkManager (read/status only)
    "nmcli connection show",
    "nmcli device status",
    // Timezone/locale (reversible)
    "timedatectl set-timezone",
    "localectl set-locale",
];

/// Paths/patterns that are HIGH risk (need confirmation).
const HIGH_RISK_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/group",
    "/etc/sudoers",
    "/etc/fstab",
    "pacman -S",
    "pacman -R",
    "systemctl enable",
    "systemctl disable",
    "systemctl start",
    "systemctl stop",
    "rm -rf",
    "dd if=",
    "mkfs",
];

/// Paths/patterns that are BLOCKED (never execute).
const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "dd if=/dev/zero of=/dev/sd",
    "chmod 777 /",
    ":(){ :|:& };:",
    "> /dev/sda",
    "mkfs.ext4 /dev/sd",
];

/// Assess risk level of a command.
pub fn assess_command_risk(command: &str) -> RiskLevel {
    let cmd_lower = command.to_lowercase();

    // Check blocked patterns first
    for pattern in BLOCKED_PATTERNS {
        if cmd_lower.contains(pattern) {
            return RiskLevel::Blocked;
        }
    }

    // Check high risk patterns
    for pattern in HIGH_RISK_PATHS {
        if cmd_lower.contains(&pattern.to_lowercase()) {
            return RiskLevel::High;
        }
    }

    // Check low risk patterns - if matches any, it's low risk
    for pattern in LOW_RISK_PATHS {
        if cmd_lower.contains(&pattern.to_lowercase()) {
            return RiskLevel::Low;
        }
    }

    // Default to high risk for unknown commands
    RiskLevel::High
}

/// Assess overall risk of a plan.
pub fn assess_plan_risk(plan: &ActionPlan) -> RiskLevel {
    let mut highest_risk = RiskLevel::Low;

    for step in &plan.steps {
        let step_risk = assess_command_risk(&step.command);
        match step_risk {
            RiskLevel::Blocked => return RiskLevel::Blocked,
            RiskLevel::High => highest_risk = RiskLevel::High,
            RiskLevel::Low => {}
        }
    }

    highest_risk
}

/// LLM response format for plan generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmPlanResponse {
    pub can_help: bool,
    pub reason: Option<String>,
    pub steps: Vec<LlmPlanStep>,
    pub verification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmPlanStep {
    pub description: String,
    pub command: String,
    pub needs_sudo: bool,
}

/// System prompt for plan generation.
pub const PLAN_GENERATION_PROMPT: &str = r#"You are Anna, a Linux system administrator assistant for Arch Linux.

The user wants to make a system configuration change. Your job is to:
1. Determine if you can help with this request
2. If yes, provide the exact commands needed
3. Each command should be safe and reversible where possible

Respond in JSON format:
{
  "can_help": true/false,
  "reason": "why you can or cannot help (if cannot)",
  "steps": [
    {
      "description": "Human-readable description of this step",
      "command": "exact shell command to run",
      "needs_sudo": true/false
    }
  ],
  "verification": "optional command to verify success"
}

Rules:
- Only provide commands you're confident are correct
- Prefer dconf/gsettings for GNOME settings
- Prefer systemd drop-in files over editing main configs
- Never provide destructive commands (rm -rf /, dd to disk, etc.)
- If unsure, set can_help to false with explanation

User request: "#;

/// Parse LLM response into ActionPlan.
pub fn parse_llm_plan(response: &str, original_request: &str) -> Option<ActionPlan> {
    // Try to extract JSON from response
    let json_str = extract_json(response)?;

    let llm_plan: LlmPlanResponse = serde_json::from_str(&json_str).ok()?;

    if !llm_plan.can_help {
        info!("LLM declined to help: {:?}", llm_plan.reason);
        return None;
    }

    if llm_plan.steps.is_empty() {
        warn!("LLM returned empty steps");
        return None;
    }

    // Build ActionPlan from LLM response
    let summary = if original_request.len() > 50 {
        format!("{}...", &original_request[..47])
    } else {
        original_request.to_string()
    };

    let mut plan = ActionPlan::new(
        "llm-generated",
        &summary,
        &format!("Execute: {}", summary),
    );

    for step in llm_plan.steps {
        plan.add_step_full(
            ActionStep::new(&step.description, &step.command, step.needs_sudo)
        );
    }

    if let Some(verify_cmd) = llm_plan.verification {
        plan.set_verification(&verify_cmd, "", "Verify changes applied");
    }

    Some(plan)
}

/// Extract JSON from LLM response (handles markdown code blocks).
fn extract_json(response: &str) -> Option<String> {
    // Try direct parse first
    if response.trim().starts_with('{') {
        return Some(response.trim().to_string());
    }

    // Look for JSON in code blocks
    if let Some(start) = response.find("```json") {
        let rest = &response[start + 7..];
        if let Some(end) = rest.find("```") {
            return Some(rest[..end].trim().to_string());
        }
    }

    // Look for JSON in generic code blocks
    if let Some(start) = response.find("```\n{") {
        let rest = &response[start + 4..];
        if let Some(end) = rest.find("```") {
            return Some(rest[..end].trim().to_string());
        }
    }

    // Look for { ... } pattern
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Some(response[start..=end].to_string());
            }
        }
    }

    None
}

/// Check if a request looks like a system config request.
pub fn is_config_request(question: &str) -> bool {
    let q = question.to_lowercase();

    // Config/settings keywords
    let config_keywords = [
        "prevent", "disable", "enable", "stop", "configure", "set up", "setup",
        "change", "modify", "turn off", "turn on", "make", "don't let", "keep",
        "sleep", "suspend", "hibernate", "blank", "dim", "idle", "timeout",
        "scale", "resolution", "display", "screen", "monitor",
        "login", "gdm", "greeter", "lock",
        "theme", "appearance", "font", "cursor",
        "keyboard", "mouse", "touchpad",
        "network", "wifi", "bluetooth",
        "sound", "audio", "volume",
        "power", "battery", "lid",
    ];

    // Must contain at least one config keyword
    let has_config_keyword = config_keywords.iter().any(|k| q.contains(k));

    // Exclude pure questions (how does X work, what is X)
    let is_pure_question = q.starts_with("what is ") ||
                          q.starts_with("how does ") ||
                          q.starts_with("why does ") ||
                          q.starts_with("explain ");

    has_config_keyword && !is_pure_question
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_assessment_low() {
        assert_eq!(assess_command_risk("dconf update"), RiskLevel::Low);
        assert_eq!(assess_command_risk("mkdir -p /etc/dconf/db/gdm.d"), RiskLevel::Low);
        assert_eq!(assess_command_risk("tee /etc/systemd/logind.conf.d/lid.conf"), RiskLevel::Low);
    }

    #[test]
    fn test_risk_assessment_high() {
        assert_eq!(assess_command_risk("pacman -S firefox"), RiskLevel::High);
        assert_eq!(assess_command_risk("systemctl enable sshd"), RiskLevel::High);
    }

    #[test]
    fn test_risk_assessment_blocked() {
        assert_eq!(assess_command_risk("rm -rf /"), RiskLevel::Blocked);
        assert_eq!(assess_command_risk("dd if=/dev/zero of=/dev/sda"), RiskLevel::Blocked);
    }

    #[test]
    fn test_is_config_request() {
        assert!(is_config_request("prevent GDM from sleeping"));
        assert!(is_config_request("disable screen blanking"));
        assert!(is_config_request("stop my laptop from suspending"));
        assert!(!is_config_request("what is systemd"));
        assert!(!is_config_request("how does dconf work"));
    }

    #[test]
    fn test_extract_json() {
        let direct = r#"{"can_help": true, "steps": []}"#;
        assert!(extract_json(direct).is_some());

        let code_block = "Here's the plan:\n```json\n{\"can_help\": true}\n```";
        assert!(extract_json(code_block).is_some());
    }
}
