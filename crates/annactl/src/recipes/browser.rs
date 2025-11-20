// Beta.158: Web Browser Installation Recipe
// Handles installation of popular web browsers (Firefox, Chrome, Chromium, Brave)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct BrowserRecipe;

#[derive(Debug, PartialEq)]
enum BrowserOperation {
    Install,      // Install specified browser
    CheckStatus,  // Verify browser installation
    SetDefault,   // Set default browser
    ListBrowsers, // List installed browsers
}

impl BrowserOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();

        // Check status (highest priority)
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || (input_lower.contains("which") && input_lower.contains("browser"))
            || input_lower.contains("installed")
        {
            return BrowserOperation::CheckStatus;
        }

        // List browsers
        if input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("available")
        {
            return BrowserOperation::ListBrowsers;
        }

        // Set default
        if input_lower.contains("default")
            || input_lower.contains("set as")
            || input_lower.contains("make it")
        {
            return BrowserOperation::SetDefault;
        }

        // Default to install
        BrowserOperation::Install
    }
}

impl BrowserRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        let has_browser_context = input_lower.contains("browser")
            || input_lower.contains("firefox")
            || input_lower.contains("chrome")
            || input_lower.contains("chromium")
            || input_lower.contains("brave")
            || input_lower.contains("web browser")
            || (input_lower.contains("google") && input_lower.contains("chrome"));

        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("add")
            || input_lower.contains("get")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("default")
            || input_lower.contains("set")
            || input_lower.contains("which")
            || input_lower.contains("available");

        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_browser_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = BrowserOperation::detect(user_input);

        match operation {
            BrowserOperation::Install => Self::build_install_plan(telemetry),
            BrowserOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            BrowserOperation::SetDefault => Self::build_set_default_plan(telemetry),
            BrowserOperation::ListBrowsers => Self::build_list_browsers_plan(telemetry),
        }
    }

    fn detect_browser(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("firefox") {
            "firefox"
        } else if input_lower.contains("chrome") && !input_lower.contains("chromium") {
            "google-chrome"
        } else if input_lower.contains("chromium") {
            "chromium"
        } else if input_lower.contains("brave") {
            "brave"
        } else {
            "firefox" // Default to Firefox
        }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let browser = Self::detect_browser(user_input);

        let (browser_name, package_name, is_aur) = match browser {
            "firefox" => ("Firefox", "firefox", false),
            "google-chrome" => ("Google Chrome", "google-chrome", true),
            "chromium" => ("Chromium", "chromium", false),
            "brave" => ("Brave", "brave-bin", true),
            _ => ("Firefox", "firefox", false),
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
                id: "check-existing-browser".to_string(),
                description: format!("Check if {} is already installed", browser_name),
                command: format!("pacman -Q {} 2>/dev/null || true", package_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let mut command_plan = vec![];
        let mut rollback_plan = vec![];

        if is_aur {
            // AUR packages require yay or paru
            command_plan.push(CommandStep {
                id: "check-aur-helper".to_string(),
                description: "Check if AUR helper (yay/paru) is installed".to_string(),
                command: "which yay || which paru".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            });

            command_plan.push(CommandStep {
                id: format!("install-{}", browser),
                description: format!("Install {} from AUR", browser_name),
                command: format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name),
                risk_level: RiskLevel::Medium,
                rollback_id: Some(format!("remove-{}", browser)),
                requires_confirmation: true,
            });

            rollback_plan.push(RollbackStep {
                id: format!("remove-{}", browser),
                description: format!("Remove {}", browser_name),
                command: format!("yay -Rns --noconfirm {} || paru -Rns --noconfirm {}", package_name, package_name),
            });
        } else {
            // Official repo packages
            command_plan.push(CommandStep {
                id: format!("install-{}", browser),
                description: format!("Install {} from official repos", browser_name),
                command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                risk_level: RiskLevel::Low,
                rollback_id: Some(format!("remove-{}", browser)),
                requires_confirmation: false,
            });

            rollback_plan.push(RollbackStep {
                id: format!("remove-{}", browser),
                description: format!("Remove {}", browser_name),
                command: format!("sudo pacman -Rns --noconfirm {}", package_name),
            });
        }

        command_plan.push(CommandStep {
            id: "verify-installation".to_string(),
            description: format!("Verify {} is installed", browser_name),
            command: format!("pacman -Q {}", package_name),
            risk_level: RiskLevel::Info,
            rollback_id: None,
            requires_confirmation: false,
        });

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("browser.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("browser".to_string(), serde_json::json!(browser_name));
        meta_other.insert("package".to_string(), serde_json::json!(package_name));
        meta_other.insert("source".to_string(), serde_json::json!(if is_aur { "AUR" } else { "Official" }));

        let install_note = if is_aur {
            format!("{} will be installed from the AUR. This requires an AUR helper (yay or paru) to be installed.\n\n\
                    AUR packages are user-submitted and not officially supported by Arch Linux.\n\n\
                    If you don't have an AUR helper, install yay first: see 'annactl \"install yay\"'", browser_name)
        } else {
            format!("{} will be installed from official Arch repositories.\n\n\
                    This is a safe, officially supported package.", browser_name)
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} web browser on Arch Linux.", browser_name),
            goals: vec![
                format!("Install {} web browser", browser_name),
                format!("Verify {} is working", browser_name),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: install_note,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("browser_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-firefox".to_string(),
                description: "Check if Firefox is installed".to_string(),
                command: "pacman -Q firefox".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-chromium".to_string(),
                description: "Check if Chromium is installed".to_string(),
                command: "pacman -Q chromium".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-chrome".to_string(),
                description: "Check if Google Chrome is installed".to_string(),
                command: "pacman -Q google-chrome".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-brave".to_string(),
                description: "Check if Brave is installed".to_string(),
                command: "pacman -Q brave-bin".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "list-browsers".to_string(),
                description: "List all installed web browsers".to_string(),
                command: "pacman -Q firefox chromium google-chrome brave-bin 2>/dev/null || echo 'No common browsers installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("browser.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking which web browsers are currently installed on the system.".to_string(),
            goals: vec![
                "Verify Firefox installation status".to_string(),
                "Verify Chromium installation status".to_string(),
                "Verify Chrome installation status".to_string(),
                "Verify Brave installation status".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will show which web browsers (Firefox, Chromium, Chrome, Brave) are currently installed.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("browser_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_set_default_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let browser = Self::detect_browser(user_input);

        let (browser_name, desktop_file) = match browser {
            "firefox" => ("Firefox", "firefox.desktop"),
            "google-chrome" => ("Google Chrome", "google-chrome.desktop"),
            "chromium" => ("Chromium", "chromium.desktop"),
            "brave" => ("Brave", "brave-browser.desktop"),
            _ => ("Firefox", "firefox.desktop"),
        };

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-xdg-settings".to_string(),
                description: "Check if xdg-utils is installed".to_string(),
                command: "which xdg-settings".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-default-instructions".to_string(),
                description: format!("Show instructions to set {} as default browser", browser_name),
                command: format!("echo 'To set {} as your default browser, run:\n\nxdg-settings set default-web-browser {}\n\nNote: This command must be run by the user (not as root) in a graphical session.\nAnna cannot run this automatically because it requires your user environment.'", browser_name, desktop_file),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("browser.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("SetDefault"));
        meta_other.insert("browser".to_string(), serde_json::json!(browser_name));

        Ok(ActionPlan {
            analysis: format!("Providing instructions to set {} as the default web browser.", browser_name),
            goals: vec![
                format!("Explain how to set {} as default", browser_name),
                "Provide xdg-settings command".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: format!("Setting the default browser requires running a command in your user session.\n\n\
                                   Anna cannot do this automatically because:\n\
                                   1. It must be run as your user (not root)\n\
                                   2. It requires a graphical session\n\n\
                                   Run the command shown above in your terminal to set {} as default.", browser_name),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("browser_set_default".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_browsers_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "list-installed-browsers".to_string(),
                description: "List installed browsers".to_string(),
                command: "echo '=== Installed Browsers ===' && pacman -Q firefox chromium google-chrome brave-bin 2>/dev/null | awk '{print \"- \" $1 \" (\" $2 \")\"}' || echo 'No browsers installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-available-browsers".to_string(),
                description: "Show available browsers for installation".to_string(),
                command: "echo '\n=== Available Browsers ===\n- Firefox (official repo)\n- Chromium (official repo)\n- Google Chrome (AUR)\n- Brave (AUR)'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("browser.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListBrowsers"));

        Ok(ActionPlan {
            analysis: "Showing installed browsers and available options for installation.".to_string(),
            goals: vec![
                "List currently installed browsers".to_string(),
                "Show available browsers for installation".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Available browsers:\n\n\
                           Official repos (pacman):\n\
                           - Firefox: Open-source, privacy-focused\n\
                           - Chromium: Open-source Chrome base\n\n\
                           AUR (requires yay/paru):\n\
                           - Google Chrome: Official Google browser\n\
                           - Brave: Privacy-focused, built-in ad blocking\n\n\
                           To install: annactl \"install <browser-name>\"".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("browser_list_browsers".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_browser_keywords() {
        assert!(BrowserRecipe::matches_request("install firefox"));
        assert!(BrowserRecipe::matches_request("install chrome"));
        assert!(BrowserRecipe::matches_request("install chromium"));
        assert!(BrowserRecipe::matches_request("install brave browser"));
        assert!(BrowserRecipe::matches_request("get a web browser"));
    }

    #[test]
    fn test_matches_browser_actions() {
        assert!(BrowserRecipe::matches_request("check browser status"));
        assert!(BrowserRecipe::matches_request("list browsers"));
        assert!(BrowserRecipe::matches_request("set firefox as default"));
    }

    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!BrowserRecipe::matches_request("what is firefox"));
        assert!(!BrowserRecipe::matches_request("tell me about chrome"));
        assert!(!BrowserRecipe::matches_request("explain chromium"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            BrowserOperation::detect("install firefox"),
            BrowserOperation::Install
        );
        assert_eq!(
            BrowserOperation::detect("check browser status"),
            BrowserOperation::CheckStatus
        );
        assert_eq!(
            BrowserOperation::detect("set firefox as default"),
            BrowserOperation::SetDefault
        );
        assert_eq!(
            BrowserOperation::detect("list available browsers"),
            BrowserOperation::ListBrowsers
        );
    }

    #[test]
    fn test_browser_detection() {
        assert_eq!(BrowserRecipe::detect_browser("install firefox"), "firefox");
        assert_eq!(BrowserRecipe::detect_browser("install chrome"), "google-chrome");
        assert_eq!(BrowserRecipe::detect_browser("install chromium"), "chromium");
        assert_eq!(BrowserRecipe::detect_browser("install brave"), "brave");
        assert_eq!(BrowserRecipe::detect_browser("install browser"), "firefox"); // Default
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install firefox".to_string());

        let plan = BrowserRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("firefox") || g.to_lowercase().contains("browser")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check browser status".to_string());

        let plan = BrowserRecipe::build_check_status_plan(&telemetry).unwrap();

        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_set_default_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "set firefox as default".to_string());

        let plan = BrowserRecipe::build_set_default_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("default")));
        assert!(!plan.command_plan.is_empty());
    }

    #[test]
    fn test_list_browsers_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "list browsers".to_string());

        let plan = BrowserRecipe::build_list_browsers_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("list") || g.to_lowercase().contains("available")));
        assert!(!plan.command_plan.is_empty());
    }
}
