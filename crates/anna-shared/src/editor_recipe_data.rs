//! Editor recipe data definitions.
//!
//! v0.0.95: Extracted from editor_recipes.rs for modularity.

use crate::editor_recipes::{ConfigFeature, ConfigLine, Editor, EditorRecipe};

/// Get recipe for a feature on an editor
pub fn get_recipe(editor: Editor, feature: ConfigFeature) -> Option<EditorRecipe> {
    let (lines, rollback) = match (editor, feature) {
        // VIM recipes
        (Editor::Vim, ConfigFeature::SyntaxHighlighting) => (
            vec![ConfigLine {
                line: "syntax on".to_string(),
                comment: "Enable syntax highlighting".to_string(),
                check_pattern: "^\\s*syntax\\s+(on|enable)".to_string(),
            }],
            "Remove 'syntax on' from ~/.vimrc",
        ),
        (Editor::Vim, ConfigFeature::LineNumbers) => (
            vec![ConfigLine {
                line: "set number".to_string(),
                comment: "Show line numbers".to_string(),
                check_pattern: "^\\s*set\\s+number".to_string(),
            }],
            "Remove 'set number' from ~/.vimrc",
        ),
        (Editor::Vim, ConfigFeature::WordWrap) => (
            vec![ConfigLine {
                line: "set wrap".to_string(),
                comment: "Enable word wrap".to_string(),
                check_pattern: "^\\s*set\\s+wrap".to_string(),
            }],
            "Change 'set wrap' to 'set nowrap' in ~/.vimrc",
        ),
        (Editor::Vim, ConfigFeature::AutoIndent) => (
            vec![
                ConfigLine {
                    line: "set autoindent".to_string(),
                    comment: "Enable auto-indent".to_string(),
                    check_pattern: "^\\s*set\\s+autoindent".to_string(),
                },
                ConfigLine {
                    line: "set smartindent".to_string(),
                    comment: "Enable smart indent".to_string(),
                    check_pattern: "^\\s*set\\s+smartindent".to_string(),
                },
            ],
            "Remove 'set autoindent' and 'set smartindent' from ~/.vimrc",
        ),

        // NEOVIM recipes (same as vim but different config path)
        (Editor::Neovim, ConfigFeature::SyntaxHighlighting) => (
            vec![ConfigLine {
                line: "syntax on".to_string(),
                comment: "Enable syntax highlighting".to_string(),
                check_pattern: "^\\s*syntax\\s+(on|enable)".to_string(),
            }],
            "Remove 'syntax on' from ~/.config/nvim/init.vim",
        ),
        (Editor::Neovim, ConfigFeature::LineNumbers) => (
            vec![ConfigLine {
                line: "set number".to_string(),
                comment: "Show line numbers".to_string(),
                check_pattern: "^\\s*set\\s+number".to_string(),
            }],
            "Remove 'set number' from ~/.config/nvim/init.vim",
        ),
        (Editor::Neovim, ConfigFeature::WordWrap) => (
            vec![ConfigLine {
                line: "set wrap".to_string(),
                comment: "Enable word wrap".to_string(),
                check_pattern: "^\\s*set\\s+wrap".to_string(),
            }],
            "Change 'set wrap' to 'set nowrap' in ~/.config/nvim/init.vim",
        ),

        // NANO recipes
        (Editor::Nano, ConfigFeature::SyntaxHighlighting) => (
            vec![ConfigLine {
                line: "include /usr/share/nano/*.nanorc".to_string(),
                comment: "Include syntax highlighting definitions".to_string(),
                check_pattern: "^\\s*include.*/nano/".to_string(),
            }],
            "Remove 'include /usr/share/nano/*.nanorc' from ~/.nanorc",
        ),
        (Editor::Nano, ConfigFeature::LineNumbers) => (
            vec![ConfigLine {
                line: "set linenumbers".to_string(),
                comment: "Show line numbers".to_string(),
                check_pattern: "^\\s*set\\s+linenumbers".to_string(),
            }],
            "Remove 'set linenumbers' from ~/.nanorc",
        ),
        (Editor::Nano, ConfigFeature::WordWrap) => (
            vec![ConfigLine {
                line: "set softwrap".to_string(),
                comment: "Enable soft word wrap".to_string(),
                check_pattern: "^\\s*set\\s+softwrap".to_string(),
            }],
            "Remove 'set softwrap' from ~/.nanorc",
        ),
        (Editor::Nano, ConfigFeature::AutoIndent) => (
            vec![ConfigLine {
                line: "set autoindent".to_string(),
                comment: "Enable auto-indent".to_string(),
                check_pattern: "^\\s*set\\s+autoindent".to_string(),
            }],
            "Remove 'set autoindent' from ~/.nanorc",
        ),

        // EMACS recipes
        (Editor::Emacs, ConfigFeature::SyntaxHighlighting) => (
            vec![ConfigLine {
                line: "(global-font-lock-mode t)".to_string(),
                comment: "Enable syntax highlighting globally".to_string(),
                check_pattern: "global-font-lock-mode".to_string(),
            }],
            "Remove '(global-font-lock-mode t)' from ~/.emacs",
        ),
        (Editor::Emacs, ConfigFeature::LineNumbers) => (
            vec![ConfigLine {
                line: "(global-display-line-numbers-mode t)".to_string(),
                comment: "Show line numbers globally".to_string(),
                check_pattern: "global-display-line-numbers-mode".to_string(),
            }],
            "Remove '(global-display-line-numbers-mode t)' from ~/.emacs",
        ),
        (Editor::Emacs, ConfigFeature::WordWrap) => (
            vec![ConfigLine {
                line: "(global-visual-line-mode t)".to_string(),
                comment: "Enable word wrap globally".to_string(),
                check_pattern: "global-visual-line-mode".to_string(),
            }],
            "Remove '(global-visual-line-mode t)' from ~/.emacs",
        ),

        // HELIX recipes (TOML config)
        (Editor::Helix, ConfigFeature::LineNumbers) => (
            vec![ConfigLine {
                line: "[editor]\nline-number = \"absolute\"".to_string(),
                comment: "Show absolute line numbers".to_string(),
                check_pattern: "line-number".to_string(),
            }],
            "Set line-number = \"off\" in ~/.config/helix/config.toml",
        ),
        (Editor::Helix, ConfigFeature::WordWrap) => (
            vec![ConfigLine {
                line: "[editor.soft-wrap]\nenable = true".to_string(),
                comment: "Enable soft word wrap".to_string(),
                check_pattern: "soft-wrap".to_string(),
            }],
            "Set enable = false under [editor.soft-wrap] in config.toml",
        ),

        // MICRO recipes (JSON config)
        (Editor::Micro, ConfigFeature::SyntaxHighlighting) => (
            vec![ConfigLine {
                line: "\"syntax\": true".to_string(),
                comment: "Enable syntax highlighting".to_string(),
                check_pattern: "\"syntax\"".to_string(),
            }],
            "Set \"syntax\": false in ~/.config/micro/settings.json",
        ),
        (Editor::Micro, ConfigFeature::LineNumbers) => (
            vec![ConfigLine {
                line: "\"ruler\": true".to_string(),
                comment: "Show line numbers".to_string(),
                check_pattern: "\"ruler\"".to_string(),
            }],
            "Set \"ruler\": false in ~/.config/micro/settings.json",
        ),
        (Editor::Micro, ConfigFeature::WordWrap) => (
            vec![ConfigLine {
                line: "\"softwrap\": true".to_string(),
                comment: "Enable soft word wrap".to_string(),
                check_pattern: "\"softwrap\"".to_string(),
            }],
            "Set \"softwrap\": false in ~/.config/micro/settings.json",
        ),

        // VS Code - limited support (suggest using GUI)
        (Editor::VsCode, _) => {
            return None; // VS Code settings are complex JSON, suggest GUI
        }

        // Kate and Gedit - limited support
        (Editor::Kate, _) | (Editor::Gedit, _) => {
            return None; // GUI editors, suggest using preferences
        }

        _ => return None,
    };

    Some(EditorRecipe {
        editor,
        feature,
        lines,
        needs_restart: false,
        rollback_hint: rollback.to_string(),
    })
}
