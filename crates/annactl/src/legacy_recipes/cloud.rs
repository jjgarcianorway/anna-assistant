// Beta.166: Cloud Storage Clients Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct CloudRecipe;

#[derive(Debug, PartialEq)]
enum CloudOperation {
    Install,
    CheckStatus,
    ListClients,
}

impl CloudOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            CloudOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            CloudOperation::ListClients
        } else {
            CloudOperation::Install
        }
    }
}

impl CloudRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("nextcloud") || input_lower.contains("dropbox")
            || input_lower.contains("cloud storage") || input_lower.contains("cloud sync");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = CloudOperation::detect(user_input);
        match operation {
            CloudOperation::Install => Self::build_install_plan(telemetry),
            CloudOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            CloudOperation::ListClients => Self::build_list_clients_plan(telemetry),
        }
    }

    fn detect_client(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("nextcloud") { "nextcloud" }
        else if input_lower.contains("dropbox") { "dropbox" }
        else { "nextcloud" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let client = Self::detect_client(user_input);
        let (client_name, package_name, is_aur) = match client {
            "nextcloud" => ("Nextcloud Desktop", "nextcloud-client", false),
            "dropbox" => ("Dropbox", "dropbox", true),
            _ => ("Nextcloud Desktop", "nextcloud-client", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("cloud.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("client".to_string(), serde_json::json!(client_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = match client {
            "nextcloud" => "Nextcloud client installed. Launch from application menu to configure server.".to_string(),
            "dropbox" => "Dropbox installed. Run 'dropbox' to start initial setup.".to_string(),
            _ => format!("{} installed. Launch from application menu.", client_name),
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} cloud storage client", client_name),
            goals: vec![format!("Install {}", client_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", client),
                    description: format!("Install {}", client_name),
                    command: install_cmd,
                    risk_level: if is_aur { RiskLevel::Medium } else { RiskLevel::Low },
                    rollback_id: Some(format!("remove-{}", client)),
                    requires_confirmation: is_aur,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", client),
                    description: format!("Remove {}", client_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("cloud_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("cloud.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking cloud storage clients".to_string(),
            goals: vec!["List installed cloud clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-cloud-clients".to_string(),
                    description: "List cloud clients".to_string(),
                    command: "pacman -Q nextcloud-client dropbox 2>/dev/null || echo 'No cloud clients installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed cloud storage clients".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("cloud_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_clients_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("cloud.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListClients"));

        Ok(ActionPlan {
            analysis: "Showing available cloud clients".to_string(),
            goals: vec!["List available cloud storage clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-clients".to_string(),
                    description: "Show available clients".to_string(),
                    command: r"echo 'Available:\n- Nextcloud (official) - Self-hosted cloud storage\n- Dropbox (AUR) - Commercial cloud storage\n- Mega (AUR) - Cloud storage with encryption\n- OneDrive (AUR) - Microsoft cloud storage'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Cloud storage clients for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("cloud_list_clients".to_string()),
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
        assert!(CloudRecipe::matches_request("install nextcloud"));
        assert!(CloudRecipe::matches_request("setup dropbox"));
        assert!(CloudRecipe::matches_request("install cloud storage"));
        assert!(!CloudRecipe::matches_request("what is nextcloud"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install nextcloud".to_string());
        let plan = CloudRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
