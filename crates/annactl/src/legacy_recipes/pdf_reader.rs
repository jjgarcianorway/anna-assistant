// Beta.167: PDF Readers Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct PdfReaderRecipe;

#[derive(Debug, PartialEq)]
enum PdfReaderOperation {
    Install,
    CheckStatus,
    ListReaders,
}

impl PdfReaderOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            PdfReaderOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            PdfReaderOperation::ListReaders
        } else {
            PdfReaderOperation::Install
        }
    }
}

impl PdfReaderRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("evince") || input_lower.contains("okular")
            || input_lower.contains("zathura") || input_lower.contains("mupdf")
            || input_lower.contains("pdf reader") || input_lower.contains("pdf viewer")
            || input_lower.contains("document viewer");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = PdfReaderOperation::detect(user_input);
        match operation {
            PdfReaderOperation::Install => Self::build_install_plan(telemetry),
            PdfReaderOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            PdfReaderOperation::ListReaders => Self::build_list_readers_plan(telemetry),
        }
    }

    fn detect_reader(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("evince") { "evince" }
        else if input_lower.contains("okular") { "okular" }
        else if input_lower.contains("zathura") { "zathura" }
        else if input_lower.contains("mupdf") { "mupdf" }
        else { "evince" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let reader = Self::detect_reader(user_input);
        let (reader_name, package_name, desktop_env, is_gui) = match reader {
            "evince" => ("Evince", "evince", "GNOME", true),
            "okular" => ("Okular", "okular", "KDE", true),
            "zathura" => ("Zathura", "zathura", "Minimal/Vim-like", true),
            "mupdf" => ("MuPDF", "mupdf", "Command-line", false),
            _ => ("Evince", "evince", "GNOME", true),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("pdf_reader.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("reader".to_string(), serde_json::json!(reader_name));

        let notes = if is_gui {
            format!("{} installed. {} PDF reader. Launch from application menu or open PDF files.", reader_name, desktop_env)
        } else {
            format!("{} installed. {} PDF reader. Open with: mupdf <file.pdf>", reader_name, desktop_env)
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} PDF reader", reader_name),
            goals: vec![format!("Install {}", reader_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", reader),
                    description: format!("Install {}", reader_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", reader)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", reader),
                    description: format!("Remove {}", reader_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("pdf_reader_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("pdf_reader.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking PDF readers".to_string(),
            goals: vec!["List installed PDF readers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-pdf-readers".to_string(),
                    description: "List PDF readers".to_string(),
                    command: "pacman -Q evince okular zathura mupdf xreader 2>/dev/null || echo 'No PDF readers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed PDF readers".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("pdf_reader_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_readers_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("pdf_reader.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListReaders"));

        Ok(ActionPlan {
            analysis: "Showing available PDF readers".to_string(),
            goals: vec!["List available PDF readers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-readers".to_string(),
                    description: "Show available readers".to_string(),
                    command: r"echo 'GUI PDF Readers:\n- Evince (official) - GNOME document viewer\n- Okular (official) - KDE universal document viewer\n- Zathura (official) - Minimal, vim-like keybindings\n- Xreader (official) - X-Apps document reader\n\nCommand-line PDF Readers:\n- MuPDF (official) - Lightweight and fast\n- pdftotext (official) - Extract text from PDFs'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "PDF readers for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("pdf_reader_list_readers".to_string()),
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
        assert!(PdfReaderRecipe::matches_request("install evince"));
        assert!(PdfReaderRecipe::matches_request("install okular"));
        assert!(PdfReaderRecipe::matches_request("install pdf reader"));
        assert!(!PdfReaderRecipe::matches_request("what is evince"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install zathura".to_string());
        let plan = PdfReaderRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
