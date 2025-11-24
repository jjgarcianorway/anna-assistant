// Beta.168: Screenshot Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ScreenshotRecipe;

#[derive(Debug, PartialEq)]
enum ScreenshotOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl ScreenshotOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            ScreenshotOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            ScreenshotOperation::ListTools
        } else {
            ScreenshotOperation::Install
        }
    }
}

impl ScreenshotRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("flameshot") || input_lower.contains("spectacle")
            || input_lower.contains("scrot") || input_lower.contains("screenshot")
            || input_lower.contains("screen capture") || input_lower.contains("gnome-screenshot");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ScreenshotOperation::detect(user_input);
        match operation {
            ScreenshotOperation::Install => Self::build_install_plan(telemetry),
            ScreenshotOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ScreenshotOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("flameshot") { "flameshot" }
        else if input_lower.contains("spectacle") { "spectacle" }
        else if input_lower.contains("scrot") { "scrot" }
        else if input_lower.contains("gnome-screenshot") { "gnome-screenshot" }
        else { "flameshot" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, desktop_env) = match tool {
            "flameshot" => ("Flameshot", "flameshot", "Cross-platform"),
            "spectacle" => ("Spectacle", "spectacle", "KDE"),
            "scrot" => ("scrot", "scrot", "Command-line"),
            "gnome-screenshot" => ("GNOME Screenshot", "gnome-screenshot", "GNOME"),
            _ => ("Flameshot", "flameshot", "Cross-platform"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("screenshot.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {} screenshot tool. Launch from application menu or use keyboard shortcut.", tool_name, desktop_env);

        Ok(ActionPlan {
            analysis: format!("Installing {} screenshot tool", tool_name),
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
                template_used: Some("screenshot_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("screenshot.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking screenshot tools".to_string(),
            goals: vec!["List installed screenshot tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-screenshot-tools".to_string(),
                    description: "List screenshot tools".to_string(),
                    command: "pacman -Q flameshot spectacle scrot gnome-screenshot 2>/dev/null || echo 'No screenshot tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed screenshot tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("screenshot_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("screenshot.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available screenshot tools".to_string(),
            goals: vec!["List available screenshot tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'GUI Screenshot Tools:\n- Flameshot (official) - Feature-rich, cross-platform\n- Spectacle (official) - KDE screenshot utility\n- GNOME Screenshot (official) - GNOME default tool\n- Ksnip (official) - Qt-based screenshot tool\n\nCommand-line Tools:\n- scrot (official) - Minimalist screenshot utility\n- maim (official) - Modern screenshot tool'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Screenshot tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("screenshot_list_tools".to_string()),
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
        assert!(ScreenshotRecipe::matches_request("install flameshot"));
        assert!(ScreenshotRecipe::matches_request("install spectacle"));
        assert!(ScreenshotRecipe::matches_request("install screenshot tool"));
        assert!(!ScreenshotRecipe::matches_request("what is flameshot"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install scrot".to_string());
        let plan = ScreenshotRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
