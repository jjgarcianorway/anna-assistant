// Beta.162: Antivirus Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct AntivirusRecipe;

#[derive(Debug, PartialEq)]
enum AntivirusOperation {
    Install,
    CheckStatus,
    UpdateSignatures,
    ScanSystem,
}

impl AntivirusOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("update") || input_lower.contains("signatures") {
            AntivirusOperation::UpdateSignatures
        } else if input_lower.contains("scan") {
            AntivirusOperation::ScanSystem
        } else if input_lower.contains("check") || input_lower.contains("status") {
            AntivirusOperation::CheckStatus
        } else {
            AntivirusOperation::Install
        }
    }
}

impl AntivirusRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("clamav") || input_lower.contains("antivirus")
            || input_lower.contains("virus") || input_lower.contains("malware");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("scan")
            || input_lower.contains("update");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = AntivirusOperation::detect(user_input);
        match operation {
            AntivirusOperation::Install => Self::build_install_plan(telemetry),
            AntivirusOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            AntivirusOperation::UpdateSignatures => Self::build_update_signatures_plan(telemetry),
            AntivirusOperation::ScanSystem => Self::build_scan_system_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("antivirus.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));

        Ok(ActionPlan {
            analysis: "Installing ClamAV antivirus".to_string(),
            goals: vec![
                "Install ClamAV".to_string(),
                "Update virus signatures".to_string(),
            ],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "install-clamav".to_string(),
                    description: "Install ClamAV".to_string(),
                    command: "sudo pacman -S --needed --noconfirm clamav".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-clamav".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "update-signatures".to_string(),
                    description: "Update virus signatures".to_string(),
                    command: "sudo freshclam".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-clamav".to_string(),
                    description: "Remove ClamAV".to_string(),
                    command: "sudo pacman -Rns --noconfirm clamav".to_string(),
                },
            ],
            notes_for_user: "ClamAV installed. Scan files: clamscan /path/to/scan, Update signatures: sudo freshclam".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("antivirus_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("antivirus.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking ClamAV status".to_string(),
            goals: vec!["Check ClamAV installation and signature version".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "check-clamav".to_string(),
                    description: "Check ClamAV installation".to_string(),
                    command: "pacman -Q clamav 2>/dev/null || echo 'ClamAV not installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-version".to_string(),
                    description: "Check signature version".to_string(),
                    command: "clamscan --version 2>/dev/null || echo 'ClamAV not available'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows ClamAV installation status and version".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("antivirus_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_update_signatures_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("antivirus.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("UpdateSignatures"));

        Ok(ActionPlan {
            analysis: "Updating ClamAV virus signatures".to_string(),
            goals: vec!["Download latest virus signatures".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-clamav".to_string(),
                    description: "Verify ClamAV is installed".to_string(),
                    command: "which freshclam".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "update-signatures".to_string(),
                    description: "Update virus signatures".to_string(),
                    command: "sudo freshclam".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Virus signatures updated. Run scans with: clamscan /path/to/scan".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("antivirus_update_signatures".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_scan_system_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("antivirus.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ScanSystem"));

        Ok(ActionPlan {
            analysis: "Scanning system for viruses".to_string(),
            goals: vec!["Run full system scan with ClamAV".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-clamav".to_string(),
                    description: "Verify ClamAV is installed".to_string(),
                    command: "which clamscan".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "scan-home".to_string(),
                    description: "Scan home directory".to_string(),
                    command: "clamscan -r --bell -i $HOME".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Scanning home directory. For full system scan: sudo clamscan -r --bell -i /".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("antivirus_scan_system".to_string()),
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
        assert!(AntivirusRecipe::matches_request("install clamav"));
        assert!(AntivirusRecipe::matches_request("scan for viruses"));
        assert!(AntivirusRecipe::matches_request("update antivirus"));
        assert!(!AntivirusRecipe::matches_request("what is clamav"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install clamav".to_string());
        let plan = AntivirusRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
