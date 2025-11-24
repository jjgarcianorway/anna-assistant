// Beta.167: Archive Managers Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ArchiveManagerRecipe;

#[derive(Debug, PartialEq)]
enum ArchiveManagerOperation {
    Install,
    CheckStatus,
    ListManagers,
}

impl ArchiveManagerOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            ArchiveManagerOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            ArchiveManagerOperation::ListManagers
        } else {
            ArchiveManagerOperation::Install
        }
    }
}

impl ArchiveManagerRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("file-roller") || input_lower.contains("ark")
            || input_lower.contains("xarchiver") || input_lower.contains("archive manager")
            || input_lower.contains("zip manager") || input_lower.contains("compression manager");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ArchiveManagerOperation::detect(user_input);
        match operation {
            ArchiveManagerOperation::Install => Self::build_install_plan(telemetry),
            ArchiveManagerOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ArchiveManagerOperation::ListManagers => Self::build_list_managers_plan(telemetry),
        }
    }

    fn detect_manager(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("file-roller") || input_lower.contains("file roller") { "file-roller" }
        else if input_lower.contains("ark") { "ark" }
        else if input_lower.contains("xarchiver") { "xarchiver" }
        else { "file-roller" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let manager = Self::detect_manager(user_input);
        let (manager_name, package_name, desktop_env) = match manager {
            "file-roller" => ("File Roller", "file-roller", "GNOME"),
            "ark" => ("Ark", "ark", "KDE"),
            "xarchiver" => ("Xarchiver", "xarchiver", "GTK"),
            _ => ("File Roller", "file-roller", "GNOME"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("archive_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("manager".to_string(), serde_json::json!(manager_name));

        let notes = format!("{} installed. {} archive manager. Launch from application menu.", manager_name, desktop_env);

        Ok(ActionPlan {
            analysis: format!("Installing {} archive manager", manager_name),
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
                template_used: Some("archive_manager_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("archive_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking archive managers".to_string(),
            goals: vec!["List installed archive managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-archive-managers".to_string(),
                    description: "List archive managers".to_string(),
                    command: "pacman -Q file-roller ark xarchiver engrampa 2>/dev/null || echo 'No archive managers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed archive managers".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("archive_manager_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_managers_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("archive_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListManagers"));

        Ok(ActionPlan {
            analysis: "Showing available archive managers".to_string(),
            goals: vec!["List available archive managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-managers".to_string(),
                    description: "Show available managers".to_string(),
                    command: r"echo 'Available:\n- File Roller (official) - GNOME archive manager\n- Ark (official) - KDE archive manager\n- Xarchiver (official) - GTK lightweight\n- Engrampa (official) - MATE archive manager\n- PeaZip (AUR) - Cross-platform with encryption'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Archive managers (zip, tar, etc.) for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("archive_manager_list_managers".to_string()),
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
        assert!(ArchiveManagerRecipe::matches_request("install file-roller"));
        assert!(ArchiveManagerRecipe::matches_request("install ark"));
        assert!(ArchiveManagerRecipe::matches_request("install archive manager"));
        assert!(!ArchiveManagerRecipe::matches_request("what is file-roller"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install ark".to_string());
        let plan = ArchiveManagerRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
