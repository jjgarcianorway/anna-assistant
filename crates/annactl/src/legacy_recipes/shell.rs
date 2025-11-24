// Beta.159: Shell Environments Recipe
// Handles installation of alternative shells (zsh, fish) and frameworks (oh-my-zsh)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ShellRecipe;

#[derive(Debug, PartialEq)]
enum ShellOperation {
    Install,         // Install shell
    CheckStatus,     // Verify installation
    InstallOhMyZsh,  // Install oh-my-zsh framework
    SetDefault,      // Set default shell
}

impl ShellOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        
        if input_lower.contains("check") || input_lower.contains("status") {
            return ShellOperation::CheckStatus;
        }
        
        if input_lower.contains("oh-my-zsh") || input_lower.contains("ohmyzsh") {
            return ShellOperation::InstallOhMyZsh;
        }
        
        if input_lower.contains("default") || input_lower.contains("set as") {
            return ShellOperation::SetDefault;
        }
        
        ShellOperation::Install
    }
}

impl ShellRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        
        let has_shell_context = input_lower.contains("zsh")
            || input_lower.contains("fish")
            || input_lower.contains("oh-my-zsh")
            || input_lower.contains("ohmyzsh")
            || input_lower.contains("shell");
        
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("default")
            || input_lower.contains("set");
        
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;
        
        has_shell_context && has_action && !is_info_only
    }
    
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ShellOperation::detect(user_input);
        
        match operation {
            ShellOperation::Install => Self::build_install_plan(telemetry),
            ShellOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ShellOperation::InstallOhMyZsh => Self::build_install_omz_plan(telemetry),
            ShellOperation::SetDefault => Self::build_set_default_plan(telemetry),
        }
    }
    
    fn detect_shell(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("fish") { "fish" }
        else { "zsh" }
    }
    
    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let shell = Self::detect_shell(user_input);
        
        let (shell_name, package_name) = match shell {
            "fish" => ("Fish", "fish"),
            _ => ("Zsh", "zsh"),
        };
        
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("shell.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("shell".to_string(), serde_json::json!(shell_name));
        
        Ok(ActionPlan {
            analysis: format!("Installing {} shell", shell_name),
            goals: vec![format!("Install {}", shell_name)],
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
                    id: format!("install-{}", shell),
                    description: format!("Install {}", shell_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", shell)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", shell),
                    description: format!("Remove {}", shell_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: format!("{} installed. To set as default: chsh -s /usr/bin/{}", shell_name, shell),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("shell_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
    
    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("shell.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));
        
        Ok(ActionPlan {
            analysis: "Checking shell installations".to_string(),
            goals: vec!["List installed shells".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-shells".to_string(),
                    description: "List installed shells".to_string(),
                    command: "pacman -Q zsh fish 2>/dev/null && echo '\nCurrent shell:' && echo $SHELL".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed shells and current default".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("shell_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
    
    fn build_install_omz_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("shell.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("InstallOhMyZsh"));
        
        Ok(ActionPlan {
            analysis: "Installing oh-my-zsh framework".to_string(),
            goals: vec!["Install oh-my-zsh".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-zsh".to_string(),
                    description: "Verify zsh is installed".to_string(),
                    command: "which zsh".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "show-omz-instructions".to_string(),
                    description: "Show oh-my-zsh installation instructions".to_string(),
                    command: "echo 'To install oh-my-zsh, run:\n\nsh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\"\n\nNote: Anna cannot run this automatically (requires user interaction).'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "oh-my-zsh requires manual installation due to interactive prompts. Run the command shown above.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("shell_install_omz".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
    
    fn build_set_default_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("shell.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("SetDefault"));
        
        Ok(ActionPlan {
            analysis: "Instructions for setting default shell".to_string(),
            goals: vec!["Explain how to change default shell".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-chsh-instructions".to_string(),
                    description: "Show chsh instructions".to_string(),
                    command: "echo 'To set default shell:\n\nFor Zsh: chsh -s /usr/bin/zsh\nFor Fish: chsh -s /usr/bin/fish\n\nNote: Requires user password. Log out and back in after changing.'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Run chsh manually to change your default shell. Requires logout/login to take effect.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("shell_set_default".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matches_shell_keywords() {
        assert!(ShellRecipe::matches_request("install zsh"));
        assert!(ShellRecipe::matches_request("install fish"));
        assert!(ShellRecipe::matches_request("install oh-my-zsh"));
    }
    
    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!ShellRecipe::matches_request("what is zsh"));
    }
    
    #[test]
    fn test_operation_detection() {
        assert_eq!(ShellOperation::detect("install zsh"), ShellOperation::Install);
        assert_eq!(ShellOperation::detect("check shell status"), ShellOperation::CheckStatus);
    }
    
    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install zsh".to_string());
        let plan = ShellRecipe::build_install_plan(&telemetry).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
