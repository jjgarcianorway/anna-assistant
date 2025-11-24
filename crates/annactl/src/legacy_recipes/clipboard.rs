// Beta.169: Clipboard Manager Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ClipboardRecipe;

#[derive(Debug, PartialEq)]
enum ClipboardOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl ClipboardOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            ClipboardOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            ClipboardOperation::ListTools
        } else {
            ClipboardOperation::Install
        }
    }
}

impl ClipboardRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("clipman") || input_lower.contains("clipmenu")
            || input_lower.contains("copyq") || input_lower.contains("clipboard manager")
            || input_lower.contains("clipboard history") || input_lower.contains("clipster");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ClipboardOperation::detect(user_input);
        match operation {
            ClipboardOperation::Install => Self::build_install_plan(telemetry),
            ClipboardOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ClipboardOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("clipman") { "clipman" }
        else if input_lower.contains("clipmenu") { "clipmenu" }
        else if input_lower.contains("copyq") { "copyq" }
        else if input_lower.contains("clipster") { "clipster" }
        else { "copyq" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description, is_aur) = match tool {
            "clipman" => ("Clipman", "clipman", "Clipboard manager for Wayland (wlroots compositors)", false),
            "clipmenu" => ("Clipmenu", "clipmenu", "dmenu-based clipboard manager for X11", false),
            "copyq" => ("CopyQ", "copyq", "Advanced clipboard manager with searchable history", false),
            "clipster" => ("Clipster", "clipster", "Lightweight clipboard manager for X11", true),
            _ => ("CopyQ", "copyq", "Advanced clipboard manager", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("clipboard.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = format!("{} installed. {}. Start daemon and bind to keybinding.", tool_name, description);

        let risk_level = if is_aur { RiskLevel::Medium } else { RiskLevel::Low };
        let requires_confirmation = is_aur;

        Ok(ActionPlan {
            analysis: format!("Installing {} clipboard manager", tool_name),
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
                template_used: Some("clipboard_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("clipboard.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking clipboard manager tools".to_string(),
            goals: vec!["List installed clipboard managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-clipboard-tools".to_string(),
                    description: "List clipboard managers".to_string(),
                    command: "pacman -Q clipman clipmenu copyq clipster 2>/dev/null || echo 'No clipboard managers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed clipboard manager tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("clipboard_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("clipboard.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available clipboard manager tools".to_string(),
            goals: vec!["List available clipboard managers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Clipboard Managers:\n\nWayland:\n- Clipman (official) - Clipboard manager for wlroots compositors (Sway, Hyprland)\n- wl-clipboard (official) - Command-line clipboard utilities for Wayland\n\nX11:\n- Clipmenu (official) - dmenu-based clipboard manager\n- Clipster (AUR) - Lightweight clipboard manager\n- Parcellite (official) - Lightweight GTK+ clipboard manager\n\nCross-Platform:\n- CopyQ (official) - Advanced clipboard manager with searchable history, scripting\n- Diodon (AUR) - Lightweight clipboard manager for Unity/GNOME\n- GPaste (official) - GNOME clipboard manager\n\nUsage:\n- Clipman: Start with wl-paste -t text --watch clipman store\n- Clipmenu: Run clipmenud daemon, bind clipmenu to keybinding\n- CopyQ: Launch from app menu or run copyq in terminal'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Clipboard managers for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("clipboard_list_tools".to_string()),
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
        assert!(ClipboardRecipe::matches_request("install clipman"));
        assert!(ClipboardRecipe::matches_request("install clipboard manager"));
        assert!(ClipboardRecipe::matches_request("install clipboard history tool"));
        assert!(!ClipboardRecipe::matches_request("what is clipman"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install copyq".to_string());
        let plan = ClipboardRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
