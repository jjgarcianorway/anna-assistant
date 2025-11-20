// Beta.161: Gaming Platform Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct GamingRecipe;

#[derive(Debug, PartialEq)]
enum GamingOperation {
    Install,
    CheckStatus,
    EnableMultilib,
    ListPlatforms,
}

impl GamingOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("enable multilib") || input_lower.contains("32-bit") {
            GamingOperation::EnableMultilib
        } else if input_lower.contains("check") || input_lower.contains("status") {
            GamingOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            GamingOperation::ListPlatforms
        } else {
            GamingOperation::Install
        }
    }
}

impl GamingRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("steam") || input_lower.contains("gaming")
            || input_lower.contains("game") || input_lower.contains("proton")
            || input_lower.contains("multilib");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("enable");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = GamingOperation::detect(user_input);
        match operation {
            GamingOperation::Install => Self::build_install_plan(telemetry),
            GamingOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            GamingOperation::EnableMultilib => Self::build_enable_multilib_plan(telemetry),
            GamingOperation::ListPlatforms => Self::build_list_platforms_plan(telemetry),
        }
    }

    fn detect_platform(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("steam") { "steam" }
        else if input_lower.contains("proton") { "steam" } // Proton requires Steam
        else { "steam" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let platform = Self::detect_platform(user_input);

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gaming.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("platform".to_string(), serde_json::json!(platform));

        // Steam requires multilib to be enabled first
        Ok(ActionPlan {
            analysis: "Installing Steam gaming platform".to_string(),
            goals: vec![
                "Enable multilib repository (32-bit support)".to_string(),
                "Install Steam".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-multilib".to_string(),
                    description: "Check if multilib is already enabled".to_string(),
                    command: "grep -q '^\\[multilib\\]' /etc/pacman.conf && echo 'enabled' || echo 'disabled'".to_string(),
                    risk_level: RiskLevel::Info,
                    required: false,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "enable-multilib".to_string(),
                    description: "Enable multilib repository".to_string(),
                    command: "sudo sed -i '/^#\\\\[multilib\\\\]/,/^#Include = \\\\/etc\\\\/pacman.d\\\\/mirrorlist/ s/^#//' /etc/pacman.conf && sudo pacman -Sy".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some("disable-multilib".to_string()),
                    requires_confirmation: true,
                },
                CommandStep {
                    id: "install-steam".to_string(),
                    description: "Install Steam".to_string(),
                    command: "sudo pacman -S --needed --noconfirm steam".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-steam".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-steam".to_string(),
                    description: "Remove Steam".to_string(),
                    command: "sudo pacman -Rns --noconfirm steam".to_string(),
                },
                RollbackStep {
                    id: "disable-multilib".to_string(),
                    description: "Disable multilib repository".to_string(),
                    command: "sudo sed -i '/^\\\\[multilib\\\\]/,/^Include = \\\\/etc\\\\/pacman.d\\\\/mirrorlist/ s/^/#/' /etc/pacman.conf && sudo pacman -Sy".to_string(),
                },
            ],
            notes_for_user: "Steam installed. Proton compatibility layer included automatically. Launch from application menu.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gaming_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gaming.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking gaming platform status".to_string(),
            goals: vec!["List installed gaming platforms".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "check-multilib".to_string(),
                    description: "Check multilib status".to_string(),
                    command: "grep -q '^\\[multilib\\]' /etc/pacman.conf && echo 'Multilib: enabled' || echo 'Multilib: disabled (required for gaming)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "list-platforms".to_string(),
                    description: "List gaming platforms".to_string(),
                    command: "pacman -Q steam 2>/dev/null || echo 'Steam: not installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows gaming platform installation status".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gaming_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_enable_multilib_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gaming.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("EnableMultilib"));

        Ok(ActionPlan {
            analysis: "Enabling multilib repository for 32-bit support".to_string(),
            goals: vec!["Enable multilib in /etc/pacman.conf".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-multilib".to_string(),
                    description: "Check if multilib is already enabled".to_string(),
                    command: "grep -q '^\\[multilib\\]' /etc/pacman.conf && echo 'enabled' || echo 'disabled'".to_string(),
                    risk_level: RiskLevel::Info,
                    required: false,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "enable-multilib".to_string(),
                    description: "Enable multilib repository".to_string(),
                    command: "sudo sed -i '/^#\\\\[multilib\\\\]/,/^#Include = \\\\/etc\\\\/pacman.d\\\\/mirrorlist/ s/^#//' /etc/pacman.conf && sudo pacman -Sy".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some("disable-multilib".to_string()),
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "disable-multilib".to_string(),
                    description: "Disable multilib repository".to_string(),
                    command: "sudo sed -i '/^\\\\[multilib\\\\]/,/^Include = \\\\/etc\\\\/pacman.d\\\\/mirrorlist/ s/^/#/' /etc/pacman.conf && sudo pacman -Sy".to_string(),
                },
            ],
            notes_for_user: "Multilib enabled. You can now install 32-bit packages and gaming platforms.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gaming_enable_multilib".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_platforms_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("gaming.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListPlatforms"));

        Ok(ActionPlan {
            analysis: "Showing available gaming platforms".to_string(),
            goals: vec!["List available platforms".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-platforms".to_string(),
                    description: "Show available platforms".to_string(),
                    command: "echo 'Available:\\n- Steam (official) - Includes Proton for Windows games\\n- Wine (see wine recipe)\\n- Lutris (see wine recipe)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Gaming platforms for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("gaming_list_platforms".to_string()),
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
        assert!(GamingRecipe::matches_request("install steam"));
        assert!(GamingRecipe::matches_request("setup gaming"));
        assert!(GamingRecipe::matches_request("enable multilib"));
        assert!(!GamingRecipe::matches_request("what is steam"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install steam".to_string());
        let plan = GamingRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
