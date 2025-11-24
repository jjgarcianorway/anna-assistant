// Beta.165: Display Managers Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct DisplayManagerRecipe;

#[derive(Debug, PartialEq)]
enum DisplayManagerOperation {
    Install,
    CheckStatus,
    Switch,
    ListManagers,
}

impl DisplayManagerOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("switch") || input_lower.contains("change to") {
            DisplayManagerOperation::Switch
        } else if input_lower.contains("check") || input_lower.contains("status") {
            DisplayManagerOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            DisplayManagerOperation::ListManagers
        } else {
            DisplayManagerOperation::Install
        }
    }
}

impl DisplayManagerRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("sddm") || input_lower.contains("gdm")
            || input_lower.contains("lightdm") || input_lower.contains("display manager")
            || input_lower.contains("login manager") || input_lower.contains("login screen");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("switch") || input_lower.contains("enable")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = DisplayManagerOperation::detect(user_input);
        match operation {
            DisplayManagerOperation::Install => Self::build_install_plan(telemetry),
            DisplayManagerOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            DisplayManagerOperation::Switch => Self::build_switch_plan(telemetry),
            DisplayManagerOperation::ListManagers => Self::build_list_managers_plan(telemetry),
        }
    }

    fn detect_manager(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("sddm") { "sddm" }
        else if input_lower.contains("gdm") { "gdm" }
        else if input_lower.contains("lightdm") { "lightdm" }
        else { "lightdm" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let manager = Self::detect_manager(user_input);
        let (manager_name, package_name, extra_packages, notes) = match manager {
            "sddm" => (
                "SDDM",
                "sddm",
                vec!["qt5-graphicaleffects", "qt5-quickcontrols2", "qt5-svg"],
                "SDDM installed. Enable: sudo systemctl enable sddm. Reboot to use."
            ),
            "gdm" => (
                "GDM",
                "gdm",
                vec![],
                "GDM installed. Enable: sudo systemctl enable gdm. Reboot to use."
            ),
            "lightdm" => (
                "LightDM",
                "lightdm",
                vec!["lightdm-gtk-greeter"],
                "LightDM installed. Enable: sudo systemctl enable lightdm. Reboot to use."
            ),
            _ => (
                "LightDM",
                "lightdm",
                vec!["lightdm-gtk-greeter"],
                "LightDM installed. Enable: sudo systemctl enable lightdm. Reboot to use."
            ),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("display_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("manager".to_string(), serde_json::json!(manager_name));

        let mut all_packages = vec![package_name.to_string()];
        all_packages.extend(extra_packages.iter().map(|s| s.to_string()));
        let packages_str = all_packages.join(" ");

        Ok(ActionPlan {
            analysis: format!("Installing {} display manager", manager_name),
            goals: vec![format!("Install {}", manager_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", manager),
                    description: format!("Install {}", manager_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", packages_str),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", manager)),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: format!("enable-{}", manager),
                    description: format!("Enable {}", manager_name),
                    command: format!("sudo systemctl enable {}", package_name),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some(format!("disable-{}", manager)),
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("disable-{}", manager),
                    description: format!("Disable {}", manager_name),
                    command: format!("sudo systemctl disable {}", package_name),
                },
                RollbackStep {
                    id: format!("remove-{}", manager),
                    description: format!("Remove {}", manager_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes.to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("display_manager_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("display_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking display managers".to_string(),
            goals: vec!["List installed display managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-managers".to_string(),
                    description: "List installed managers".to_string(),
                    command: "pacman -Q sddm gdm lightdm 2>/dev/null || echo 'No display managers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-active".to_string(),
                    description: "Check active manager".to_string(),
                    command: "systemctl list-units --type=service --state=active | grep -E '(sddm|gdm|lightdm)' || echo 'No active display manager'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed display managers and which one is active".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("display_manager_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_switch_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let new_manager = Self::detect_manager(user_input);
        let manager_name = match new_manager {
            "sddm" => "SDDM",
            "gdm" => "GDM",
            "lightdm" => "LightDM",
            _ => "LightDM",
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("display_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Switch"));
        meta_other.insert("target_manager".to_string(), serde_json::json!(manager_name));

        Ok(ActionPlan {
            analysis: format!("Switching to {} display manager", manager_name),
            goals: vec![
                "Disable current display managers".to_string(),
                format!("Enable {}", manager_name),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-installed".to_string(),
                    description: format!("Verify {} is installed", manager_name),
                    command: format!("pacman -Q {}", new_manager),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "disable-all".to_string(),
                    description: "Disable all display managers".to_string(),
                    command: "sudo systemctl disable sddm gdm lightdm 2>/dev/null || true".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: None,
                    requires_confirmation: true,
                },
                CommandStep {
                    id: format!("enable-{}", new_manager),
                    description: format!("Enable {}", manager_name),
                    command: format!("sudo systemctl enable {}", new_manager),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some(format!("disable-{}", new_manager)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("disable-{}", new_manager),
                    description: format!("Disable {}", manager_name),
                    command: format!("sudo systemctl disable {}", new_manager),
                },
            ],
            notes_for_user: format!("{} enabled. Reboot to use the new display manager.", manager_name),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("display_manager_switch".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_managers_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("display_manager.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListManagers"));

        Ok(ActionPlan {
            analysis: "Showing available display managers".to_string(),
            goals: vec!["List available display managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-managers".to_string(),
                    description: "Show available managers".to_string(),
                    command: r"echo 'Available:\n- SDDM (official) - KDE default, Qt-based\n- GDM (official) - GNOME default, GTK-based\n- LightDM (official) - Lightweight, cross-desktop'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Display managers (login screens) for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("display_manager_list_managers".to_string()),
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
        assert!(DisplayManagerRecipe::matches_request("install sddm"));
        assert!(DisplayManagerRecipe::matches_request("switch to gdm"));
        assert!(DisplayManagerRecipe::matches_request("install display manager"));
        assert!(!DisplayManagerRecipe::matches_request("what is sddm"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install lightdm".to_string());
        let plan = DisplayManagerRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
