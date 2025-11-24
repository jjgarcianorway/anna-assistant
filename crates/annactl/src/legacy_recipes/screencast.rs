// Beta.168: Screen Recording Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ScreencastRecipe;

#[derive(Debug, PartialEq)]
enum ScreencastOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl ScreencastOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            ScreencastOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            ScreencastOperation::ListTools
        } else {
            ScreencastOperation::Install
        }
    }
}

impl ScreencastRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("obs") || input_lower.contains("simplescreenrecorder")
            || input_lower.contains("peek") || input_lower.contains("screen record")
            || input_lower.contains("screencast") || input_lower.contains("video capture");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ScreencastOperation::detect(user_input);
        match operation {
            ScreencastOperation::Install => Self::build_install_plan(telemetry),
            ScreencastOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ScreencastOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("obs") { "obs" }
        else if input_lower.contains("simplescreenrecorder") || input_lower.contains("ssr") { "simplescreenrecorder" }
        else if input_lower.contains("peek") { "peek" }
        else { "obs" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "obs" => ("OBS Studio", "obs-studio", "Professional streaming and recording"),
            "simplescreenrecorder" => ("SimpleScreenRecorder", "simplescreenrecorder", "Easy-to-use screen recorder"),
            "peek" => ("Peek", "peek", "Animated GIF recorder"),
            _ => ("OBS Studio", "obs-studio", "Professional streaming and recording"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("screencast.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {}. Launch from application menu.", tool_name, description);

        Ok(ActionPlan {
            analysis: format!("Installing {} screen recording tool", tool_name),
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
                template_used: Some("screencast_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("screencast.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking screen recording tools".to_string(),
            goals: vec!["List installed screen recording tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-screencast-tools".to_string(),
                    description: "List screen recording tools".to_string(),
                    command: "pacman -Q obs-studio simplescreenrecorder peek 2>/dev/null || echo 'No screen recording tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed screen recording tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("screencast_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("screencast.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available screen recording tools".to_string(),
            goals: vec!["List available screen recording tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Screen Recording Tools:\n- OBS Studio (official) - Professional streaming/recording, supports plugins\n- SimpleScreenRecorder (official) - Easy-to-use, good performance\n- Peek (official) - Animated GIF recorder for quick demos\n- Kazam (official) - Simple screencasting program\n- wf-recorder (official) - Wayland screen recorder\n- FFmpeg (official) - Command-line recording (advanced)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Screen recording tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("screencast_list_tools".to_string()),
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
        assert!(ScreencastRecipe::matches_request("install obs"));
        assert!(ScreencastRecipe::matches_request("install simplescreenrecorder"));
        assert!(ScreencastRecipe::matches_request("install screen recording tool"));
        assert!(!ScreencastRecipe::matches_request("what is obs"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install peek".to_string());
        let plan = ScreencastRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
