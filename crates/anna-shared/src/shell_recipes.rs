//! Shell configuration recipes for .bashrc, .zshrc, etc.
//!
//! v0.0.100: Learned from specialists, reusable by translator.
//!
//! Key principle: Recipes are LEARNED from specialist responses and
//! can be applied to SIMILAR queries by the translator without LLM.

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

/// Get built-in shell recipes
pub fn builtin_recipes() -> Vec<ShellRecipe> {
    vec![
        // Bash - colored prompt
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::ColoredPrompt,
            "Add colored prompt to bash",
            vec![
                "# Colored prompt",
                r#"PS1='\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[0m\]$ '"#,
            ],
        ).with_rollback("Remove the PS1= line from .bashrc"),

        // Bash - git prompt
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::GitPrompt,
            "Show git branch in bash prompt",
            vec![
                "# Git branch in prompt",
                r#"parse_git_branch() { git branch 2>/dev/null | grep '^*' | cut -d' ' -f2 | sed 's/.*/ (&)/'; }"#,
                r#"PS1='\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[33m\]$(parse_git_branch)\[\e[0m\]$ '"#,
            ],
        ).with_rollback("Remove parse_git_branch function and PS1 line"),

        // Bash - colored ls
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::ColoredLs,
            "Enable colored ls output",
            vec![
                "# Colored ls",
                "alias ls='ls --color=auto'",
                "alias ll='ls -la --color=auto'",
            ],
        ).with_rollback("Remove the ls aliases from .bashrc"),

        // Bash - history settings
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::HistorySettings,
            "Improve bash history",
            vec![
                "# History settings",
                "HISTSIZE=10000",
                "HISTFILESIZE=20000",
                "HISTCONTROL=ignoreboth:erasedups",
                "shopt -s histappend",
            ],
        ).with_rollback("Remove HIST* lines from .bashrc"),

        // Zsh - syntax highlighting
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::SyntaxHighlighting,
            "Enable zsh syntax highlighting",
            vec![
                "# Syntax highlighting (requires zsh-syntax-highlighting package)",
                "source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh 2>/dev/null",
            ],
        ).with_rollback("Remove the source line for syntax highlighting"),

        // Zsh - auto-suggestions
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::AutoSuggestions,
            "Enable zsh auto-suggestions",
            vec![
                "# Auto-suggestions (requires zsh-autosuggestions package)",
                "source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh 2>/dev/null",
            ],
        ).with_rollback("Remove the source line for auto-suggestions"),

        // Zsh - colored prompt
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::ColoredPrompt,
            "Add colored prompt to zsh",
            vec![
                "# Colored prompt",
                "autoload -U colors && colors",
                r#"PROMPT='%{$fg[green]%}%n@%m%{$reset_color%}:%{$fg[blue]%}%~%{$reset_color%}$ '"#,
            ],
        ).with_rollback("Remove PROMPT= line from .zshrc"),

        // Zsh - git prompt
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::GitPrompt,
            "Show git branch in zsh prompt",
            vec![
                "# Git branch in prompt",
                "autoload -Uz vcs_info",
                "precmd() { vcs_info }",
                "zstyle ':vcs_info:git:*' formats ' (%b)'",
                "setopt PROMPT_SUBST",
                r#"PROMPT='%{$fg[green]%}%n@%m%{$reset_color%}:%{$fg[blue]%}%~%{$fg[yellow]%}${vcs_info_msg_0_}%{$reset_color%}$ '"#,
            ],
        ).with_rollback("Remove vcs_info and PROMPT lines"),

        // Fish - syntax highlighting (built-in, but can configure)
        ShellRecipe::new(
            Shell::Fish,
            ShellFeature::SyntaxHighlighting,
            "Configure fish syntax highlighting colors",
            vec![
                "# Syntax highlighting colors",
                "set fish_color_command green",
                "set fish_color_param cyan",
                "set fish_color_error red",
            ],
        ).with_rollback("Remove set fish_color_* lines"),

        // Common aliases (all shells)
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::Aliases,
            "Common useful aliases",
            vec![
                "# Common aliases",
                "alias ..='cd ..'",
                "alias ...='cd ../..'",
                "alias grep='grep --color=auto'",
                "alias df='df -h'",
                "alias du='du -h'",
            ],
        ).with_rollback("Remove the alias lines from .bashrc"),
    ]
}

/// Find a recipe for a shell and feature
pub fn find_recipe(shell: Shell, feature: ShellFeature) -> Option<ShellRecipe> {
    builtin_recipes()
        .into_iter()
        .find(|r| r.shell == shell && r.feature == feature)
}

/// Find recipes matching keywords
pub fn find_recipes_by_keywords(keywords: &[&str], shell: Option<Shell>) -> Vec<ShellRecipe> {
    builtin_recipes()
        .into_iter()
        .filter(|r| {
            // Filter by shell if specified
            if let Some(s) = shell {
                if r.shell != s {
                    return false;
                }
            }

            // Check if any keyword matches feature keywords
            let feature_keywords = r.feature.keywords();
            keywords.iter().any(|kw| {
                let kw_lower = kw.to_lowercase();
                feature_keywords.iter().any(|fk| fk.contains(&kw_lower) || kw_lower.contains(fk))
            })
        })
        .collect()
}

/// Detect feature from query
pub fn detect_feature(query: &str) -> Option<ShellFeature> {
    let q = query.to_lowercase();

    if q.contains("git") && q.contains("prompt") {
        return Some(ShellFeature::GitPrompt);
    }
    if q.contains("color") && (q.contains("prompt") || q.contains("ps1")) {
        return Some(ShellFeature::ColoredPrompt);
    }
    if q.contains("syntax") || q.contains("highlight") {
        return Some(ShellFeature::SyntaxHighlighting);
    }
    if q.contains("suggest") || q.contains("auto") && q.contains("complete") {
        return Some(ShellFeature::AutoSuggestions);
    }
    if q.contains("color") && q.contains("ls") {
        return Some(ShellFeature::ColoredLs);
    }
    if q.contains("history") {
        return Some(ShellFeature::HistorySettings);
    }
    if q.contains("alias") {
        return Some(ShellFeature::Aliases);
    }
    if q.contains("path") {
        return Some(ShellFeature::PathAdditions);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_config_path() {
        // Just test that paths end with expected filenames
        assert!(Shell::Bash.config_path().to_string_lossy().ends_with(".bashrc"));
        assert!(Shell::Zsh.config_path().to_string_lossy().ends_with(".zshrc"));
    }

    #[test]
    fn test_find_recipe() {
        let recipe = find_recipe(Shell::Bash, ShellFeature::ColoredPrompt);
        assert!(recipe.is_some());
        assert!(recipe.unwrap().lines.iter().any(|l| l.contains("PS1")));
    }

    #[test]
    fn test_detect_feature() {
        assert_eq!(detect_feature("show git branch in prompt"), Some(ShellFeature::GitPrompt));
        assert_eq!(detect_feature("enable syntax highlighting"), Some(ShellFeature::SyntaxHighlighting));
        assert_eq!(detect_feature("colored ls"), Some(ShellFeature::ColoredLs));
    }

    #[test]
    fn test_builtin_recipes_count() {
        let recipes = builtin_recipes();
        assert!(recipes.len() >= 5);
    }
}
