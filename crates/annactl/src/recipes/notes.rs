// Beta.171: Note-Taking Application Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct NotesRecipe;

#[derive(Debug, PartialEq)]
enum NotesOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl NotesOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            NotesOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            NotesOperation::ListTools
        } else {
            NotesOperation::Install
        }
    }
}

impl NotesRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("obsidian") || input_lower.contains("joplin")
            || input_lower.contains("typora") || input_lower.contains("zettlr")
            || input_lower.contains("note-taking") || input_lower.contains("notes app")
            || input_lower.contains("markdown editor");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = NotesOperation::detect(user_input);
        match operation {
            NotesOperation::Install => Self::build_install_plan(telemetry),
            NotesOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            NotesOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("obsidian") { "obsidian" }
        else if input_lower.contains("joplin") { "joplin" }
        else if input_lower.contains("typora") { "typora" }
        else if input_lower.contains("zettlr") { "zettlr" }
        else { "obsidian" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description, is_aur) = match tool {
            "obsidian" => ("Obsidian", "obsidian", "Markdown-based knowledge base with graph view and plugins", true),
            "joplin" => ("Joplin", "joplin", "Open-source note-taking and to-do app with sync", false),
            "typora" => ("Typora", "typora", "Minimal markdown editor with live preview", true),
            "zettlr" => ("Zettlr", "zettlr-bin", "Markdown editor for academic writing and knowledge management", true),
            _ => ("Obsidian", "obsidian", "Markdown knowledge base", true),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("notes.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = format!("{} installed. {}. Launch from app menu.", tool_name, description);

        let risk_level = if is_aur { RiskLevel::Medium } else { RiskLevel::Low };
        let requires_confirmation = is_aur;

        Ok(ActionPlan {
            analysis: format!("Installing {} note-taking app", tool_name),
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
                template_used: Some("notes_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("notes.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking note-taking application tools".to_string(),
            goals: vec!["List installed note-taking apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-notes-tools".to_string(),
                    description: "List note-taking apps".to_string(),
                    command: "pacman -Q obsidian joplin typora zettlr-bin 2>/dev/null || echo 'No note-taking apps installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed note-taking application tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("notes_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("notes.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available note-taking application tools".to_string(),
            goals: vec!["List available note-taking apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Note-Taking Applications:

Knowledge Base/Zettelkasten:
- Obsidian (AUR) - Markdown knowledge base with graph view, plugins, backlinks
- Zettlr (AUR) - Academic writing and knowledge management tool
- Logseq (AUR) - Privacy-first knowledge base with outliner and graph
- Notion (AUR) - All-in-one workspace (requires account)

General Note-Taking:
- Joplin (official) - Open-source notes with sync (Nextcloud, Dropbox, WebDAV)
- Simplenote (AUR) - Minimalist note-taking with sync
- Standard Notes (AUR) - Encrypted notes with extensions
- QOwnNotes (official) - Plain-text file markdown note-taking

Markdown Editors:
- Typora (AUR) - WYSIWYG markdown editor with live preview
- Mark Text (AUR) - Simple markdown editor
- ghostwriter (official) - Distraction-free markdown editor

Terminal/CLI:
- vim/neovim (official) - Text editor with markdown plugins
- nb (AUR) - CLI note-taking, bookmarking, archiving tool
- jrnl (official) - Simple journal application for command line

Comparison:
- Obsidian: Best for knowledge graphs, local-first, plugins
- Joplin: Best open-source option with sync
- Zettlr: Best for academic writing and citations
- Typora: Best for simple markdown editing

Features:
- Obsidian: Graph view, backlinks, plugins, community themes
- Joplin: E2E encryption, multiple sync targets, web clipper
- Zettlr: Citations, Zettelkasten, export to LaTeX/PDF
- Typora: Live preview, focus mode, export formats

Storage:
- Obsidian: Local markdown files in vault folder
- Joplin: SQLite database, markdown export available
- Zettlr: Plain markdown files
- Typora: Plain markdown files'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Note-taking applications for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("notes_list_tools".to_string()),
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
        assert!(NotesRecipe::matches_request("install obsidian"));
        assert!(NotesRecipe::matches_request("install note-taking app"));
        assert!(NotesRecipe::matches_request("setup joplin"));
        assert!(!NotesRecipe::matches_request("what is obsidian"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install typora".to_string());
        let plan = NotesRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
