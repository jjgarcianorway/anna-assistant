// Beta.173: RSS/Feed Reader Applications Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct RssRecipe;

#[derive(Debug, PartialEq)]
enum RssOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl RssOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            RssOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            RssOperation::ListTools
        } else {
            RssOperation::Install
        }
    }
}

impl RssRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("newsboat") || input_lower.contains("liferea")
            || input_lower.contains("akregator") || input_lower.contains("rssguard")
            || input_lower.contains("rss reader") || input_lower.contains("feed reader");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = RssOperation::detect(user_input);
        match operation {
            RssOperation::Install => Self::build_install_plan(telemetry),
            RssOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            RssOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("newsboat") { "newsboat" }
        else if input_lower.contains("liferea") { "liferea" }
        else if input_lower.contains("akregator") { "akregator" }
        else if input_lower.contains("rssguard") { "rssguard" }
        else { "newsboat" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description, is_aur) = match tool {
            "newsboat" => ("Newsboat", "newsboat", "Terminal RSS/Atom feed reader with vim-like keybindings", false),
            "liferea" => ("Liferea", "liferea", "GTK news aggregator for online news feeds and blogs", false),
            "akregator" => ("Akregator", "akregator", "KDE RSS/Atom feed reader with Konqueror integration", false),
            "rssguard" => ("RSS Guard", "rssguard", "Simple RSS/Atom feed reader with notification support", true),
            _ => ("Newsboat", "newsboat", "Terminal RSS feed reader", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("rss.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = format!("{} installed. {}. Run '{}' to start reading feeds.",
            tool_name, description, package_name);

        let risk_level = if is_aur { RiskLevel::Medium } else { RiskLevel::Low };
        let requires_confirmation = is_aur;

        Ok(ActionPlan {
            analysis: format!("Installing {} RSS/feed reader", tool_name),
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
                template_used: Some("rss_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("rss.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking RSS/feed reader applications".to_string(),
            goals: vec!["List installed RSS/feed readers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-rss-readers".to_string(),
                    description: "List RSS readers".to_string(),
                    command: "pacman -Q newsboat liferea akregator rssguard 2>/dev/null || echo 'No RSS readers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed RSS/feed reader applications".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("rss_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("rss.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available RSS/feed reader applications".to_string(),
            goals: vec!["List available RSS readers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'RSS/Feed Reader Applications:

Terminal/CLI Readers:
- Newsboat (official) - Modern RSS/Atom reader with vim-like interface and podcast support
- Newsbeuter (deprecated) - Original RSS reader (replaced by Newsboat)
- CANTO (AUR) - Curses-based RSS aggregator with Python scripting
- Tuir (AUR) - Terminal UI for Reddit (browse Reddit like RSS)

Desktop GUI Readers:
- Liferea (official) - GTK news aggregator with folder organization and search
- Akregator (official) - KDE feed reader integrated with Konqueror and Kontact
- RSS Guard (AUR) - Simple Qt-based reader with notification support
- FeedReader (AUR) - Modern feed reader for GNOME with online service sync

Web-Based/Self-Hosted:
- Miniflux (AUR) - Minimalist web-based feed reader with keyboard shortcuts
- FreshRSS (AUR) - Self-hosted RSS aggregator with mobile support
- Tiny Tiny RSS (AUR) - Web-based news feed reader and aggregator
- NewsBlur (web) - Social news reader with intelligence training

Browser Extensions:
- Feedbro (Firefox/Chrome) - Browser-based RSS reader extension
- Brief (Firefox) - Minimalist RSS reader for Firefox
- Sage (Firefox) - Lightweight RSS and Atom feed reader

Comparison:
- Newsboat: Best for terminal users, fast, vim keybindings, scriptable
- Liferea: Best GTK/GNOME desktop reader with traditional UI
- Akregator: Best for KDE users with Kontact integration
- RSS Guard: Best lightweight Qt reader with modern features

Features:
- Newsboat: Macros, filters, podcasts, bookmarks, scriptable with Lua
- Liferea: Folder organization, search, plugins, social sharing
- Akregator: Konqueror integration, archive, filtering, notification
- RSS Guard: Labels, filters, notifications, read later, OPML import/export

Sync Support:
- Newsboat: Sync via Nextcloud News, The Old Reader, FreshRSS, newsboat-sync
- Liferea: Google Reader API compatible services (TheOldReader, FreshRSS)
- Akregator: Local only (can import OPML)
- RSS Guard: Local, TT-RSS, Inoreader, Nextcloud News, Google Reader API

Use Cases:
- Newsboat: Terminal workflow, SSH access, scripting, minimal resource usage
- Liferea: Desktop reading with traditional three-pane layout
- Akregator: KDE desktop integration with email and PIM
- Miniflux: Self-hosted solution with web and mobile access

Configuration:
- Newsboat: ~/.newsboat/config and ~/.newsboat/urls (plain text)
- Liferea: GUI settings, OPML import/export
- Akregator: KDE settings, feed folders
- RSS Guard: GUI settings, account management'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "RSS/Feed reader applications for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("rss_list_tools".to_string()),
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
        assert!(RssRecipe::matches_request("install newsboat"));
        assert!(RssRecipe::matches_request("install rss reader"));
        assert!(RssRecipe::matches_request("setup liferea"));
        assert!(!RssRecipe::matches_request("what is newsboat"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install akregator".to_string());
        let plan = RssRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
