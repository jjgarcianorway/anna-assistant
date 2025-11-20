// Beta.161: Gamepad & Controller Support Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct GamepadRecipe;

#[derive(Debug, PartialEq)]
enum GamepadOperation {
    Install,
    CheckStatus,
    TestController,
    ListTools,
}

impl GamepadOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("test") {
            GamepadOperation::TestController
        } else if input_lower.contains("check") || input_lower.contains("status") {
            GamepadOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            GamepadOperation::ListTools
        } else {
            GamepadOperation::Install
        }
    }
}

impl GamepadRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("gamepad") || input_lower.contains("controller")
            || input_lower.contains("joystick") || input_lower.contains("steam input")
            || input_lower.contains("xbox controller") || input_lower.contains("ps4 controller")
            || input_lower.contains("ps5 controller");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("test");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = GamepadOperation::detect(user_input);
        match operation {
            GamepadOperation::Install => Self::build_install_plan(telemetry),
            GamepadOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            GamepadOperation::TestController => Self::build_test_controller_plan(telemetry),
            GamepadOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gamepad.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));

        Ok(ActionPlan {
            analysis: "Installing gamepad/controller support".to_string(),
            goals: vec![
                "Install jstest-gtk (controller testing)".to_string(),
                "Install xboxdrv (Xbox controller driver)".to_string(),
            ],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "install-jstest".to_string(),
                    description: "Install jstest-gtk".to_string(),
                    command: "sudo pacman -S --needed --noconfirm jstest-gtk".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-jstest".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "install-xboxdrv".to_string(),
                    description: "Install xboxdrv (optional Xbox driver)".to_string(),
                    command: "yay -S --needed --noconfirm xboxdrv || paru -S --needed --noconfirm xboxdrv || echo 'xboxdrv install failed (optional)'".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-xboxdrv".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-jstest".to_string(),
                    description: "Remove jstest-gtk".to_string(),
                    command: "sudo pacman -Rns --noconfirm jstest-gtk".to_string(),
                },
                RollbackStep {
                    id: "remove-xboxdrv".to_string(),
                    description: "Remove xboxdrv".to_string(),
                    command: "sudo pacman -Rns --noconfirm xboxdrv".to_string(),
                },
            ],
            notes_for_user: "Controller support installed. Test with: jstest-gtk. Steam has built-in controller support.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gamepad_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gamepad.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking gamepad support".to_string(),
            goals: vec!["List installed gamepad tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-gamepad-tools".to_string(),
                    description: "List gamepad tools".to_string(),
                    command: "pacman -Q jstest-gtk xboxdrv 2>/dev/null || echo 'No gamepad tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "detect-controllers".to_string(),
                    description: "Detect connected controllers".to_string(),
                    command: "ls /dev/input/js* 2>/dev/null && echo 'Controllers detected' || echo 'No controllers connected'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows gamepad tool status and connected controllers".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gamepad_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_test_controller_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gamepad.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("TestController"));

        Ok(ActionPlan {
            analysis: "Testing controller input".to_string(),
            goals: vec!["Launch controller testing tool".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-jstest".to_string(),
                    description: "Verify jstest-gtk is installed".to_string(),
                    command: "which jstest-gtk".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "test-controller".to_string(),
                    description: "Launch jstest-gtk".to_string(),
                    command: "jstest-gtk &".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Launching jstest-gtk. Connect your controller and test buttons/axes.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gamepad_test_controller".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gamepad.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available gamepad tools".to_string(),
            goals: vec!["List available tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: "echo 'Available:\\n- jstest-gtk (official) - Controller testing tool\\n- xboxdrv (AUR) - Xbox controller driver\\n- Steam Input (built into Steam)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Gamepad/controller tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gamepad_list_tools".to_string()),
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
        assert!(GamepadRecipe::matches_request("install gamepad support"));
        assert!(GamepadRecipe::matches_request("setup xbox controller"));
        assert!(GamepadRecipe::matches_request("test controller"));
        assert!(!GamepadRecipe::matches_request("what is a gamepad"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install gamepad support".to_string());
        let plan = GamepadRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
