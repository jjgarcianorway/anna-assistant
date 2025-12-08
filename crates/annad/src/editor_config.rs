//! Editor configuration answer builders (v0.0.147).
//!
//! Extracted from rpc_handler.rs to meet 400-line limit.
//! Builds answers for editor syntax highlighting queries.

use anna_shared::change::ChangePlan;
use tracing::warn;

/// Build editor config answer (informational only, no changes)
pub fn build_editor_config_answer(editor: &str) -> String {
    use anna_shared::editor_recipes::{get_recipe, ConfigFeature, Editor};

    // Try to get recipe from EditorRecipes module
    if let Some(editor_enum) = Editor::from_tool_name(editor) {
        if let Some(recipe) = get_recipe(editor_enum, ConfigFeature::SyntaxHighlighting) {
            // Build answer from recipe
            let lines: Vec<String> = recipe
                .lines
                .iter()
                .map(|l| format!("   {}", l.line))
                .collect();

            return format!(
                "Detected {} installed. To enable syntax highlighting:\n\
                1. Edit ~/{}\n\
                2. Add the following:\n{}\n\
                3. Save and reopen {}\n\n\
                To undo: {}",
                editor_enum.display_name(),
                editor_enum.config_path(),
                lines.join("\n"),
                editor,
                recipe.rollback_hint
            );
        }
    }

    // Fallback for GUI editors without recipes (VS Code, Kate, Gedit)
    match editor {
        "code" => String::from(
            "Detected VS Code installed. Syntax highlighting is automatic based on file type.\n\
            To configure:\n\
            1. Open a file - VS Code detects language from extension\n\
            2. Click language indicator (bottom-right) to change mode\n\
            3. Install language extensions for better support (Ctrl+Shift+X)\n\
            Theme: File > Preferences > Color Theme",
        ),
        "kate" => String::from(
            "Detected Kate installed. Syntax highlighting is enabled by default.\n\
            To configure:\n\
            1. Settings > Configure Kate > Fonts & Colors\n\
            2. Select a color scheme\n\
            Line numbers: Settings > Configure Kate > Appearance > Show line numbers",
        ),
        "gedit" => String::from(
            "Detected gedit installed. Syntax highlighting is enabled by default.\n\
            To configure:\n\
            1. Preferences > Font & Colors\n\
            2. Select a color scheme\n\
            Line numbers: Preferences > View > Display line numbers",
        ),
        "helix" | "hx" => String::from(
            "Detected Helix installed. Syntax highlighting is enabled by default.\n\
            To customize themes:\n\
            1. Edit ~/.config/helix/config.toml\n\
            2. Add: theme = \"gruvbox\" (or another theme name)\n\
            3. Save and reopen helix\n\
            List themes with :theme command inside helix",
        ),
        "micro" => String::from(
            "Detected micro installed. Syntax highlighting is enabled by default.\n\
            To customize:\n\
            1. Edit ~/.config/micro/settings.json\n\
            2. Set \"colorscheme\": \"monokai\" (or another scheme)\n\
            For line numbers: set \"ruler\": true",
        ),
        _ => format!(
            "Detected {} installed. Check its documentation for syntax highlighting configuration.",
            editor
        ),
    }
}

