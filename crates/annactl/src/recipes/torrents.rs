// Beta.170: Torrent Client Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct TorrentsRecipe;

#[derive(Debug, PartialEq)]
enum TorrentsOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl TorrentsOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            TorrentsOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            TorrentsOperation::ListTools
        } else {
            TorrentsOperation::Install
        }
    }
}

impl TorrentsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("qbittorrent") || input_lower.contains("transmission")
            || input_lower.contains("deluge") || input_lower.contains("rtorrent")
            || input_lower.contains("torrent client") || input_lower.contains("bittorrent");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = TorrentsOperation::detect(user_input);
        match operation {
            TorrentsOperation::Install => Self::build_install_plan(telemetry),
            TorrentsOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            TorrentsOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("qbittorrent") { "qbittorrent" }
        else if input_lower.contains("transmission") { "transmission-gtk" }
        else if input_lower.contains("deluge") { "deluge" }
        else if input_lower.contains("rtorrent") { "rtorrent" }
        else { "qbittorrent" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "qbittorrent" => ("qBittorrent", "qbittorrent", "Feature-rich BitTorrent client with web interface"),
            "transmission-gtk" => ("Transmission", "transmission-gtk", "Lightweight BitTorrent client with GTK UI"),
            "deluge" => ("Deluge", "deluge", "Python-based BitTorrent client with plugin support"),
            "rtorrent" => ("rTorrent", "rtorrent", "Terminal-based BitTorrent client"),
            _ => ("qBittorrent", "qbittorrent", "Feature-rich BitTorrent client"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("torrents.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {}. Launch from app menu or run in terminal.", tool_name, description);

        Ok(ActionPlan {
            analysis: format!("Installing {} torrent client", tool_name),
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
                template_used: Some("torrents_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("torrents.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking torrent client tools".to_string(),
            goals: vec!["List installed torrent clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-torrent-tools".to_string(),
                    description: "List torrent clients".to_string(),
                    command: "pacman -Q qbittorrent transmission-gtk deluge rtorrent 2>/dev/null || echo 'No torrent clients installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed torrent client tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("torrents_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("torrents.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available torrent client tools".to_string(),
            goals: vec!["List available torrent clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Torrent Clients:

GUI Clients:
- qBittorrent (official) - Feature-rich client with built-in search, web UI, RSS support
- Transmission GTK (official) - Lightweight, simple interface, low resource usage
- Transmission Qt (official) - Qt version of Transmission
- Deluge (official) - Python-based client with plugin system and web UI
- Fragments (official) - Modern GTK4 torrent client for GNOME
- KTorrent (official) - KDE torrent client with advanced features

Terminal Clients:
- rTorrent (official) - Powerful ncurses-based client with tmux/screen support
- Transmission CLI (official) - Command-line version of Transmission
- aria2 (official) - Multi-protocol download utility supporting torrents

Headless/Server:
- qBittorrent-nox (official) - qBittorrent without GUI, web UI only
- Transmission daemon (official) - Server version with web UI and RPC
- Deluge daemon (official) - Background daemon with web/console clients

Features Comparison:
- qBittorrent: Best overall, web UI, search, RSS, sequential downloads
- Transmission: Simplest, fastest, lowest resource usage
- Deluge: Plugin support, multiple UI frontends, label system
- rTorrent: Terminal power users, scriptable, lightweight

Usage:
- qBittorrent: Launch from app menu, configure in preferences
- Transmission: Launch from app menu, very simple interface
- rTorrent: Run rtorrent in terminal, configure ~/.rtorrent.rc
- Web UIs: Access at localhost:8080 (qBittorrent) or localhost:9091 (Transmission)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Torrent clients for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("torrents_list_tools".to_string()),
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
        assert!(TorrentsRecipe::matches_request("install qbittorrent"));
        assert!(TorrentsRecipe::matches_request("install torrent client"));
        assert!(TorrentsRecipe::matches_request("setup transmission"));
        assert!(!TorrentsRecipe::matches_request("what is qbittorrent"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install deluge".to_string());
        let plan = TorrentsRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
