// Beta.158: Productivity Applications Recipe
// Handles installation of productivity apps (LibreOffice, GIMP, Inkscape, etc.)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ProductivityRecipe;

#[derive(Debug, PartialEq)]
enum ProductivityOperation {
    Install,      // Install specified productivity app
    CheckStatus,  // Verify installation
    ListApps,     // List installed productivity apps
    InstallSuite, // Install full office suite
}

impl ProductivityOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();

        // Check status (highest priority)
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || (input_lower.contains("which") && input_lower.contains("installed"))
        {
            return ProductivityOperation::CheckStatus;
        }

        // List apps
        if input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("available")
        {
            return ProductivityOperation::ListApps;
        }

        // Install suite
        if input_lower.contains("suite")
            || input_lower.contains("full office")
            || input_lower.contains("complete office")
        {
            return ProductivityOperation::InstallSuite;
        }

        // Default to install
        ProductivityOperation::Install
    }
}

impl ProductivityRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        let has_productivity_context = input_lower.contains("office")
            || input_lower.contains("libreoffice")
            || input_lower.contains("gimp")
            || input_lower.contains("inkscape")
            || input_lower.contains("productivity")
            || input_lower.contains("document editor")
            || input_lower.contains("spreadsheet")
            || input_lower.contains("presentation")
            || input_lower.contains("image editor")
            || input_lower.contains("photo editor")
            || input_lower.contains("vector graphics");

        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("add")
            || input_lower.contains("get")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("available");

        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_productivity_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = ProductivityOperation::detect(user_input);

        match operation {
            ProductivityOperation::Install => Self::build_install_plan(telemetry),
            ProductivityOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ProductivityOperation::ListApps => Self::build_list_apps_plan(telemetry),
            ProductivityOperation::InstallSuite => Self::build_install_suite_plan(telemetry),
        }
    }

    fn detect_app(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("libreoffice") || input_lower.contains("office suite") {
            "libreoffice"
        } else if input_lower.contains("gimp") || input_lower.contains("image editor") || input_lower.contains("photo editor") {
            "gimp"
        } else if input_lower.contains("inkscape") || input_lower.contains("vector") {
            "inkscape"
        } else {
            "libreoffice" // Default to LibreOffice
        }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let app = Self::detect_app(user_input);

        let (app_name, package_name, description) = match app {
            "libreoffice" => ("LibreOffice", "libreoffice-fresh", "Full office suite (Writer, Calc, Impress, Draw)"),
            "gimp" => ("GIMP", "gimp", "GNU Image Manipulation Program (photo editor)"),
            "inkscape" => ("Inkscape", "inkscape", "Vector graphics editor"),
            _ => ("LibreOffice", "libreoffice-fresh", "Full office suite"),
        };

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-pacman".to_string(),
                description: "Verify pacman is available".to_string(),
                command: "which pacman".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-app".to_string(),
                description: format!("Check if {} is already installed", app_name),
                command: format!("pacman -Q {} 2>/dev/null || true", package_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: format!("install-{}", app),
                description: format!("Install {}", app_name),
                command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                risk_level: RiskLevel::Low,
                rollback_id: Some(format!("remove-{}", app)),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: format!("Verify {} is installed", app_name),
                command: format!("pacman -Q {}", package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: format!("remove-{}", app),
                description: format!("Remove {}", app_name),
                command: format!("sudo pacman -Rns --noconfirm {}", package_name),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("productivity.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("app".to_string(), serde_json::json!(app_name));
        meta_other.insert("package".to_string(), serde_json::json!(package_name));

        let launch_info = match app {
            "libreoffice" => "Launch apps: lowriter (Writer), localc (Calc), loimpress (Impress), lodraw (Draw)",
            "gimp" => "Launch: gimp",
            "inkscape" => "Launch: inkscape",
            _ => "Launch from application menu",
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} from official Arch repositories. {}", app_name, description),
            goals: vec![
                format!("Install {}", app_name),
                format!("Verify {} is working", app_name),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: format!("{} will be installed from official Arch repositories.\n\n\
                                   Description: {}\n\n\
                                   {}",
                                   app_name, description, launch_info),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("productivity_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-libreoffice".to_string(),
                description: "Check if LibreOffice is installed".to_string(),
                command: "pacman -Q libreoffice-fresh".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-gimp".to_string(),
                description: "Check if GIMP is installed".to_string(),
                command: "pacman -Q gimp".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-inkscape".to_string(),
                description: "Check if Inkscape is installed".to_string(),
                command: "pacman -Q inkscape".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "list-productivity-apps".to_string(),
                description: "List all installed productivity applications".to_string(),
                command: "echo '=== Office Suite ===' && pacman -Q libreoffice-fresh 2>/dev/null || echo 'Not installed' && echo '\n=== Graphics ===' && pacman -Q gimp inkscape 2>/dev/null || echo 'No graphics apps installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("productivity.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking which productivity applications are currently installed.".to_string(),
            goals: vec![
                "Verify LibreOffice installation status".to_string(),
                "Verify GIMP installation status".to_string(),
                "Verify Inkscape installation status".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will show which productivity apps (LibreOffice, GIMP, Inkscape) are currently installed.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("productivity_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_apps_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "list-installed-apps".to_string(),
                description: "List installed productivity applications".to_string(),
                command: "echo '=== Installed Productivity Apps ===' && pacman -Q libreoffice-fresh gimp inkscape 2>/dev/null | awk '{print \"- \" $1 \" (\" $2 \")\"}' || echo 'No productivity apps installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-available-apps".to_string(),
                description: "Show available productivity applications".to_string(),
                command: "echo '\n=== Available Productivity Apps ===\n\nOffice Suite:\n- LibreOffice: Full office suite (Writer, Calc, Impress, Draw)\n\nGraphics:\n- GIMP: Photo/image editor (like Photoshop)\n- Inkscape: Vector graphics editor (like Illustrator)'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("productivity.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListApps"));

        Ok(ActionPlan {
            analysis: "Showing installed productivity applications and available options.".to_string(),
            goals: vec![
                "List currently installed productivity apps".to_string(),
                "Show available apps for installation".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Available productivity applications:\n\n\
                           Office Suite:\n\
                           - LibreOffice: Complete office suite\n  \
                           - Writer (word processor)\n  \
                           - Calc (spreadsheet)\n  \
                           - Impress (presentations)\n  \
                           - Draw (diagrams)\n\n\
                           Graphics:\n\
                           - GIMP: Professional photo/image editing\n\
                           - Inkscape: Vector graphics and illustrations\n\n\
                           To install: annactl \"install <app-name>\"".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("productivity_list_apps".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_install_suite_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-pacman".to_string(),
                description: "Verify pacman is available".to_string(),
                command: "which pacman".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-suite".to_string(),
                description: "Check if LibreOffice suite is already installed".to_string(),
                command: "pacman -Q libreoffice-fresh 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-libreoffice-suite".to_string(),
                description: "Install full LibreOffice suite".to_string(),
                command: "sudo pacman -S --needed --noconfirm libreoffice-fresh".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-libreoffice-suite".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: "Verify LibreOffice suite is installed".to_string(),
                command: "pacman -Q libreoffice-fresh".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-libreoffice-suite".to_string(),
                description: "Remove LibreOffice suite".to_string(),
                command: "sudo pacman -Rns --noconfirm libreoffice-fresh".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("productivity.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("InstallSuite"));

        Ok(ActionPlan {
            analysis: "Installing full LibreOffice office suite from official Arch repositories.".to_string(),
            goals: vec![
                "Install complete LibreOffice suite".to_string(),
                "Verify all components are working".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "LibreOffice suite will be installed with all components:\n\n\
                           - Writer: Word processing (like MS Word)\n\
                           - Calc: Spreadsheets (like MS Excel)\n\
                           - Impress: Presentations (like MS PowerPoint)\n\
                           - Draw: Diagrams and drawings\n\
                           - Math: Formula editor\n\
                           - Base: Database (optional)\n\n\
                           Launch apps:\n\
                           - lowriter (Writer)\n\
                           - localc (Calc)\n\
                           - loimpress (Impress)\n\
                           - lodraw (Draw)".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("productivity_install_suite".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_productivity_keywords() {
        assert!(ProductivityRecipe::matches_request("install libreoffice"));
        assert!(ProductivityRecipe::matches_request("install gimp"));
        assert!(ProductivityRecipe::matches_request("install inkscape"));
        assert!(ProductivityRecipe::matches_request("install office suite"));
        assert!(ProductivityRecipe::matches_request("get productivity apps"));
    }

    #[test]
    fn test_matches_productivity_actions() {
        assert!(ProductivityRecipe::matches_request("check office status"));
        assert!(ProductivityRecipe::matches_request("list productivity apps"));
        assert!(ProductivityRecipe::matches_request("install image editor"));
    }

    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!ProductivityRecipe::matches_request("what is libreoffice"));
        assert!(!ProductivityRecipe::matches_request("tell me about gimp"));
        assert!(!ProductivityRecipe::matches_request("explain inkscape"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            ProductivityOperation::detect("install libreoffice"),
            ProductivityOperation::Install
        );
        assert_eq!(
            ProductivityOperation::detect("check office status"),
            ProductivityOperation::CheckStatus
        );
        assert_eq!(
            ProductivityOperation::detect("install full office suite"),
            ProductivityOperation::InstallSuite
        );
        assert_eq!(
            ProductivityOperation::detect("list productivity apps"),
            ProductivityOperation::ListApps
        );
    }

    #[test]
    fn test_app_detection() {
        assert_eq!(ProductivityRecipe::detect_app("install libreoffice"), "libreoffice");
        assert_eq!(ProductivityRecipe::detect_app("install gimp"), "gimp");
        assert_eq!(ProductivityRecipe::detect_app("install inkscape"), "inkscape");
        assert_eq!(ProductivityRecipe::detect_app("install office"), "libreoffice"); // Default
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install libreoffice".to_string());

        let plan = ProductivityRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("libreoffice") || g.to_lowercase().contains("install")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check productivity apps".to_string());

        let plan = ProductivityRecipe::build_check_status_plan(&telemetry).unwrap();

        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_list_apps_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "list productivity apps".to_string());

        let plan = ProductivityRecipe::build_list_apps_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("list") || g.to_lowercase().contains("available")));
        assert!(!plan.command_plan.is_empty());
    }

    #[test]
    fn test_install_suite_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install office suite".to_string());

        let plan = ProductivityRecipe::build_install_suite_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("suite") || g.to_lowercase().contains("libreoffice") || g.to_lowercase().contains("complete")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }
}
