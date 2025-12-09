//! Shell recipe types (v0.0.231).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported shells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn display_name(&self) -> &'static str {
        match self {
            Shell::Bash => "Bash",
            Shell::Zsh => "Zsh",
            Shell::Fish => "Fish",
        }
    }

    /// Get config file path
    pub fn config_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(match self {
            Shell::Bash => ".bashrc",
            Shell::Zsh => ".zshrc",
            Shell::Fish => ".config/fish/config.fish",
        })
    }

    /// Detect user's shell from $SHELL
    pub fn detect() -> Option<Self> {
        let shell = std::env::var("SHELL").ok()?;
        if shell.contains("bash") {
            Some(Shell::Bash)
        } else if shell.contains("zsh") {
            Some(Shell::Zsh)
        } else if shell.contains("fish") {
            Some(Shell::Fish)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Shell configuration features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellFeature {
    /// Colored prompt
    ColoredPrompt,
    /// Show git branch in prompt
    GitPrompt,
    /// Syntax highlighting (zsh/fish)
    SyntaxHighlighting,
    /// Auto-suggestions (zsh/fish)
    AutoSuggestions,
    /// Colored ls output
    ColoredLs,
    /// History settings
    HistorySettings,
    /// Custom aliases
    Aliases,
    /// PATH additions
    PathAdditions,
}

impl ShellFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            ShellFeature::ColoredPrompt => "colored prompt",
            ShellFeature::GitPrompt => "git branch in prompt",
            ShellFeature::SyntaxHighlighting => "syntax highlighting",
            ShellFeature::AutoSuggestions => "auto-suggestions",
            ShellFeature::ColoredLs => "colored ls",
            ShellFeature::HistorySettings => "history settings",
            ShellFeature::Aliases => "aliases",
            ShellFeature::PathAdditions => "PATH additions",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            ShellFeature::ColoredPrompt => &["color", "prompt", "ps1"],
            ShellFeature::GitPrompt => &["git", "branch", "prompt"],
            ShellFeature::SyntaxHighlighting => &["syntax", "highlight"],
            ShellFeature::AutoSuggestions => &["auto", "suggest", "complete"],
            ShellFeature::ColoredLs => &["color", "ls", "dir"],
            ShellFeature::HistorySettings => &["history", "histsize"],
            ShellFeature::Aliases => &["alias"],
            ShellFeature::PathAdditions => &["path", "bin"],
        }
    }
}

/// A shell configuration recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellRecipe {
    pub shell: Shell,
    pub feature: ShellFeature,
    pub description: String,
    pub lines: Vec<String>,
    pub rollback_hint: Option<String>,
}

impl ShellRecipe {
    pub fn new(shell: Shell, feature: ShellFeature, desc: &str, lines: Vec<&str>) -> Self {
        Self {
            shell,
            feature,
            description: desc.to_string(),
            lines: lines.into_iter().map(|s| s.to_string()).collect(),
            rollback_hint: None,
        }
    }

    pub fn with_rollback(mut self, hint: &str) -> Self {
        self.rollback_hint = Some(hint.to_string());
        self
    }
}
