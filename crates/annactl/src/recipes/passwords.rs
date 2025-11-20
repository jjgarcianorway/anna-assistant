// Beta.170: Password Manager Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct PasswordsRecipe;

#[derive(Debug, PartialEq)]
enum PasswordsOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl PasswordsOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            PasswordsOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            PasswordsOperation::ListTools
        } else {
            PasswordsOperation::Install
        }
    }
}

impl PasswordsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("keepassxc") || input_lower.contains("keepass")
            || input_lower.contains("bitwarden") || input_lower.contains("pass ")
            || input_lower.contains("password manager") || input_lower.contains("password store")
            || input_lower.contains("1password");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = PasswordsOperation::detect(user_input);
        match operation {
            PasswordsOperation::Install => Self::build_install_plan(telemetry),
            PasswordsOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            PasswordsOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("keepassxc") || input_lower.contains("keepass") { "keepassxc" }
        else if input_lower.contains("bitwarden") { "bitwarden-cli" }
        else if input_lower.contains("pass") { "pass" }
        else if input_lower.contains("1password") { "1password" }
        else { "keepassxc" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description, is_aur) = match tool {
            "keepassxc" => ("KeePassXC", "keepassxc", "Cross-platform password manager with browser integration", false),
            "bitwarden-cli" => ("Bitwarden CLI", "bitwarden-cli", "Command-line client for Bitwarden password manager", false),
            "pass" => ("Pass", "pass", "Unix password manager using GPG encryption", false),
            "1password" => ("1Password", "1password", "Proprietary password manager with sync", true),
            _ => ("KeePassXC", "keepassxc", "Cross-platform password manager", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("passwords.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = format!("{} installed. {}. Launch from app menu or run in terminal.", tool_name, description);

        let risk_level = if is_aur { RiskLevel::Medium } else { RiskLevel::Low };
        let requires_confirmation = is_aur;

        Ok(ActionPlan {
            analysis: format!("Installing {} password manager", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: if is_aur {
                        format!("yay -Rns --noconfirm {} || paru -Rns --noconfirm {}", package_name, package_name)
                    } else {
                        format!("sudo pacman -Rns --noconfirm {}", package_name)
                    },
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("passwords_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("passwords.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking password manager tools".to_string(),
            goals: vec!["List installed password managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-password-tools".to_string(),
                    description: "List password managers".to_string(),
                    command: "pacman -Q keepassxc bitwarden-cli pass 2>/dev/null || echo 'No password managers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed password manager tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("passwords_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("passwords.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available password manager tools".to_string(),
            goals: vec!["List available password managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Password Managers:

GUI Applications:
- KeePassXC (official) - Cross-platform password manager, offline database with browser integration
- Bitwarden Desktop (AUR) - Open-source password manager with cloud sync
- 1Password (AUR) - Proprietary password manager with excellent UI
- Enpass (AUR) - Cross-platform password manager with cloud sync

Command-Line:
- pass (official) - Unix password manager using GPG and git
- Bitwarden CLI (official) - Command-line client for Bitwarden
- gopass (official) - Enhanced password store compatible with pass
- passage (AUR) - Password manager using age encryption

Browser Extensions:
- KeePassXC Browser - Native messaging with browser (Firefox, Chrome)
- Bitwarden Browser Extension - Official browser extension
- 1Password Browser Extension - Browser integration

Security Notes:
- KeePassXC: Local database, no cloud sync (sync manually via Syncthing/cloud)
- Bitwarden: Open source, self-hostable, cloud sync
- pass: GPG-encrypted files, git integration for sync
- 1Password: Proprietary, requires subscription

Usage:
- KeePassXC: Launch from app menu, create or open database
- pass: Initialize with pass init gpg-key-id, use pass insert/show/generate
- Bitwarden CLI: Login with bw login, sync and access vault'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Password managers for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("passwords_list_tools".to_string()),
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
        assert!(PasswordsRecipe::matches_request("install keepassxc"));
        assert!(PasswordsRecipe::matches_request("install password manager"));
        assert!(PasswordsRecipe::matches_request("setup bitwarden"));
        assert!(!PasswordsRecipe::matches_request("what is keepassxc"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install pass".to_string());
        let plan = PasswordsRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
