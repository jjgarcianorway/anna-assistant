// Beta.169: Application Launcher Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct LauncherRecipe;

#[derive(Debug, PartialEq)]
enum LauncherOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl LauncherOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            LauncherOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            LauncherOperation::ListTools
        } else {
            LauncherOperation::Install
        }
    }
}

impl LauncherRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("rofi") || input_lower.contains("dmenu")
            || input_lower.contains("wofi") || input_lower.contains("launcher")
            || input_lower.contains("app menu") || input_lower.contains("application menu");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = LauncherOperation::detect(user_input);
        match operation {
            LauncherOperation::Install => Self::build_install_plan(telemetry),
            LauncherOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            LauncherOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("rofi") { "rofi" }
        else if input_lower.contains("dmenu") { "dmenu" }
        else if input_lower.contains("wofi") { "wofi" }
        else { "rofi" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "rofi" => ("Rofi", "rofi", "Window switcher, application launcher, and dmenu replacement"),
            "dmenu" => ("dmenu", "dmenu", "Dynamic menu for X11 (Suckless)"),
            "wofi" => ("Wofi", "wofi", "Wayland-native launcher (rofi/dmenu for Wayland)"),
            _ => ("Rofi", "rofi", "Window switcher and application launcher"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("launcher.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {}. Launch with `{} -show drun` (or bind to keybinding).", tool_name, description, tool);

        Ok(ActionPlan {
            analysis: format!("Installing {} application launcher", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("launcher_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("launcher.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking application launcher tools".to_string(),
            goals: vec!["List installed launchers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-launchers".to_string(),
                    description: "List launcher tools".to_string(),
                    command: "pacman -Q rofi dmenu wofi 2>/dev/null || echo 'No launcher tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed application launcher tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("launcher_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("launcher.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available application launcher tools".to_string(),
            goals: vec!["List available launchers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Application Launchers:\n- Rofi (official) - Window switcher, app launcher, dmenu replacement\n- dmenu (official) - Suckless dynamic menu for X11\n- Wofi (official) - Wayland-native launcher (rofi/dmenu for Wayland)\n- Albert (AUR) - Fast and flexible keyboard launcher\n- Ulauncher (AUR) - Application launcher for Linux\n- KRunner (official) - KDE Plasma launcher (part of plasma-workspace)\n\nUsage:\n- rofi: rofi -show drun (bind to Super+Space)\n- dmenu: dmenu_run (bind to keybinding)\n- wofi: wofi --show drun'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Application launchers for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("launcher_list_tools".to_string()),
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
        assert!(LauncherRecipe::matches_request("install rofi"));
        assert!(LauncherRecipe::matches_request("install dmenu"));
        assert!(LauncherRecipe::matches_request("install application launcher"));
        assert!(!LauncherRecipe::matches_request("what is rofi"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install wofi".to_string());
        let plan = LauncherRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
