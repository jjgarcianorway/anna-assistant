// Beta.174: LaTeX Document Preparation System Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct LatexRecipe;

#[derive(Debug, PartialEq)]
enum LatexOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl LatexOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            LatexOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            LatexOperation::ListTools
        } else {
            LatexOperation::Install
        }
    }
}

impl LatexRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("latex") || input_lower.contains("tex live")
            || input_lower.contains("texlive") || input_lower.contains("texstudio")
            || input_lower.contains("lyx") || input_lower.contains("texmaker")
            || input_lower.contains("tex editor");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = LatexOperation::detect(user_input);
        match operation {
            LatexOperation::Install => Self::build_install_plan(telemetry),
            LatexOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            LatexOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("texstudio") { "texstudio" }
        else if input_lower.contains("lyx") { "lyx" }
        else if input_lower.contains("texmaker") { "texmaker" }
        else if input_lower.contains("texlive") || input_lower.contains("tex live") { "texlive" }
        else { "texlive-core" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "texlive" => ("TeX Live (Full)", "texlive", "Complete TeX Live distribution with all packages"),
            "texlive-core" => ("TeX Live Core", "texlive-core", "Essential TeX Live packages for basic document compilation"),
            "texstudio" => ("TeXstudio", "texstudio", "Feature-rich LaTeX editor with integrated PDF viewer"),
            "lyx" => ("LyX", "lyx", "WYSIWYM document processor with LaTeX backend"),
            "texmaker" => ("Texmaker", "texmaker", "Cross-platform LaTeX editor with integrated viewer"),
            _ => ("TeX Live Core", "texlive-core", "LaTeX document preparation system"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("latex.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);
        let notes = format!("{} installed. {}. For editors like TeXstudio, run the application from your desktop environment.",
            tool_name, description);

        Ok(ActionPlan {
            analysis: format!("Installing {} LaTeX system", tool_name),
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
                template_used: Some("latex_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("latex.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking LaTeX installation".to_string(),
            goals: vec!["List installed LaTeX tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-latex-tools".to_string(),
                    description: "List LaTeX tools".to_string(),
                    command: "pacman -Q texlive-core texlive texstudio lyx texmaker 2>/dev/null || echo 'No LaTeX tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed LaTeX distribution and editors".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("latex_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("latex.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available LaTeX tools".to_string(),
            goals: vec!["List available LaTeX distributions and editors".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'LaTeX Document Preparation System:

TeX Distributions:
- TeX Live Core (official) - Essential TeX Live packages for basic document compilation
- TeX Live (official) - Complete TeX Live distribution with all packages and tools
- TeX Live Most (official) - TeX Live with most common packages (smaller than full)
- TeX Live Lang (official) - Language-specific TeX Live packages

Full-Featured Editors:
- TeXstudio (official) - Feature-rich editor with syntax highlighting, code completion, integrated viewer
- Texmaker (official) - Cross-platform editor with quick build, integrated viewer, structure view
- TeXworks (official) - Simple editor inspired by TeXShop, integrated previewer
- Kile (official) - KDE integrated LaTeX editor with project management

WYSIWYM Processors:
- LyX (official) - WYSIWYM document processor, visual editing with LaTeX backend
- TeXmacs (AUR) - Scientific text editor with WYSIWYG interface

Lightweight Editors:
- Gummi (AUR) - Lightweight editor with live preview
- TeXpen (AUR) - Minimal LaTeX editor

Online/Collaborative:
- Overleaf (web) - Online collaborative LaTeX editor
- CoCalc (web) - Collaborative calculation and LaTeX environment

Vim/Emacs Integration:
- vimtex (vim plugin) - LaTeX plugin for Vim with completion and compilation
- AUCTeX (emacs package) - Emacs mode for LaTeX with preview and folding

Comparison:
- TeXstudio: Best for feature-rich desktop editing with advanced completion
- LyX: Best for WYSIWYM editing without learning LaTeX syntax
- Texmaker: Best lightweight full-featured editor
- TeX Live: Best complete distribution for serious LaTeX work

Features:
- TeXstudio: Syntax highlighting, code completion, 1000+ math symbols, grammar checker, bibliography management
- LyX: WYSIWYM interface, math editor, table editor, graphics, bibliography, presentation support
- Texmaker: Quick build, master document support, structure view, integrated viewer, spell checking
- TeX Live: Complete LaTeX distribution, thousands of packages, fonts, bibliography tools

Package Management:
- TeX Live: Use tlmgr for package management (texlive-most includes tlmgr)
- Arch packages: Install additional texlive-* packages via pacman
- CTAN: Comprehensive TeX Archive Network for all LaTeX packages

Distribution Sizes:
- texlive-core: ~200 MB (essential packages only)
- texlive-most: ~1.5 GB (most common packages)
- texlive (full): ~5 GB (all packages and documentation)

Use Cases:
- Academic papers: TeXstudio or Texmaker with texlive-most
- Books/Thesis: TeXstudio with texlive (full) for all packages
- Quick documents: LyX for WYSIWYM without LaTeX knowledge
- Presentations: Beamer class with TeXstudio or LyX

Essential Packages:
- texlive-core: Basic LaTeX compilation
- texlive-latexextra: Additional LaTeX packages and classes
- texlive-fontsextra: Extra fonts
- texlive-bibtexextra: Bibliography tools
- texlive-science: Scientific and technical packages

Configuration:
- TeXstudio: Settings → Configure TeXstudio (custom commands, shortcuts, themes)
- LyX: Tools → Preferences (appearance, editing, LaTeX preamble)
- Texmaker: Options → Configure Texmaker (editor, commands, shortcuts)
- TeX Live: Managed via tlmgr or pacman packages'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "LaTeX distributions and editors for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("latex_list_tools".to_string()),
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
        assert!(LatexRecipe::matches_request("install latex"));
        assert!(LatexRecipe::matches_request("install texlive"));
        assert!(LatexRecipe::matches_request("setup texstudio"));
        assert!(!LatexRecipe::matches_request("what is latex"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install texstudio".to_string());
        let plan = LatexRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
