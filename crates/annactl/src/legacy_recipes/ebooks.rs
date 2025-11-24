// Beta.173: Ebook Reader Applications Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct EbooksRecipe;

#[derive(Debug, PartialEq)]
enum EbooksOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl EbooksOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            EbooksOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            EbooksOperation::ListTools
        } else {
            EbooksOperation::Install
        }
    }
}

impl EbooksRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("calibre") || input_lower.contains("foliate")
            || input_lower.contains("zathura") || input_lower.contains("okular")
            || input_lower.contains("ebook reader") || input_lower.contains("epub reader")
            || input_lower.contains("pdf reader");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = EbooksOperation::detect(user_input);
        match operation {
            EbooksOperation::Install => Self::build_install_plan(telemetry),
            EbooksOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            EbooksOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("calibre") { "calibre" }
        else if input_lower.contains("foliate") { "foliate" }
        else if input_lower.contains("zathura") { "zathura" }
        else if input_lower.contains("okular") { "okular" }
        else { "foliate" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "calibre" => ("Calibre", "calibre", "Complete ebook library manager with format conversion and editing"),
            "foliate" => ("Foliate", "foliate", "Modern GTK ebook reader with EPUB support and customizable themes"),
            "zathura" => ("Zathura", "zathura", "Minimalist document viewer with vim-like keybindings"),
            "okular" => ("Okular", "okular", "KDE universal document viewer supporting PDF, EPUB, and many formats"),
            _ => ("Foliate", "foliate", "Modern ebook reader"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("ebooks.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);
        let notes = format!("{} installed. {}. Launch from app menu or run '{}'.",
            tool_name, description, package_name);

        Ok(ActionPlan {
            analysis: format!("Installing {} ebook reader", tool_name),
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
                template_used: Some("ebooks_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("ebooks.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking ebook reader applications".to_string(),
            goals: vec!["List installed ebook readers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-ebook-readers".to_string(),
                    description: "List ebook readers".to_string(),
                    command: "pacman -Q calibre foliate zathura okular 2>/dev/null || echo 'No ebook readers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed ebook reader applications".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("ebooks_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("ebooks.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available ebook reader applications".to_string(),
            goals: vec!["List available ebook readers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Ebook Reader Applications:

Full-Featured Library Managers:
- Calibre (official) - Complete ebook management with library, conversion, editing, and sync
- Calibre Web (AUR) - Web-based interface for Calibre library access

Modern GUI Readers:
- Foliate (official) - GTK ebook reader with clean interface, EPUB/MOBI/PDF support
- Bookworm (AUR) - Simple ebook reader for GNOME with minimal interface
- Komga (AUR) - Web-based comic/manga/book server and reader

Traditional Document Viewers:
- Okular (official) - KDE universal document viewer (PDF, EPUB, MOBI, DjVu, CHM)
- Evince (official) - GNOME document viewer (PDF, EPUB, DjVu, TIFF)
- Atril (official) - MATE document viewer (lightweight Evince fork)

Minimalist/Terminal Readers:
- Zathura (official) - Vim-like document viewer with keyboard focus
- FBReader (AUR) - Cross-platform ebook reader with plugin support
- Epy (AUR) - Terminal EPUB reader written in Python

Web-Based/Server Solutions:
- Kavita (AUR) - Cross-platform manga/comic/book server with web interface
- Ubooquity (AUR) - Home server for comics and ebooks
- Polar Bookshelf (AUR) - Document manager for web and PDF content

Comparison:
- Calibre: Best all-around tool for library management and conversion
- Foliate: Best modern reading experience with clean UI
- Okular: Best for KDE users, supports most formats
- Zathura: Best for keyboard-driven minimalist workflow

Features:
- Calibre: Format conversion, metadata editing, device sync, news download, plugins
- Foliate: Custom themes, annotations, dictionary lookup, TTS support
- Okular: Annotations, forms, signatures, accessibility features
- Zathura: Vim keybindings, customizable, lightweight, scriptable

Format Support:
- Calibre: EPUB, MOBI, AZW, PDF, CBZ, CBR, + convert between 20+ formats
- Foliate: EPUB, MOBI, AZW, FB2, CBZ, PDF
- Okular: PDF, EPUB, MOBI, DjVu, CHM, XPS, PostScript
- Zathura: PDF, EPUB, DjVu, PostScript (via plugins)

Library Management:
- Calibre: Full library with tags, series, authors, collections, search
- Foliate: Basic library with reading progress tracking
- Okular: Recent files only, no library management
- Zathura: No library management (file-based)

Use Cases:
- Calibre: Managing large ebook collections, format conversion, device sync
- Foliate: Casual reading with pleasant interface
- Okular: Academic reading with annotations and research
- Zathura: Fast document viewing with keyboard navigation'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Ebook reader applications for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("ebooks_list_tools".to_string()),
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
        assert!(EbooksRecipe::matches_request("install calibre"));
        assert!(EbooksRecipe::matches_request("install ebook reader"));
        assert!(EbooksRecipe::matches_request("setup foliate"));
        assert!(!EbooksRecipe::matches_request("what is calibre"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install zathura".to_string());
        let plan = EbooksRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
