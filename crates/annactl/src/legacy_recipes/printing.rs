// Beta.166: Printing System (CUPS) Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct PrintingRecipe;

#[derive(Debug, PartialEq)]
enum PrintingOperation {
    Install,
    CheckStatus,
    AddPrinter,
    ListPrinters,
}

impl PrintingOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("add printer") || input_lower.contains("setup printer") {
            PrintingOperation::AddPrinter
        } else if input_lower.contains("list printer") || input_lower.contains("show printer") {
            PrintingOperation::ListPrinters
        } else if input_lower.contains("check") || input_lower.contains("status") {
            PrintingOperation::CheckStatus
        } else {
            PrintingOperation::Install
        }
    }
}

impl PrintingRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("cups") || input_lower.contains("print")
            || input_lower.contains("printer");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("add") || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = PrintingOperation::detect(user_input);
        match operation {
            PrintingOperation::Install => Self::build_install_plan(telemetry),
            PrintingOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            PrintingOperation::AddPrinter => Self::build_add_printer_plan(telemetry),
            PrintingOperation::ListPrinters => Self::build_list_printers_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("printing.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));

        Ok(ActionPlan {
            analysis: "Installing CUPS printing system".to_string(),
            goals: vec![
                "Install CUPS and printer drivers".to_string(),
                "Enable CUPS service".to_string(),
            ],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "install-cups".to_string(),
                    description: "Install CUPS".to_string(),
                    command: "sudo pacman -S --needed --noconfirm cups cups-pdf".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-cups".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "install-drivers".to_string(),
                    description: "Install printer drivers".to_string(),
                    command: "sudo pacman -S --needed --noconfirm gutenprint foomatic-db-engine foomatic-db foomatic-db-ppds foomatic-db-nonfree foomatic-db-nonfree-ppds".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "enable-cups".to_string(),
                    description: "Enable CUPS service".to_string(),
                    command: "sudo systemctl enable --now cups".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("disable-cups".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "disable-cups".to_string(),
                    description: "Disable CUPS service".to_string(),
                    command: "sudo systemctl disable --now cups".to_string(),
                },
                RollbackStep {
                    id: "remove-cups".to_string(),
                    description: "Remove CUPS".to_string(),
                    command: "sudo pacman -Rns --noconfirm cups cups-pdf".to_string(),
                },
            ],
            notes_for_user: "CUPS installed and running. Web interface: http://localhost:631. Add printers via web interface or command: lpadmin".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("printing_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("printing.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking printing system status".to_string(),
            goals: vec!["Show CUPS status and installed printers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "check-cups".to_string(),
                    description: "Check CUPS service".to_string(),
                    command: "systemctl status cups".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "list-printers".to_string(),
                    description: "List installed printers".to_string(),
                    command: "lpstat -p 2>/dev/null || echo 'No printers configured'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows CUPS service status and configured printers".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("printing_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_add_printer_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("printing.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("AddPrinter"));

        Ok(ActionPlan {
            analysis: "Guiding printer addition".to_string(),
            goals: vec!["Show how to add printers".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-cups".to_string(),
                    description: "Verify CUPS is installed".to_string(),
                    command: "systemctl is-active cups".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "show-instructions".to_string(),
                    description: "Show printer setup instructions".to_string(),
                    command: r"echo 'To add a printer:\n1. Web UI: http://localhost:631 → Administration → Add Printer\n2. Command: sudo lpadmin -p <printer_name> -v <device_uri> -E\n3. Auto-detect: sudo hp-setup (for HP printers)\n\nFind USB printers: lpinfo -v'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Printer addition requires device-specific information. Use web interface for easiest setup.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("printing_add_printer".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_printers_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("printing.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListPrinters"));

        Ok(ActionPlan {
            analysis: "Listing configured printers".to_string(),
            goals: vec!["Show all printers and their status".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-printers".to_string(),
                    description: "List all printers".to_string(),
                    command: "lpstat -p -d 2>/dev/null || echo 'No printers configured'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "show-print-jobs".to_string(),
                    description: "Show print queue".to_string(),
                    command: "lpstat -o 2>/dev/null || echo 'No print jobs in queue'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows configured printers and active print jobs".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("printing_list_printers".to_string()),
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
        assert!(PrintingRecipe::matches_request("install cups"));
        assert!(PrintingRecipe::matches_request("setup printer"));
        assert!(PrintingRecipe::matches_request("add printer"));
        assert!(!PrintingRecipe::matches_request("what is cups"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install cups".to_string());
        let plan = PrintingRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
