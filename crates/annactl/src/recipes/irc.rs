// Beta.171: IRC/Chat Client Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct IrcRecipe;

#[derive(Debug, PartialEq)]
enum IrcOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl IrcOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            IrcOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            IrcOperation::ListTools
        } else {
            IrcOperation::Install
        }
    }
}

impl IrcRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("weechat") || input_lower.contains("irssi")
            || input_lower.contains("hexchat") || input_lower.contains("pidgin")
            || input_lower.contains("irc client") || input_lower.contains("chat client")
            || input_lower.contains("irc");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = IrcOperation::detect(user_input);
        match operation {
            IrcOperation::Install => Self::build_install_plan(telemetry),
            IrcOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            IrcOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("weechat") { "weechat" }
        else if input_lower.contains("irssi") { "irssi" }
        else if input_lower.contains("hexchat") { "hexchat" }
        else if input_lower.contains("pidgin") { "pidgin" }
        else { "weechat" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "weechat" => ("WeeChat", "weechat", "Extensible terminal IRC client with scripts and plugins"),
            "irssi" => ("Irssi", "irssi", "Classic terminal-based IRC client"),
            "hexchat" => ("HexChat", "hexchat", "GTK-based IRC client with GUI"),
            "pidgin" => ("Pidgin", "pidgin", "Multi-protocol chat client (IRC, XMPP, etc.)"),
            _ => ("WeeChat", "weechat", "Extensible terminal IRC client"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("irc.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {}. Run {} in terminal to start.", tool_name, description, tool);

        Ok(ActionPlan {
            analysis: format!("Installing {} IRC client", tool_name),
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
                template_used: Some("irc_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("irc.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking IRC client tools".to_string(),
            goals: vec!["List installed IRC clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-irc-tools".to_string(),
                    description: "List IRC clients".to_string(),
                    command: "pacman -Q weechat irssi hexchat pidgin 2>/dev/null || echo 'No IRC clients installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed IRC client tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("irc_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("irc.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available IRC client tools".to_string(),
            goals: vec!["List available IRC clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'IRC/Chat Clients:

Terminal Clients:
- WeeChat (official) - Extensible IRC client with script support, plugins, relay mode
- Irssi (official) - Classic modular IRC client with Perl scripting
- BitchX (AUR) - Advanced ircII-based client with features
- sic (official) - Simple IRC client from suckless

GUI Clients:
- HexChat (official) - GTK-based IRC client, fork of XChat
- Konversation (official) - KDE IRC client with notifications, DCC
- Quassel (official) - Distributed IRC client (core/client separation)
- KVIrc (official) - Qt-based IRC client with scripting

Multi-Protocol:
- Pidgin (official) - Multi-protocol chat (IRC, XMPP, AIM, etc.)
- Gajim (official) - XMPP/Jabber client with IRC plugin
- Polari (official) - GNOME IRC client, simple and modern
- Empathy (official) - GNOME multi-protocol chat client

Modern IRC (IRCv3):
- Srain (AUR) - Modern IRC client with GTK UI
- Thunderbird (official) - Email client with IRC/Matrix chat support

Features Comparison:
- WeeChat: Best for terminal, highly scriptable, relay/bouncer mode
- Irssi: Classic terminal client, Perl scripts, screen/tmux friendly
- HexChat: Best GUI client, simple interface, channel management
- Konversation: KDE integration, OSD notifications, scripting

Configuration:
- WeeChat: Run weechat, /server add, /connect
- Irssi: Run irssi, /server add, /connect, /save
- HexChat: Launch from menu, Network List dialog for servers
- Pidgin: Add Accounts dialog, select IRC protocol'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "IRC/chat clients for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("irc_list_tools".to_string()),
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
        assert!(IrcRecipe::matches_request("install weechat"));
        assert!(IrcRecipe::matches_request("install irc client"));
        assert!(IrcRecipe::matches_request("setup hexchat"));
        assert!(!IrcRecipe::matches_request("what is irssi"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install pidgin".to_string());
        let plan = IrcRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
