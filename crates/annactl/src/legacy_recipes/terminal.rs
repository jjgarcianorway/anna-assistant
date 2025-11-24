// Beta.159: Terminal Tools Recipe
// Handles installation of terminal emulators and terminal multiplexers (alacritty, kitty, tmux, screen)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct TerminalRecipe;

#[derive(Debug, PartialEq)]
enum TerminalOperation {
    Install,      // Install terminal tool
    CheckStatus,  // Verify installation
    ConfigureTmux, // Configure tmux with sensible defaults
    ListTools,    // List installed terminal tools
}

impl TerminalOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();

        // Check status (highest priority)
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || (input_lower.contains("which") && input_lower.contains("installed"))
        {
            return TerminalOperation::CheckStatus;
        }

        // List tools
        if input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("available")
        {
            return TerminalOperation::ListTools;
        }

        // Configure tmux
        if (input_lower.contains("configure") || input_lower.contains("setup") || input_lower.contains("config"))
            && input_lower.contains("tmux")
        {
            return TerminalOperation::ConfigureTmux;
        }

        // Default to install
        TerminalOperation::Install
    }
}

impl TerminalRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        let has_terminal_context = input_lower.contains("terminal")
            || input_lower.contains("alacritty")
            || input_lower.contains("kitty")
            || input_lower.contains("tmux")
            || input_lower.contains("screen")
            || input_lower.contains("multiplexer")
            || input_lower.contains("terminal emulator");

        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("add")
            || input_lower.contains("get")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("configure")
            || input_lower.contains("available");

        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_terminal_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = TerminalOperation::detect(user_input);

        match operation {
            TerminalOperation::Install => Self::build_install_plan(telemetry),
            TerminalOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            TerminalOperation::ConfigureTmux => Self::build_configure_tmux_plan(telemetry),
            TerminalOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("alacritty") {
            "alacritty"
        } else if input_lower.contains("kitty") {
            "kitty"
        } else if input_lower.contains("tmux") {
            "tmux"
        } else if input_lower.contains("screen") {
            "screen"
        } else {
            "tmux" // Default to tmux
        }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);

        let (tool_name, package_name, description) = match tool {
            "alacritty" => ("Alacritty", "alacritty", "GPU-accelerated terminal emulator"),
            "kitty" => ("Kitty", "kitty", "GPU-based terminal emulator with advanced features"),
            "tmux" => ("tmux", "tmux", "Terminal multiplexer (split/detach sessions)"),
            "screen" => ("GNU Screen", "screen", "Classic terminal multiplexer"),
            _ => ("tmux", "tmux", "Terminal multiplexer"),
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
                id: "check-existing-tool".to_string(),
                description: format!("Check if {} is already installed", tool_name),
                command: format!("pacman -Q {} 2>/dev/null || true", package_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: format!("install-{}", tool),
                description: format!("Install {}", tool_name),
                command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                risk_level: RiskLevel::Low,
                rollback_id: Some(format!("remove-{}", tool)),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: format!("Verify {} is installed", tool_name),
                command: format!("pacman -Q {}", package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: format!("remove-{}", tool),
                description: format!("Remove {}", tool_name),
                command: format!("sudo pacman -Rns --noconfirm {}", package_name),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("terminal.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));
        meta_other.insert("package".to_string(), serde_json::json!(package_name));

        let usage_info = match tool {
            "alacritty" => "Launch: alacritty\n\nConfig: ~/.config/alacritty/alacritty.yml",
            "kitty" => "Launch: kitty\n\nConfig: ~/.config/kitty/kitty.conf",
            "tmux" => "Launch: tmux\n\nBasic commands:\n- tmux new: Create session\n- Ctrl+b d: Detach\n- tmux attach: Reattach",
            "screen" => "Launch: screen\n\nBasic commands:\n- screen: Create session\n- Ctrl+a d: Detach\n- screen -r: Reattach",
            _ => "Check documentation for usage",
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} from official Arch repositories. {}", tool_name, description),
            goals: vec![
                format!("Install {}", tool_name),
                format!("Verify {} is working", tool_name),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: format!("{} will be installed from official Arch repositories.\n\n\
                                   Description: {}\n\n\
                                   {}",
                                   tool_name, description, usage_info),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("terminal_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-alacritty".to_string(),
                description: "Check if Alacritty is installed".to_string(),
                command: "pacman -Q alacritty".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-kitty".to_string(),
                description: "Check if Kitty is installed".to_string(),
                command: "pacman -Q kitty".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-tmux".to_string(),
                description: "Check if tmux is installed".to_string(),
                command: "pacman -Q tmux".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-screen".to_string(),
                description: "Check if screen is installed".to_string(),
                command: "pacman -Q screen".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "list-terminal-tools".to_string(),
                description: "List all installed terminal tools".to_string(),
                command: "echo '=== Terminal Emulators ===' && pacman -Q alacritty kitty 2>/dev/null || echo 'No terminal emulators installed' && echo '\n=== Terminal Multiplexers ===' && pacman -Q tmux screen 2>/dev/null || echo 'No multiplexers installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("terminal.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking which terminal tools are currently installed.".to_string(),
            goals: vec![
                "Verify terminal emulator installation status".to_string(),
                "Verify terminal multiplexer installation status".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will show which terminal tools (Alacritty, Kitty, tmux, screen) are currently installed.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("terminal_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_configure_tmux_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-tmux-installed".to_string(),
                description: "Verify tmux is installed".to_string(),
                command: "which tmux".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-tmux-config-instructions".to_string(),
                description: "Show tmux configuration instructions".to_string(),
                command: "echo 'To configure tmux, create ~/.tmux.conf with sensible defaults:\n\n\
                         # Example configuration:\n\
                         set -g mouse on\n\
                         set -g base-index 1\n\
                         set -g history-limit 10000\n\
                         set -g status-style bg=colour235,fg=colour136\n\
                         \n\
                         # Better prefix (Ctrl+a instead of Ctrl+b):\n\
                         # unbind C-b\n\
                         # set -g prefix C-a\n\
                         # bind C-a send-prefix\n\
                         \n\
                         After creating the file, run: tmux source ~/.tmux.conf'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("terminal.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ConfigureTmux"));

        Ok(ActionPlan {
            analysis: "Providing instructions for configuring tmux with sensible defaults.".to_string(),
            goals: vec![
                "Explain tmux configuration file location".to_string(),
                "Provide example configuration".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "tmux configuration file: ~/.tmux.conf\n\n\
                           Recommended settings:\n\
                           - Enable mouse support\n\
                           - Increase history limit\n\
                           - Customize status bar\n\
                           - Consider changing prefix key\n\n\
                           After editing, reload with: tmux source ~/.tmux.conf".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("terminal_configure_tmux".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "list-installed-tools".to_string(),
                description: "List installed terminal tools".to_string(),
                command: "echo '=== Installed Terminal Tools ===' && pacman -Q alacritty kitty tmux screen 2>/dev/null | awk '{print \"- \" $1 \" (\" $2 \")\"}' || echo 'No terminal tools installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-available-tools".to_string(),
                description: "Show available terminal tools".to_string(),
                command: "echo '\n=== Available Terminal Tools ===\n\nTerminal Emulators:\n- Alacritty: GPU-accelerated, minimal config\n- Kitty: GPU-based, advanced features\n\nTerminal Multiplexers:\n- tmux: Modern, widely used\n- screen: Classic, stable'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("terminal.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing installed terminal tools and available options.".to_string(),
            goals: vec![
                "List currently installed terminal tools".to_string(),
                "Show available tools for installation".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Available terminal tools:\n\n\
                           Terminal Emulators:\n\
                           - Alacritty: Fast, GPU-accelerated\n\
                           - Kitty: Feature-rich, GPU-based\n\n\
                           Terminal Multiplexers:\n\
                           - tmux: Session management, split panes\n\
                           - screen: Classic multiplexer\n\n\
                           To install: annactl \"install <tool-name>\"".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("terminal_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_terminal_keywords() {
        assert!(TerminalRecipe::matches_request("install alacritty"));
        assert!(TerminalRecipe::matches_request("install kitty"));
        assert!(TerminalRecipe::matches_request("install tmux"));
        assert!(TerminalRecipe::matches_request("install terminal emulator"));
        assert!(TerminalRecipe::matches_request("setup terminal multiplexer"));
    }

    #[test]
    fn test_matches_terminal_actions() {
        assert!(TerminalRecipe::matches_request("check terminal status"));
        assert!(TerminalRecipe::matches_request("list terminal tools"));
        assert!(TerminalRecipe::matches_request("configure tmux"));
    }

    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!TerminalRecipe::matches_request("what is tmux"));
        assert!(!TerminalRecipe::matches_request("tell me about alacritty"));
        assert!(!TerminalRecipe::matches_request("explain screen"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            TerminalOperation::detect("install tmux"),
            TerminalOperation::Install
        );
        assert_eq!(
            TerminalOperation::detect("check terminal status"),
            TerminalOperation::CheckStatus
        );
        assert_eq!(
            TerminalOperation::detect("configure tmux"),
            TerminalOperation::ConfigureTmux
        );
        assert_eq!(
            TerminalOperation::detect("list terminal tools"),
            TerminalOperation::ListTools
        );
    }

    #[test]
    fn test_tool_detection() {
        assert_eq!(TerminalRecipe::detect_tool("install alacritty"), "alacritty");
        assert_eq!(TerminalRecipe::detect_tool("install kitty"), "kitty");
        assert_eq!(TerminalRecipe::detect_tool("install tmux"), "tmux");
        assert_eq!(TerminalRecipe::detect_tool("install screen"), "screen");
        assert_eq!(TerminalRecipe::detect_tool("install terminal"), "tmux"); // Default
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install tmux".to_string());

        let plan = TerminalRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("tmux") || g.to_lowercase().contains("install")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check terminal status".to_string());

        let plan = TerminalRecipe::build_check_status_plan(&telemetry).unwrap();

        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_configure_tmux_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "configure tmux".to_string());

        let plan = TerminalRecipe::build_configure_tmux_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("config") || g.to_lowercase().contains("tmux")));
        assert!(!plan.command_plan.is_empty());
    }

    #[test]
    fn test_list_tools_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "list terminal tools".to_string());

        let plan = TerminalRecipe::build_list_tools_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("list") || g.to_lowercase().contains("available")));
        assert!(!plan.command_plan.is_empty());
    }
}
