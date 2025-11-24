// Beta.162: Security Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct SecurityRecipe;

#[derive(Debug, PartialEq)]
enum SecurityOperation {
    Install,
    CheckStatus,
    ConfigureFail2ban,
    ListTools,
}

impl SecurityOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("configure") || input_lower.contains("setup fail2ban") {
            SecurityOperation::ConfigureFail2ban
        } else if input_lower.contains("check") || input_lower.contains("status") {
            SecurityOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            SecurityOperation::ListTools
        } else {
            SecurityOperation::Install
        }
    }
}

impl SecurityRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("fail2ban") || input_lower.contains("aide")
            || input_lower.contains("intrusion detection") || input_lower.contains("security tool")
            || input_lower.contains("brute force protection");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = SecurityOperation::detect(user_input);
        match operation {
            SecurityOperation::Install => Self::build_install_plan(telemetry),
            SecurityOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            SecurityOperation::ConfigureFail2ban => Self::build_configure_fail2ban_plan(telemetry),
            SecurityOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("fail2ban") { "fail2ban" }
        else if input_lower.contains("aide") { "aide" }
        else { "fail2ban" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "fail2ban" => ("fail2ban", "fail2ban", "Brute force protection service"),
            "aide" => ("AIDE", "aide", "Advanced Intrusion Detection Environment"),
            _ => ("fail2ban", "fail2ban", "Brute force protection service"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("security.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        Ok(ActionPlan {
            analysis: format!("Installing {} security tool", tool_name),
            goals: vec![format!("Install {} - {}", tool_name, description)],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-systemd".to_string(),
                    description: "Verify systemd is available".to_string(),
                    command: "which systemctl".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: format!("enable-{}", tool),
                    description: format!("Enable {} service", tool_name),
                    command: format!("sudo systemctl enable --now {}", package_name),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some(format!("disable-{}", tool)),
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("disable-{}", tool),
                    description: format!("Disable {} service", tool_name),
                    command: format!("sudo systemctl disable --now {}", package_name),
                },
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: format!("{} installed and enabled. Check status: sudo systemctl status {}", tool_name, package_name),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("security_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("security.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking security tools".to_string(),
            goals: vec!["List installed security tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-security-tools".to_string(),
                    description: "List security tools".to_string(),
                    command: "pacman -Q fail2ban aide 2>/dev/null || echo 'No security tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-services".to_string(),
                    description: "Check security service status".to_string(),
                    command: "systemctl is-active fail2ban aide 2>/dev/null || echo 'Services not running'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed security tools and service status".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("security_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_configure_fail2ban_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("security.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ConfigureFail2ban"));

        Ok(ActionPlan {
            analysis: "Configuring fail2ban for SSH protection".to_string(),
            goals: vec![
                "Enable fail2ban SSH jail".to_string(),
                "Restart fail2ban service".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-fail2ban".to_string(),
                    description: "Verify fail2ban is installed".to_string(),
                    command: "pacman -Q fail2ban".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "configure-ssh-jail".to_string(),
                    description: "Enable SSH jail in fail2ban".to_string(),
                    command: r#"echo -e '[sshd]\nenabled = true\nport = ssh\nlogpath = /var/log/auth.log\nmaxretry = 5' | sudo tee /etc/fail2ban/jail.local"#.to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some("remove-jail-config".to_string()),
                    requires_confirmation: true,
                },
                CommandStep {
                    id: "restart-fail2ban".to_string(),
                    description: "Restart fail2ban service".to_string(),
                    command: "sudo systemctl restart fail2ban".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-jail-config".to_string(),
                    description: "Remove custom jail configuration".to_string(),
                    command: "sudo rm -f /etc/fail2ban/jail.local".to_string(),
                },
            ],
            notes_for_user: "fail2ban configured for SSH protection. Check banned IPs: sudo fail2ban-client status sshd".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("security_configure_fail2ban".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("security.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available security tools".to_string(),
            goals: vec!["List available security tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available security tools".to_string(),
                    command: r"echo 'Available:\n- fail2ban (official) - Brute force protection\n- AIDE (official) - Intrusion detection\n- ClamAV (see antivirus recipe)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Security tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("security_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matches() {
        assert!(SecurityRecipe::matches_request("install fail2ban"));
        assert!(SecurityRecipe::matches_request("setup intrusion detection"));
        assert!(SecurityRecipe::matches_request("configure fail2ban"));
        assert!(!SecurityRecipe::matches_request("what is fail2ban"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install fail2ban".to_string());
        let plan = SecurityRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
