// Beta.157: System Monitoring Tools Recipe
// Handles installation and configuration of system monitoring tools (htop, btop, glances)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct MonitoringRecipe;

#[derive(Debug, PartialEq)]
enum MonitoringOperation {
    Install,       // Install monitoring tools (htop, btop, glances)
    CheckStatus,   // Verify tool installation
    ConfigureBtop, // Configure btop theme and settings
    LaunchTool,    // Launch specific monitoring tool
}

impl MonitoringOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();

        // Check status (highest priority)
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || input_lower.contains("list")
            || input_lower.contains("which")
            || input_lower.contains("installed")
        {
            return MonitoringOperation::CheckStatus;
        }

        // Launch tool
        if input_lower.contains("launch")
            || input_lower.contains("start")
            || input_lower.contains("run")
            || input_lower.contains("open")
            || input_lower.contains("show me")
        {
            return MonitoringOperation::LaunchTool;
        }

        // Configure btop
        if input_lower.contains("configure")
            || input_lower.contains("config")
            || input_lower.contains("setup")
            || input_lower.contains("theme")
            || input_lower.contains("customize")
            || input_lower.contains("settings")
        {
            return MonitoringOperation::ConfigureBtop;
        }

        // Default to install
        MonitoringOperation::Install
    }
}

