// Beta.161: Wine & Windows Compatibility Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct WineRecipe;

#[derive(Debug, PartialEq)]
enum WineOperation {
    Install,
    CheckStatus,
    InstallLutris,
    ListTools,
}

impl WineOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("lutris") {
            WineOperation::InstallLutris
        } else if input_lower.contains("check") || input_lower.contains("status") {
            WineOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            WineOperation::ListTools
        } else {
            WineOperation::Install
        }
    }
}

impl WineRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("wine") || input_lower.contains("lutris")
            || input_lower.contains("windows game") || input_lower.contains("windows app")
            || input_lower.contains("windows compatibility");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = WineOperation::detect(user_input);
        match operation {
            WineOperation::Install => Self::build_install_plan(telemetry),
            WineOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            WineOperation::InstallLutris => Self::build_install_lutris_plan(telemetry),
            WineOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("wine.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));

        Ok(ActionPlan {
            analysis: "Installing Wine Windows compatibility layer".to_string(),
            goals: vec![
                "Install Wine".to_string(),
                "Install Winetricks (helper utilities)".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-multilib".to_string(),
                    description: "Verify multilib is enabled (required for Wine)".to_string(),
                    command: "grep -q '^\\[multilib\\]' /etc/pacman.conf && echo 'enabled' || echo 'disabled'".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "install-wine".to_string(),
                    description: "Install Wine and dependencies".to_string(),
                    command: "sudo pacman -S --needed --noconfirm wine wine-mono wine-gecko winetricks".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-wine".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-wine".to_string(),
                    description: "Remove Wine".to_string(),
                    command: "sudo pacman -Rns --noconfirm wine wine-mono wine-gecko winetricks".to_string(),
                },
            ],
            notes_for_user: "Wine installed. Run Windows applications with: wine program.exe".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("wine_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_install_lutris_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("wine.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("InstallLutris"));

        Ok(ActionPlan {
            analysis: "Installing Lutris gaming platform".to_string(),
            goals: vec![
                "Install Lutris".to_string(),
                "Includes Wine, DXVK, and other runners".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-multilib".to_string(),
                    description: "Verify multilib is enabled".to_string(),
                    command: "grep -q '^\\[multilib\\]' /etc/pacman.conf && echo 'enabled' || echo 'disabled'".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "install-lutris".to_string(),
                    description: "Install Lutris".to_string(),
                    command: "sudo pacman -S --needed --noconfirm lutris wine wine-mono wine-gecko".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-lutris".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-lutris".to_string(),
                    description: "Remove Lutris".to_string(),
                    command: "sudo pacman -Rns --noconfirm lutris".to_string(),
                },
            ],
            notes_for_user: "Lutris installed. Launch from application menu to manage game installations.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("wine_install_lutris".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("wine.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking Wine compatibility tools".to_string(),
            goals: vec!["List installed Wine tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-wine-tools".to_string(),
                    description: "List Wine tools".to_string(),
                    command: "pacman -Q wine lutris winetricks 2>/dev/null || echo 'No Wine tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed Windows compatibility tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("wine_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("wine.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available Wine tools".to_string(),
            goals: vec!["List available tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: "echo 'Available:\\n- Wine (official) - Windows compatibility layer\\n- Lutris (official) - Game manager with Wine\\n- Winetricks (official) - Wine helper utilities'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Windows compatibility tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("wine_list_tools".to_string()),
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
        assert!(WineRecipe::matches_request("install wine"));
        assert!(WineRecipe::matches_request("setup lutris"));
        assert!(WineRecipe::matches_request("install windows compatibility"));
        assert!(!WineRecipe::matches_request("what is wine"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install wine".to_string());
        let plan = WineRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
