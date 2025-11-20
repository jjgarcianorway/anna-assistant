// Beta.160: File Sync Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct SyncRecipe;

#[derive(Debug, PartialEq)]
enum SyncOperation {
    Install,
    CheckStatus,
    EnableSyncthing,
    ListTools,
}

impl SyncOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("enable") || input_lower.contains("start") {
            SyncOperation::EnableSyncthing
        } else if input_lower.contains("check") || input_lower.contains("status") {
            SyncOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            SyncOperation::ListTools
        } else {
            SyncOperation::Install
        }
    }
}

impl SyncRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("syncthing") || input_lower.contains("rclone")
            || input_lower.contains("sync tool") || input_lower.contains("file sync")
            || input_lower.contains("synchronize") || input_lower.contains("cloud sync");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("enable") || input_lower.contains("start");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = SyncOperation::detect(user_input);
        match operation {
            SyncOperation::Install => Self::build_install_plan(telemetry),
            SyncOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            SyncOperation::EnableSyncthing => Self::build_enable_syncthing_plan(telemetry),
            SyncOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("syncthing") { "syncthing" }
        else if input_lower.contains("rclone") { "rclone" }
        else { "syncthing" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, is_aur) = match tool {
            "syncthing" => ("Syncthing", "syncthing", false),
            "rclone" => ("rclone", "rclone", false),
            _ => ("Syncthing", "syncthing", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("sync.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = if tool == "syncthing" {
            "Syncthing installed. Enable with: sudo systemctl enable --now syncthing@$USER".to_string()
        } else {
            format!("{} installed. Configure with your cloud provider credentials.", tool_name)
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} file sync tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
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
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
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
                template_used: Some("sync_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("sync.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking file sync tools".to_string(),
            goals: vec!["List installed sync tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-sync-tools".to_string(),
                    description: "List sync tools".to_string(),
                    command: "pacman -Q syncthing rclone 2>/dev/null || echo 'No sync tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed file sync tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("sync_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_enable_syncthing_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("sync.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("EnableSyncthing"));

        Ok(ActionPlan {
            analysis: "Enabling Syncthing service".to_string(),
            goals: vec!["Enable and start Syncthing".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-syncthing".to_string(),
                    description: "Verify Syncthing is installed".to_string(),
                    command: "pacman -Q syncthing".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "enable-syncthing".to_string(),
                    description: "Enable Syncthing service".to_string(),
                    command: "systemctl enable --now --user syncthing".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("disable-syncthing".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "disable-syncthing".to_string(),
                    description: "Disable Syncthing service".to_string(),
                    command: "systemctl disable --now --user syncthing".to_string(),
                },
            ],
            notes_for_user: "Syncthing enabled. Access web UI at http://localhost:8384".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("sync_enable_syncthing".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("sync.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available file sync tools".to_string(),
            goals: vec!["List available sync tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-sync-tools".to_string(),
                    description: "Show available sync tools".to_string(),
                    command: "echo 'Available:\\n- Syncthing (official) - P2P file sync\\n- rclone (official) - Cloud storage sync'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "File synchronization tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("sync_list_tools".to_string()),
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
        assert!(SyncRecipe::matches_request("install syncthing"));
        assert!(SyncRecipe::matches_request("setup rclone"));
        assert!(SyncRecipe::matches_request("enable syncthing"));
        assert!(!SyncRecipe::matches_request("what is syncthing"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install syncthing".to_string());
        let plan = SyncRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
