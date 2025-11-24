// Beta.160: Communication Apps Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct CommunicationRecipe;

#[derive(Debug, PartialEq)]
enum CommunicationOperation {
    Install,
    CheckStatus,
    ListApps,
}

impl CommunicationOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            CommunicationOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            CommunicationOperation::ListApps
        } else {
            CommunicationOperation::Install
        }
    }
}

impl CommunicationRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("discord") || input_lower.contains("slack")
            || input_lower.contains("telegram") || input_lower.contains("signal")
            || input_lower.contains("chat app") || input_lower.contains("messaging");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list");
        let is_info_only = (input_lower.starts_with("what is") 
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }
    
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = CommunicationOperation::detect(user_input);
        match operation {
            CommunicationOperation::Install => Self::build_install_plan(telemetry),
            CommunicationOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            CommunicationOperation::ListApps => Self::build_list_apps_plan(telemetry),
        }
    }
    
    fn detect_app(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("discord") { "discord" }
        else if input_lower.contains("slack") { "slack-desktop" }
        else if input_lower.contains("telegram") { "telegram-desktop" }
        else if input_lower.contains("signal") { "signal-desktop" }
        else { "discord" }
    }
    
    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let app = Self::detect_app(user_input);
        let (app_name, package_name, is_aur) = match app {
            "discord" => ("Discord", "discord", false),
            "slack-desktop" => ("Slack", "slack-desktop", true),
            "telegram-desktop" => ("Telegram", "telegram-desktop", false),
            "signal-desktop" => ("Signal", "signal-desktop", true),
            _ => ("Discord", "discord", false),
        };
        
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("communication.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("app".to_string(), serde_json::json!(app_name));
        
        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };
        
        Ok(ActionPlan {
            analysis: format!("Installing {} chat application", app_name),
            goals: vec![format!("Install {}", app_name)],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-pacman".to_string(),
                    description: "Verify pacman is available".to_string(),
                    command: "which pacman".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", app),
                    description: format!("Install {}", app_name),
                    command: install_cmd,
                    risk_level: if is_aur { RiskLevel::Medium } else { RiskLevel::Low },
                    rollback_id: Some(format!("remove-{}", app)),
                    requires_confirmation: is_aur,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", app),
                    description: format!("Remove {}", app_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: format!("{} installed. Launch from application menu.", app_name),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("communication_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
    
    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("communication.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));
        
        Ok(ActionPlan {
            analysis: "Checking communication apps".to_string(),
            goals: vec!["List installed chat apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-apps".to_string(),
                    description: "List communication apps".to_string(),
                    command: "pacman -Q discord slack-desktop telegram-desktop signal-desktop 2>/dev/null || echo 'No apps installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed communication applications".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("communication_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
    
    fn build_list_apps_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("communication.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListApps"));
        
        Ok(ActionPlan {
            analysis: "Showing available communication apps".to_string(),
            goals: vec!["List available apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-apps".to_string(),
                    description: "Show available apps".to_string(),
                    command: "echo 'Available:\n- Discord (official)\n- Telegram (official)\n- Slack (AUR)\n- Signal (AUR)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Popular communication apps for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("communication_list_apps".to_string()),
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
        assert!(CommunicationRecipe::matches_request("install discord"));
        assert!(!CommunicationRecipe::matches_request("what is discord"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install discord".to_string());
        let plan = CommunicationRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
