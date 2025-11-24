// Beta.160: Text Editors Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct EditorRecipe;

#[derive(Debug, PartialEq)]
enum EditorOperation {
    Install,
    CheckStatus,
    ListEditors,
}

impl EditorOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            EditorOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            EditorOperation::ListEditors
        } else {
            EditorOperation::Install
        }
    }
}

impl EditorRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("vscode") || input_lower.contains("vs code")
            || input_lower.contains("code editor") || input_lower.contains("sublime")
            || input_lower.contains("text editor") || input_lower.contains("neovim")
            || input_lower.contains("nvim");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = EditorOperation::detect(user_input);
        match operation {
            EditorOperation::Install => Self::build_install_plan(telemetry),
            EditorOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            EditorOperation::ListEditors => Self::build_list_editors_plan(telemetry),
        }
    }

    fn detect_editor(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("vscode") || input_lower.contains("vs code") { "vscode" }
        else if input_lower.contains("sublime") { "sublime" }
        else if input_lower.contains("neovim") || input_lower.contains("nvim") { "neovim" }
        else { "neovim" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let editor = Self::detect_editor(user_input);
        let (editor_name, package_name, is_aur) = match editor {
            "vscode" => ("VS Code", "visual-studio-code-bin", true),
            "sublime" => ("Sublime Text", "sublime-text-4", true),
            "neovim" => ("Neovim", "neovim", false),
            _ => ("Neovim", "neovim", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("editor.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("editor".to_string(), serde_json::json!(editor_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} text editor", editor_name),
            goals: vec![format!("Install {}", editor_name)],
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
                    id: format!("install-{}", editor),
                    description: format!("Install {}", editor_name),
                    command: install_cmd,
                    risk_level: if is_aur { RiskLevel::Medium } else { RiskLevel::Low },
                    rollback_id: Some(format!("remove-{}", editor)),
                    requires_confirmation: is_aur,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", editor),
                    description: format!("Remove {}", editor_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: format!("{} installed. Launch from application menu or terminal.", editor_name),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("editor_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("editor.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking text editors".to_string(),
            goals: vec!["List installed editors".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-editors".to_string(),
                    description: "List text editors".to_string(),
                    command: "pacman -Q visual-studio-code-bin sublime-text-4 neovim 2>/dev/null || echo 'No editors installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed text editors".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("editor_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_editors_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("editor.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListEditors"));

        Ok(ActionPlan {
            analysis: "Showing available text editors".to_string(),
            goals: vec!["List available editors".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-editors".to_string(),
                    description: "Show available editors".to_string(),
                    command: "echo 'Available:\\n- Neovim (official)\\n- VS Code (AUR)\\n- Sublime Text (AUR)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Popular text editors for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("editor_list_editors".to_string()),
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
        assert!(EditorRecipe::matches_request("install vscode"));
        assert!(EditorRecipe::matches_request("install vs code"));
        assert!(EditorRecipe::matches_request("setup sublime text"));
        assert!(EditorRecipe::matches_request("install neovim"));
        assert!(!EditorRecipe::matches_request("what is vscode"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install neovim".to_string());
        let plan = EditorRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
