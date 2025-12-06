//! Editor configuration recipes.
//!
//! v0.0.74: Generic, safe, idempotent recipes for editor configuration.
//!
//! # Supported Editors
//! - vim, neovim
//! - nano
//! - emacs
//! - helix
//! - micro
//! - VS Code (limited)
//!
//! # Safety
//! - All edits require user confirmation
//! - Backups are created before editing
//! - Edits are idempotent (safe to apply multiple times)
//! - Rollback instructions provided

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported editor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    Vim,
    Neovim,
    Nano,
    Emacs,
    Helix,
    Micro,
    VsCode,
    Kate,
    Gedit,
}

impl Editor {
    /// Get canonical name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            Editor::Vim => "Vim",
            Editor::Neovim => "Neovim",
            Editor::Nano => "nano",
            Editor::Emacs => "Emacs",
            Editor::Helix => "Helix",
            Editor::Micro => "micro",
            Editor::VsCode => "VS Code",
            Editor::Kate => "Kate",
            Editor::Gedit => "gedit",
        }
    }

    /// Get config file path (relative to home)
    pub fn config_path(&self) -> &'static str {
        match self {
            Editor::Vim => ".vimrc",
            Editor::Neovim => ".config/nvim/init.vim",
            Editor::Nano => ".nanorc",
            Editor::Emacs => ".emacs",
            Editor::Helix => ".config/helix/config.toml",
            Editor::Micro => ".config/micro/settings.json",
            Editor::VsCode => ".config/Code/User/settings.json",
            Editor::Kate => ".config/katerc",
            Editor::Gedit => ".config/gedit/preferences.xml",
        }
    }

    /// Get absolute config file path
    pub fn config_file(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        PathBuf::from(home).join(self.config_path())
    }

    /// Parse from tool name
    pub fn from_tool_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "vim" | "vi" => Some(Editor::Vim),  // vi uses vim config
            "nvim" | "neovim" => Some(Editor::Neovim),
            "nano" => Some(Editor::Nano),
            "emacs" => Some(Editor::Emacs),
            "hx" | "helix" => Some(Editor::Helix),
            "micro" => Some(Editor::Micro),
            "code" | "vscode" => Some(Editor::VsCode),
            "kate" => Some(Editor::Kate),
            "gedit" => Some(Editor::Gedit),
            _ => None,
        }
    }
}

impl std::fmt::Display for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Configuration feature type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFeature {
    /// Enable syntax highlighting
    SyntaxHighlighting,
    /// Show line numbers
    LineNumbers,
    /// Enable word wrap
    WordWrap,
    /// Set indentation (spaces)
    Indentation,
    /// Enable auto-indent
    AutoIndent,
    /// Show whitespace
    ShowWhitespace,
}

impl ConfigFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            ConfigFeature::SyntaxHighlighting => "syntax highlighting",
            ConfigFeature::LineNumbers => "line numbers",
            ConfigFeature::WordWrap => "word wrap",
            ConfigFeature::Indentation => "indentation",
            ConfigFeature::AutoIndent => "auto-indent",
            ConfigFeature::ShowWhitespace => "show whitespace",
        }
    }
}

impl std::fmt::Display for ConfigFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A configuration line to add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLine {
    /// The line to add
    pub line: String,
    /// Comment explaining the line
    pub comment: String,
    /// Pattern to check if already present (for idempotency)
    pub check_pattern: String,
}

/// A recipe for configuring an editor feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorRecipe {
    /// Target editor
    pub editor: Editor,
    /// Feature being configured
    pub feature: ConfigFeature,
    /// Lines to add to config
    pub lines: Vec<ConfigLine>,
    /// Whether a restart/reload is needed
    pub needs_restart: bool,
    /// Rollback instructions
    pub rollback_hint: String,
}

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

