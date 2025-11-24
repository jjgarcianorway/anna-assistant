// Beta.169: Notification Daemon Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct NotificationsRecipe;

#[derive(Debug, PartialEq)]
enum NotificationsOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl NotificationsOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            NotificationsOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            NotificationsOperation::ListTools
        } else {
            NotificationsOperation::Install
        }
    }
}

impl NotificationsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("dunst") || input_lower.contains("mako")
            || input_lower.contains("notify-osd") || input_lower.contains("notification daemon")
            || input_lower.contains("notification server") || input_lower.contains("libnotify");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = NotificationsOperation::detect(user_input);
        match operation {
            NotificationsOperation::Install => Self::build_install_plan(telemetry),
            NotificationsOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            NotificationsOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("dunst") { "dunst" }
        else if input_lower.contains("mako") { "mako" }
        else if input_lower.contains("notify-osd") { "notify-osd" }
        else { "dunst" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "dunst" => ("Dunst", "dunst", "Lightweight notification daemon for X11 and Wayland"),
            "mako" => ("Mako", "mako", "Lightweight notification daemon for Wayland"),
            "notify-osd" => ("notify-osd", "notify-osd", "Canonical's on-screen display notification daemon"),
            _ => ("Dunst", "dunst", "Lightweight notification daemon"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("notifications.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {}. Start daemon manually or via window manager config.", tool_name, description);

        Ok(ActionPlan {
            analysis: format!("Installing {} notification daemon", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("notifications_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("notifications.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking notification daemon tools".to_string(),
            goals: vec!["List installed notification daemons".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-notification-tools".to_string(),
                    description: "List notification daemons".to_string(),
                    command: "pacman -Q dunst mako notify-osd 2>/dev/null || echo 'No notification daemons installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed notification daemon tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("notifications_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("notifications.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available notification daemon tools".to_string(),
            goals: vec!["List available notification daemons".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Notification Daemons:\n\nCross-Platform:\n- Dunst (official) - Lightweight, highly configurable (X11 and Wayland)\n- libnotify (official) - Desktop notification library (required by many apps)\n\nWayland:\n- Mako (official) - Lightweight notification daemon for Wayland\n- SwayNC (AUR) - Notification center for Sway\n\nX11:\n- notify-osd (official) - Canonicals notification daemon\n- xfce4-notifyd (official) - XFCE notification daemon\n\nGNOME/KDE:\n- GNOME Shell (built-in notification support)\n- KDE Plasma (built-in notification system)\n\nUsage:\n- Dunst: Start with dunst & (or add to autostart)\n- Mako: Start with mako & (or add to Sway config)\n- Test: notify-send Test Notification'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Notification daemons for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("notifications_list_tools".to_string()),
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
        assert!(NotificationsRecipe::matches_request("install dunst"));
        assert!(NotificationsRecipe::matches_request("install notification daemon"));
        assert!(NotificationsRecipe::matches_request("install notification server"));
        assert!(!NotificationsRecipe::matches_request("what is dunst"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install mako".to_string());
        let plan = NotificationsRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
