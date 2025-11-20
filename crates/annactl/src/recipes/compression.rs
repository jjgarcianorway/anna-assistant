// Beta.159: Compression Tools Recipe
// Handles installation of archive/compression tools (zip, unzip, p7zip, unrar)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct CompressionRecipe;

#[derive(Debug, PartialEq)]
enum CompressionOperation {
    Install,     // Install compression tools
    CheckStatus, // Verify installation
    InstallAll,  // Install complete suite
    ListTools,   // List available tools
}

impl CompressionOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            return CompressionOperation::CheckStatus;
        }
        if input_lower.contains("list") || input_lower.contains("show") {
            return CompressionOperation::ListTools;
        }
        if input_lower.contains("all") || input_lower.contains("suite") || input_lower.contains("complete") {
            return CompressionOperation::InstallAll;
        }
        CompressionOperation::Install
    }
}

impl CompressionRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        
        let has_compression_context = input_lower.contains("zip")
            || input_lower.contains("unzip")
            || input_lower.contains("7zip")
            || input_lower.contains("p7zip")
            || input_lower.contains("unrar")
            || input_lower.contains("rar")
            || input_lower.contains("archive")
            || input_lower.contains("compression")
            || input_lower.contains("extract");
        
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("check")
            || input_lower.contains("list");
        
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;
        
        has_compression_context && has_action && !is_info_only
    }
    
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = CompressionOperation::detect(user_input);
        
        match operation {
            CompressionOperation::Install => Self::build_install_plan(telemetry),
            CompressionOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            CompressionOperation::InstallAll => Self::build_install_all_plan(telemetry),
            CompressionOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }
    
    fn build_install_all_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("compression.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("InstallAll"));
        
        Ok(ActionPlan {
            analysis: "Installing complete compression tool suite".to_string(),
            goals: vec!["Install zip/unzip, p7zip, and unrar".to_string()],
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
                    id: "install-zip".to_string(),
                    description: "Install zip/unzip tools".to_string(),
                    command: "sudo pacman -S --needed --noconfirm zip unzip".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "install-p7zip".to_string(),
                    description: "Install p7zip (7-Zip)".to_string(),
                    command: "sudo pacman -S --needed --noconfirm p7zip".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "install-unrar".to_string(),
                    description: "Install unrar".to_string(),
                    command: "sudo pacman -S --needed --noconfirm unrar".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Compression tools installed:\n- zip/unzip: .zip archives\n- p7zip: .7z archives\n- unrar: .rar archives".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("compression_install_all".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
    
    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        Self::build_install_all_plan(_telemetry)
    }
    
    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("compression.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));
        
        Ok(ActionPlan {
            analysis: "Checking compression tools".to_string(),
            goals: vec!["List installed compression tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-tools".to_string(),
                    description: "List compression tools".to_string(),
                    command: "pacman -Q zip unzip p7zip unrar 2>/dev/null".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed compression tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("compression_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
    
    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("compression.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));
        
        Ok(ActionPlan {
            analysis: "Showing compression tools".to_string(),
            goals: vec!["List available compression tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show compression tools".to_string(),
                    command: "echo 'Available:\n- zip/unzip: .zip\n- p7zip: .7z\n- unrar: .rar'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Common compression tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("compression_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matches_compression_keywords() {
        assert!(CompressionRecipe::matches_request("install zip"));
        assert!(CompressionRecipe::matches_request("install compression tools"));
    }
    
    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!CompressionRecipe::matches_request("what is zip"));
    }
    
    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install compression tools".to_string());
        let plan = CompressionRecipe::build_install_plan(&telemetry).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