/// Check if a line already exists in config (for idempotency)
pub fn line_exists(content: &str, pattern: &str) -> bool {
    // Enable multiline mode so ^ matches start of any line
    let multiline_pattern = format!("(?m){}", pattern);
    let re = regex::Regex::new(&multiline_pattern).ok();
    match re {
        Some(r) => r.is_match(content),
        None => content.contains(pattern),
    }
}

/// Apply a recipe to config content (returns new content)
pub fn apply_recipe(content: &str, recipe: &EditorRecipe) -> String {
    let mut result = content.to_string();

    for line in &recipe.lines {
        // Skip if already present
        if line_exists(&result, &line.check_pattern) {
            continue;
        }

        // Add line with comment
        let addition = format!("\n\" {} (added by Anna)\n{}\n", line.comment, line.line);
        result.push_str(&addition);
    }

    result
}

/// Generate a diff-like summary of changes
pub fn describe_changes(recipe: &EditorRecipe, existing_content: &str) -> String {
    let mut changes = Vec::new();

    for line in &recipe.lines {
        if line_exists(existing_content, &line.check_pattern) {
            changes.push(format!("  [skip] {} (already present)", line.comment));
        } else {
            changes.push(format!("  [add]  {}", line.line));
        }
    }

    if changes.iter().all(|c| c.contains("[skip]")) {
        "No changes needed - configuration already present.".to_string()
    } else {
        format!("Changes to {}:\n{}", recipe.editor.config_path(), changes.join("\n"))
    }
}

/// Generate confirmation prompt for user
pub fn confirmation_prompt(recipe: &EditorRecipe) -> String {
    format!(
        "Apply {} configuration for {}?\n\
         File: ~/{}\n\
         To undo: {}\n\
         \n\
         Proceed? [y/N]",
        recipe.feature.display_name(),
        recipe.editor.display_name(),
        recipe.editor.config_path(),
        recipe.rollback_hint
    )
}

/// Create backup of config file
pub fn backup_path(editor: &Editor) -> PathBuf {
    let config = editor.config_file();
    let backup_name = format!(
        "{}.anna-backup-{}",
        config.file_name().unwrap_or_default().to_string_lossy(),
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );
    config.parent().map(|p| p.join(backup_name)).unwrap_or_else(|| config.with_extension("backup"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_from_tool_name() {
        assert_eq!(Editor::from_tool_name("vim"), Some(Editor::Vim));
        assert_eq!(Editor::from_tool_name("nvim"), Some(Editor::Neovim));
        assert_eq!(Editor::from_tool_name("hx"), Some(Editor::Helix));
        assert_eq!(Editor::from_tool_name("unknown"), None);
    }

    #[test]
    fn test_get_recipe_vim_syntax() {
        let recipe = get_recipe(Editor::Vim, ConfigFeature::SyntaxHighlighting);
        assert!(recipe.is_some());
        let r = recipe.unwrap();
        assert!(r.lines[0].line.contains("syntax on"));
    }

    #[test]
    fn test_line_exists_vim() {
        assert!(line_exists("syntax on\nset number", "^\\s*syntax\\s+(on|enable)"));
        assert!(!line_exists("set number", "^\\s*syntax\\s+(on|enable)"));
    }

    #[test]
    fn test_apply_recipe_idempotent() {
        let recipe = get_recipe(Editor::Vim, ConfigFeature::SyntaxHighlighting).unwrap();
        let content = "\" My vimrc\nsyntax on\n";

        // Apply should not duplicate
        let result = apply_recipe(content, &recipe);
        let count = result.matches("syntax on").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_describe_changes() {
        let recipe = get_recipe(Editor::Vim, ConfigFeature::LineNumbers).unwrap();
        let desc = describe_changes(&recipe, "\" empty");
        assert!(desc.contains("[add]"));

        // When all lines already present, returns "No changes needed"
        let desc2 = describe_changes(&recipe, "set number");
        assert!(desc2.contains("No changes needed"));
    }
}
