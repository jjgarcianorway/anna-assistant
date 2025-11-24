// Beta.172: Calendar Application Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct CalendarRecipe;

#[derive(Debug, PartialEq)]
enum CalendarOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl CalendarOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            CalendarOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            CalendarOperation::ListTools
        } else {
            CalendarOperation::Install
        }
    }
}

impl CalendarRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("korganizer") || input_lower.contains("gnome-calendar")
            || input_lower.contains("calcurse") || input_lower.contains("orage")
            || input_lower.contains("calendar app") || input_lower.contains("calendar application");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = CalendarOperation::detect(user_input);
        match operation {
            CalendarOperation::Install => Self::build_install_plan(telemetry),
            CalendarOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            CalendarOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("korganizer") { "korganizer" }
        else if input_lower.contains("gnome-calendar") { "gnome-calendar" }
        else if input_lower.contains("calcurse") { "calcurse" }
        else if input_lower.contains("orage") { "orage" }
        else { "gnome-calendar" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "korganizer" => ("KOrganizer", "korganizer", "KDE calendar and scheduling application with CalDAV support"),
            "gnome-calendar" => ("GNOME Calendar", "gnome-calendar", "Simple GNOME calendar application with Evolution Data Server integration"),
            "calcurse" => ("calcurse", "calcurse", "Terminal-based calendar and scheduling application"),
            "orage" => ("Orage", "orage", "XFCE calendar and reminder application"),
            _ => ("GNOME Calendar", "gnome-calendar", "Simple calendar application"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("calendar.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);
        let notes = format!("{} installed. {}. Launch from app menu or run '{}'.",
            tool_name, description, package_name);

        Ok(ActionPlan {
            analysis: format!("Installing {} calendar application", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
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
                template_used: Some("calendar_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("calendar.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking calendar application tools".to_string(),
            goals: vec!["List installed calendar apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-calendar-tools".to_string(),
                    description: "List calendar apps".to_string(),
                    command: "pacman -Q korganizer gnome-calendar calcurse orage 2>/dev/null || echo 'No calendar apps installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed calendar application tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("calendar_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("calendar.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available calendar application tools".to_string(),
            goals: vec!["List available calendar apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Calendar Applications:

Desktop Calendar Apps:
- GNOME Calendar (official) - Simple calendar with Evolution Data Server integration
- KOrganizer (official) - KDE calendar and scheduling with CalDAV/CardDAV support
- Orage (official) - XFCE calendar with alarms and reminders
- Thunderbird (official) - Email client with Lightning calendar extension
- Evolution (official) - Email/calendar suite with Exchange support

Terminal/CLI Calendar Apps:
- calcurse (official) - ncurses calendar and scheduling application
- when (official) - Minimalist personal calendar
- khal (official) - CLI calendar application with CalDAV support
- vdirsyncer (AUR) - Sync calendars between devices and CalDAV servers

Web-Based Calendar (Self-Hosted):
- Nextcloud Calendar (via nextcloud-server) - Full-featured web calendar
- Radicale (AUR) - Lightweight CalDAV/CardDAV server
- Baikal (AUR) - CalDAV/CardDAV server built on PHP

Comparison:
- GNOME Calendar: Best for GNOME desktop, simple and clean
- KOrganizer: Best for KDE desktop, feature-rich with project management
- calcurse: Best for terminal users, lightweight and scriptable
- Thunderbird: Best if you want email + calendar in one app

Features:
- GNOME Calendar: Online accounts integration (Google, Nextcloud), evolution-data-server backend
- KOrganizer: Todo lists, journals, alarms, iCal export/import, group scheduling
- calcurse: Todo lists, appointments, iCal import/export, vim-style keybindings
- Thunderbird: Lightning extension, multiple calendars, email integration

Sync Support:
- GNOME Calendar: Google Calendar, Nextcloud, CalDAV servers
- KOrganizer: CalDAV, Google Calendar, Exchange
- calcurse: CalDAV via vdirsyncer or khal
- Thunderbird: CalDAV, Google Calendar (via add-on)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Calendar applications for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("calendar_list_tools".to_string()),
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
        assert!(CalendarRecipe::matches_request("install gnome-calendar"));
        assert!(CalendarRecipe::matches_request("install calendar app"));
        assert!(CalendarRecipe::matches_request("setup korganizer"));
        assert!(!CalendarRecipe::matches_request("what is korganizer"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install calcurse".to_string());
        let plan = CalendarRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
