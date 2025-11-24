// Beta.167: File Managers Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct FileManagerRecipe;

#[derive(Debug, PartialEq)]
enum FileManagerOperation {
    Install,
    CheckStatus,
    ListManagers,
}

impl FileManagerOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            FileManagerOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            FileManagerOperation::ListManagers
        } else {
            FileManagerOperation::Install
        }
    }
}

impl FileManagerRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("nautilus") || input_lower.contains("dolphin")
            || input_lower.contains("thunar") || input_lower.contains("ranger")
            || input_lower.contains("file manager") || input_lower.contains("file browser");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = FileManagerOperation::detect(user_input);
        match operation {
            FileManagerOperation::Install => Self::build_install_plan(telemetry),
            FileManagerOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            FileManagerOperation::ListManagers => Self::build_list_managers_plan(telemetry),
        }
    }

    fn detect_manager(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("nautilus") { "nautilus" }
        else if input_lower.contains("dolphin") { "dolphin" }
        else if input_lower.contains("thunar") { "thunar" }
        else if input_lower.contains("ranger") { "ranger" }
        else { "thunar" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let manager = Self::detect_manager(user_input);
        let (manager_name, package_name, is_gui) = match manager {
            "nautilus" => ("Nautilus", "nautilus", true),
            "dolphin" => ("Dolphin", "dolphin", true),
            "thunar" => ("Thunar", "thunar", true),
            "ranger" => ("ranger", "ranger", false),
            _ => ("Thunar", "thunar", true),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("file_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("manager".to_string(), serde_json::json!(manager_name));

        let notes = if is_gui {
            format!("{} installed. Launch from application menu or run: {}", manager_name, package_name)
        } else {
            format!("{} installed. Terminal file manager. Launch with: {}", manager_name, package_name)
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} file manager", manager_name),
            goals: vec![format!("Install {}", manager_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", manager),
                    description: format!("Install {}", manager_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", manager)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", manager),
                    description: format!("Remove {}", manager_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("file_manager_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("file_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking file managers".to_string(),
            goals: vec!["List installed file managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-file-managers".to_string(),
                    description: "List file managers".to_string(),
                    command: "pacman -Q nautilus dolphin thunar ranger nemo pcmanfm 2>/dev/null || echo 'No file managers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed file managers".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("file_manager_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_managers_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("file_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListManagers"));

        Ok(ActionPlan {
            analysis: "Showing available file managers".to_string(),
            goals: vec!["List available file managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-managers".to_string(),
                    description: "Show available managers".to_string(),
                    command: r"echo 'GUI File Managers:\n- Nautilus (official) - GNOME default\n- Dolphin (official) - KDE default\n- Thunar (official) - XFCE default\n- Nemo (official) - Cinnamon fork of Nautilus\n- PCManFM (official) - Lightweight\n\nTerminal File Managers:\n- ranger (official) - Vi-like keybindings\n- nnn (official) - Minimal and fast\n- lf (official) - Inspired by ranger'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "File managers for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("file_manager_list_managers".to_string()),
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
        assert!(FileManagerRecipe::matches_request("install nautilus"));
        assert!(FileManagerRecipe::matches_request("install dolphin"));
        assert!(FileManagerRecipe::matches_request("install file manager"));
        assert!(!FileManagerRecipe::matches_request("what is nautilus"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install thunar".to_string());
        let plan = FileManagerRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