/// Build editor config answer WITH a proposed ChangePlan.
/// Returns (answer_text, Option<ChangePlan>, Vec<ChangePlan>) for Safe Change Engine integration.
pub fn build_editor_config_with_change(
    editor: &str,
) -> (String, Option<ChangePlan>, Vec<ChangePlan>) {
    use anna_shared::change::{plan_ensure_line, plan_ensure_line_with_pattern};
    use anna_shared::editor_recipes::{get_recipe, ConfigFeature, Editor};

    // Try to get recipe from EditorRecipes module
    if let Some(editor_enum) = Editor::from_tool_name(editor) {
        if let Some(recipe) = get_recipe(editor_enum, ConfigFeature::SyntaxHighlighting) {
            // Get the config file path
            let config_file = editor_enum.config_file();

            // Build change plans for all recipe lines (supports multi-line recipes)
            let mut proposed_changes: Vec<ChangePlan> = Vec::new();
            for line in &recipe.lines {
                let plan_result = if !line.check_pattern.is_empty() {
                    plan_ensure_line_with_pattern(&config_file, &line.line, &line.check_pattern)
                } else {
                    plan_ensure_line(&config_file, &line.line)
                };

                match plan_result {
                    Ok(plan) => proposed_changes.push(plan),
                    Err(e) => warn!("Could not plan change for {}: {}", editor, e),
                }
            }

            let proposed_change = proposed_changes.first().cloned();

            // Build answer from recipe
            let lines: Vec<String> = recipe
                .lines
                .iter()
                .map(|l| format!("   {}", l.line))
                .collect();

            let answer = if proposed_change.is_some() {
                format!(
                    "I can enable syntax highlighting for {}.\n\n\
                    This will add the following to ~/{}:\n{}\n\n\
                    Reply 'yes' to apply this change, or 'no' to cancel.\n\
                    To undo later: {}",
                    editor_enum.display_name(),
                    editor_enum.config_path(),
                    lines.join("\n"),
                    recipe.rollback_hint
                )
            } else {
                // Fallback to manual instructions if change planning failed
                format!(
                    "Detected {} installed. To enable syntax highlighting:\n\
                    1. Edit ~/{}\n\
                    2. Add the following:\n{}\n\
                    3. Save and reopen {}\n\n\
                    To undo: {}",
                    editor_enum.display_name(),
                    editor_enum.config_path(),
                    lines.join("\n"),
                    editor,
                    recipe.rollback_hint
                )
            };

            return (answer, proposed_change, proposed_changes);
        }
    }

    // Fallback for GUI editors - no automated change possible
    let answer = build_editor_config_answer(editor);
    (answer, None, Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.0.57: Vim answer must not contain question marks.
    #[test]
    fn test_vim_answer_no_questions() {
        let answer = build_editor_config_answer("vim");
        assert!(answer.contains("vim"), "Must mention vim");
        assert!(answer.contains(".vimrc"), "Must mention .vimrc");
        assert!(!answer.contains('?'), "Must not contain question marks");
    }

    /// v0.0.57: Nvim answer has correct config paths.
    #[test]
    fn test_nvim_answer_correct_paths() {
        let answer = build_editor_config_answer("nvim");
        assert!(answer.contains("nvim"), "Must mention nvim");
        assert!(
            answer.contains("init.vim") || answer.contains("init.lua"),
            "Must mention nvim config file"
        );
        assert!(!answer.contains('?'), "Must not contain question marks");
    }

    /// v0.0.57: Each editor has specific answer without other editors' paths.
    #[test]
    fn test_editor_answers_are_specific() {
        let editors = [
            "vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit",
        ];

        for editor in editors {
            let answer = build_editor_config_answer(editor);

            // Must mention the detected editor
            assert!(
                answer.to_lowercase().contains(editor)
                    || (editor == "code" && answer.contains("VS Code")),
                "Answer for {} must mention the editor",
                editor
            );

            // Must NOT contain question marks
            assert!(
                !answer.contains('?'),
                "Answer for {} must not contain question marks",
                editor
            );

            // Answers should be distinct (not generic)
            assert!(
                answer.len() > 100,
                "Answer for {} should be detailed (got {} chars)",
                editor,
                answer.len()
            );
        }
    }

    /// v0.0.57: vi is treated like vim.
    #[test]
    fn test_vi_uses_vim_config() {
        let answer = build_editor_config_answer("vi");
        assert!(answer.contains(".vimrc"), "vi should use vim config");
    }

    /// v0.0.66: Editor answers must not use markdown formatting.
    #[test]
    fn test_v066_editor_answers_no_markdown() {
        let editors = [
            "vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit",
        ];

        for editor in editors {
            let answer = build_editor_config_answer(editor);
            // No markdown bold
            assert!(
                !answer.contains("**"),
                "Answer for {} must not contain markdown bold **",
                editor
            );
        }
    }

    /// v0.0.66: Editor answers start with "Detected" statement.
    #[test]
    fn test_v066_editor_answers_start_with_detected() {
        let editors = [
            "vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit",
        ];

        for editor in editors {
            let answer = build_editor_config_answer(editor);
            assert!(
                answer.starts_with("Detected"),
                "Answer for {} must start with 'Detected', got: {}",
                editor,
                &answer[..40.min(answer.len())]
            );
        }
    }
}
