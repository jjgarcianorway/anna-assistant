//! Editor configuration recipes.
//!
//! v0.0.95: Refactored - recipe data moved to editor_recipe_data.rs.
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

// Re-export get_recipe from data module
pub use crate::editor_recipe_data::get_recipe;

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
            "vim" | "vi" => Some(Editor::Vim),
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

// Tests in tests/editor_recipes_tests.rs