impl MonitoringRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        let has_monitoring_context = input_lower.contains("monitor")
            || input_lower.contains("htop")
            || input_lower.contains("btop")
            || input_lower.contains("glances")
            || input_lower.contains("system resources")
            || input_lower.contains("task manager")
            || input_lower.contains("resource usage")
            || (input_lower.contains("cpu") && input_lower.contains("usage"))
            || (input_lower.contains("memory") && input_lower.contains("usage"))
            || (input_lower.contains("process") && input_lower.contains("viewer"));

        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("launch")
            || input_lower.contains("start")
            || input_lower.contains("run")
            || input_lower.contains("open")
            || input_lower.contains("show")
            || input_lower.contains("add")
            || input_lower.contains("get")
            || input_lower.contains("set")
            || input_lower.contains("theme")
            || input_lower.contains("customize");

        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_monitoring_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = MonitoringOperation::detect(user_input);

        match operation {
            MonitoringOperation::Install => Self::build_install_plan(telemetry),
            MonitoringOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            MonitoringOperation::ConfigureBtop => Self::build_configure_btop_plan(telemetry),
            MonitoringOperation::LaunchTool => Self::build_launch_tool_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-pacman".to_string(),
                description: "Verify pacman is available".to_string(),
                command: "which pacman".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-tools".to_string(),
                description: "Check which monitoring tools are already installed".to_string(),
                command: "pacman -Q htop btop glances 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-htop".to_string(),
                description: "Install htop (interactive process viewer)".to_string(),
                command: "sudo pacman -S --needed --noconfirm htop".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-htop".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-btop".to_string(),
                description: "Install btop (resource monitor with modern UI)".to_string(),
                command: "sudo pacman -S --needed --noconfirm btop".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-btop".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-glances".to_string(),
                description: "Install glances (comprehensive system monitoring)".to_string(),
                command: "sudo pacman -S --needed --noconfirm glances".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-glances".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: "Verify all tools are installed".to_string(),
                command: "pacman -Q htop btop glances".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-htop".to_string(),
                description: "Remove htop".to_string(),
                command: "sudo pacman -Rns --noconfirm htop".to_string(),
            },
            RollbackStep {
                id: "remove-btop".to_string(),
                description: "Remove btop".to_string(),
                command: "sudo pacman -Rns --noconfirm btop".to_string(),
            },
            RollbackStep {
                id: "remove-glances".to_string(),
                description: "Remove glances".to_string(),
                command: "sudo pacman -Rns --noconfirm glances".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("monitoring.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tools".to_string(), serde_json::json!("htop, btop, glances"));

        Ok(ActionPlan {
            analysis: "Installing three popular system monitoring tools: htop (classic process viewer), btop (modern resource monitor), and glances (comprehensive system monitoring).".to_string(),
            goals: vec![
                "Install htop for interactive process viewing".to_string(),
                "Install btop for modern resource monitoring".to_string(),
                "Install glances for comprehensive system stats".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "These tools provide different monitoring interfaces:\n- htop: Classic, lightweight process viewer\n- btop: Modern UI with graphs and themes\n- glances: Comprehensive system stats (CPU, RAM, disk, network)\n\nRun them with: htop, btop, or glances".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("monitoring_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-htop".to_string(),
                description: "Check if htop is installed".to_string(),
                command: "pacman -Q htop".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-btop".to_string(),
                description: "Check if btop is installed".to_string(),
                command: "pacman -Q btop".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-glances".to_string(),
                description: "Check if glances is installed".to_string(),
                command: "pacman -Q glances".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "list-monitoring-tools".to_string(),
                description: "List all installed monitoring tools".to_string(),
                command: "pacman -Q htop btop glances 2>/dev/null || echo 'No monitoring tools installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("monitoring.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking which system monitoring tools are currently installed.".to_string(),
            goals: vec![
                "Verify htop installation status".to_string(),
                "Verify btop installation status".to_string(),
                "Verify glances installation status".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will show which monitoring tools (htop, btop, glances) are currently installed on your system.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("monitoring_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_configure_btop_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-btop-installed".to_string(),
                description: "Verify btop is installed".to_string(),
                command: "which btop".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "create-config-dir".to_string(),
                description: "Create btop config directory".to_string(),
                command: "mkdir -p ~/.config/btop".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "generate-default-config".to_string(),
                description: "Generate default btop configuration".to_string(),
                command: "btop --help > /dev/null 2>&1 || true".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-config-location".to_string(),
                description: "Show btop config file location".to_string(),
                command: "echo 'Config file: ~/.config/btop/btop.conf'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("monitoring.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ConfigureBtop"));

        Ok(ActionPlan {
            analysis: "Setting up btop configuration directory and providing guidance on customization.".to_string(),
            goals: vec![
                "Create btop config directory".to_string(),
                "Provide config file location".to_string(),
                "Enable theme and setting customization".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "btop configuration file: ~/.config/btop/btop.conf\n\nRun btop once to generate default config, then edit the file to customize:\n- color_theme (Default, TTY, Monokai, Nord, etc.)\n- update_ms (refresh rate)\n- shown_boxes (which panels to display)\n\nPress 'ESC' in btop to access the menu for theme selection.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("monitoring_configure_btop".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_launch_tool_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-tools-installed".to_string(),
                description: "Check which monitoring tools are available".to_string(),
                command: "pacman -Q htop btop glances 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-available-tools".to_string(),
                description: "Show available monitoring tools".to_string(),
                command: "echo 'Available monitoring tools:' && pacman -Q htop btop glances 2>/dev/null | awk '{print \"- \" $1}' || echo 'No monitoring tools installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("monitoring.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("LaunchTool"));

        Ok(ActionPlan {
            analysis: "Providing information on how to launch installed monitoring tools.".to_string(),
            goals: vec![
                "Show which tools are installed".to_string(),
                "Provide launch commands".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "To launch monitoring tools, run:\n- htop    (interactive process viewer)\n- btop    (modern resource monitor)\n- glances (comprehensive system stats)\n\nPress 'q' or 'Ctrl+C' to exit any tool.\n\nNote: These are interactive TUI applications and must be run directly in your terminal, not through annactl.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("monitoring_launch_tool".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_monitoring_keywords() {
        assert!(MonitoringRecipe::matches_request("install htop"));
        assert!(MonitoringRecipe::matches_request("setup btop"));
        assert!(MonitoringRecipe::matches_request("install glances"));
        assert!(MonitoringRecipe::matches_request("install system monitoring tools"));
    }

    #[test]
    fn test_matches_task_manager_context() {
        assert!(MonitoringRecipe::matches_request("install task manager"));
        assert!(MonitoringRecipe::matches_request("setup resource monitor"));
        assert!(MonitoringRecipe::matches_request("show me cpu usage tool"));
    }

    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!MonitoringRecipe::matches_request("what is htop"));
        assert!(!MonitoringRecipe::matches_request("tell me about btop"));
        assert!(!MonitoringRecipe::matches_request("explain glances"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            MonitoringOperation::detect("install htop"),
            MonitoringOperation::Install
        );
        assert_eq!(
            MonitoringOperation::detect("check htop status"),
            MonitoringOperation::CheckStatus
        );
        assert_eq!(
            MonitoringOperation::detect("configure btop theme"),
            MonitoringOperation::ConfigureBtop
        );
        assert_eq!(
            MonitoringOperation::detect("launch btop"),
            MonitoringOperation::LaunchTool
        );
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install monitoring tools".to_string());

        let plan = MonitoringRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("htop")));
        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("btop")));
        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("glances")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check monitoring tools".to_string());

        let plan = MonitoringRecipe::build_check_status_plan(&telemetry).unwrap();

        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_configure_btop_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "configure btop".to_string());

        let plan = MonitoringRecipe::build_configure_btop_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("config")));
        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
    }

    #[test]
    fn test_launch_tool_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "launch btop".to_string());

        let plan = MonitoringRecipe::build_launch_tool_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("launch") || g.to_lowercase().contains("show")));
        assert!(!plan.command_plan.is_empty());
    }
}
