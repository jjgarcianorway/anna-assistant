// Beta.165: Video Editing & Graphics Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct VideoRecipe;

#[derive(Debug, PartialEq)]
enum VideoOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl VideoOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            VideoOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            VideoOperation::ListTools
        } else {
            VideoOperation::Install
        }
    }
}

impl VideoRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("kdenlive") || input_lower.contains("openshot")
            || input_lower.contains("blender") || input_lower.contains("video edit")
            || input_lower.contains("video editor") || input_lower.contains("3d modeling");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = VideoOperation::detect(user_input);
        match operation {
            VideoOperation::Install => Self::build_install_plan(telemetry),
            VideoOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            VideoOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("kdenlive") { "kdenlive" }
        else if input_lower.contains("openshot") { "openshot" }
        else if input_lower.contains("blender") { "blender" }
        else { "kdenlive" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, extra_packages) = match tool {
            "kdenlive" => ("Kdenlive", "kdenlive", vec!["breeze", "breeze-icons"]),
            "openshot" => ("OpenShot", "openshot", vec![]),
            "blender" => ("Blender", "blender", vec![]),
            _ => ("Kdenlive", "kdenlive", vec!["breeze", "breeze-icons"]),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("video.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let mut all_packages = vec![package_name.to_string()];
        all_packages.extend(extra_packages.iter().map(|s| s.to_string()));
        let packages_str = all_packages.join(" ");

        let notes = match tool {
            "kdenlive" => "Kdenlive installed. Professional video editor with MLT framework. Launch from application menu.".to_string(),
            "openshot" => "OpenShot installed. User-friendly video editor. Launch from application menu.".to_string(),
            "blender" => "Blender installed. 3D modeling, animation, video editing suite. Launch from application menu.".to_string(),
            _ => format!("{} installed. Launch from application menu.", tool_name),
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} video tool", tool_name),
            goals: vec![format!("Install {} and dependencies", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", packages_str),
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
                template_used: Some("video_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("video.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking video editing tools".to_string(),
            goals: vec!["List installed video tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-video-tools".to_string(),
                    description: "List video tools".to_string(),
                    command: "pacman -Q kdenlive openshot blender 2>/dev/null || echo 'No video tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed video editing and graphics tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("video_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("video.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available video tools".to_string(),
            goals: vec!["List available video tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Available:\n- Kdenlive (official) - Professional video editor\n- OpenShot (official) - User-friendly video editor\n- Blender (official) - 3D modeling and video suite\n- DaVinci Resolve (AUR) - Professional color grading'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Video editing and graphics tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("video_list_tools".to_string()),
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
        assert!(VideoRecipe::matches_request("install kdenlive"));
        assert!(VideoRecipe::matches_request("install blender"));
        assert!(VideoRecipe::matches_request("install video editor"));
        assert!(!VideoRecipe::matches_request("what is kdenlive"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install kdenlive".to_string());
        let plan = VideoRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
