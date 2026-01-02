//! Clarification Types - v0.0.442.
//!
//! Type definitions for the clarification system.

use serde::{Deserialize, Serialize};

/// Intents that require clarification BEFORE probes.
/// These are "config-like" intents where we can't know what to probe
/// without user input.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClarificationRequiredIntent {
    /// "Is syntax highlighting enabled in my editor?"
    EditorSyntaxStatus,
    /// "Enable syntax highlighting in my editor"
    EditorSyntaxEnable,
    /// "Where are my wallpapers?"
    DesktopWallpapersLocation,
    /// "What is my vim setup?" / editor config overview
    EditorConfigOverview,
    /// "What theme is my terminal using?"
    TerminalTheme,
    /// "Change my shell prompt"
    ShellPromptChange,
}

impl ClarificationRequiredIntent {
    /// Get required facts for this intent.
    pub fn required_facts(&self) -> Vec<&'static str> {
        match self {
            Self::EditorSyntaxStatus => vec!["editor.name", "editor.config_path"],
            Self::EditorSyntaxEnable => vec!["editor.name", "editor.config_path"],
            Self::DesktopWallpapersLocation => vec!["desktop.wallpaper_dir"],
            Self::EditorConfigOverview => vec!["editor.name", "editor.config_path"],
            Self::TerminalTheme => vec!["terminal.name", "terminal.config_path"],
            Self::ShellPromptChange => vec!["shell.name", "shell.config_path"],
        }
    }

    /// Get clarification question for missing fact.
    pub fn clarification_for(&self, missing_fact: &str) -> Option<ClarificationQuestion> {
        match (self, missing_fact) {
            (Self::EditorSyntaxStatus | Self::EditorSyntaxEnable | Self::EditorConfigOverview, "editor.name") => {
                Some(ClarificationQuestion {
                    question: "Which editor do you mean?".to_string(),
                    options: vec![
                        ClarificationOption::new("vim", "vim"),
                        ClarificationOption::new("neovim", "neovim"),
                        ClarificationOption::new("nano", "nano"),
                        ClarificationOption::new("emacs", "emacs"),
                        ClarificationOption::new("something else", "__other__"),
                        ClarificationOption::new("cancel", "__cancel__"),
                    ],
                    fact_to_set: "editor.name".to_string(),
                })
            }
            (Self::DesktopWallpapersLocation, "desktop.wallpaper_dir") => {
                Some(ClarificationQuestion {
                    question: "Where do you keep your wallpaper files?\n(Example: ~/Pictures/Wallpapers or ~/.config/hypr/hyprpaper.conf)".to_string(),
                    options: vec![
                        ClarificationOption::new("~/Pictures/Wallpapers", "~/Pictures/Wallpapers"),
                        ClarificationOption::new("~/.config/hypr/", "~/.config/hypr/"),
                        ClarificationOption::new("something else (type it)", "__other__"),
                        ClarificationOption::new("cancel", "__cancel__"),
                    ],
                    fact_to_set: "desktop.wallpaper_dir".to_string(),
                })
            }
            (Self::EditorConfigOverview, "editor.config_path") if self == &Self::EditorConfigOverview => {
                Some(ClarificationQuestion {
                    question: "Do you mean your user config or system-wide config?".to_string(),
                    options: vec![
                        ClarificationOption::new("user config (~/.vimrc)", "~/.vimrc"),
                        ClarificationOption::new("system-wide (/etc/vim)", "/etc/vim"),
                        ClarificationOption::new("cancel", "__cancel__"),
                    ],
                    fact_to_set: "editor.config_path".to_string(),
                })
            }
            (Self::TerminalTheme, "terminal.name") => {
                Some(ClarificationQuestion {
                    question: "Which terminal emulator?".to_string(),
                    options: vec![
                        ClarificationOption::new("kitty", "kitty"),
                        ClarificationOption::new("alacritty", "alacritty"),
                        ClarificationOption::new("wezterm", "wezterm"),
                        ClarificationOption::new("gnome-terminal", "gnome-terminal"),
                        ClarificationOption::new("something else", "__other__"),
                        ClarificationOption::new("cancel", "__cancel__"),
                    ],
                    fact_to_set: "terminal.name".to_string(),
                })
            }
            (Self::ShellPromptChange, "shell.name") => {
                Some(ClarificationQuestion {
                    question: "Which shell?".to_string(),
                    options: vec![
                        ClarificationOption::new("bash", "bash"),
                        ClarificationOption::new("zsh", "zsh"),
                        ClarificationOption::new("fish", "fish"),
                        ClarificationOption::new("cancel", "__cancel__"),
                    ],
                    fact_to_set: "shell.name".to_string(),
                })
            }
            _ => None,
        }
    }

    /// Parse from intent string.
    pub fn from_intent(intent: &str) -> Option<Self> {
        match intent.to_lowercase().as_str() {
            "editor.syntax_status" | "editor_syntax_status" => Some(Self::EditorSyntaxStatus),
            "editor.syntax_enable" | "editor_syntax_enable" => Some(Self::EditorSyntaxEnable),
            "desktop.wallpapers_location" | "wallpapers_location" => {
                Some(Self::DesktopWallpapersLocation)
            }
            "editor.config_overview" | "editor_config" | "vim_setup" => {
                Some(Self::EditorConfigOverview)
            }
            "terminal.theme" | "terminal_theme" => Some(Self::TerminalTheme),
            "shell.prompt_change" | "shell_prompt" => Some(Self::ShellPromptChange),
            _ => None,
        }
    }
}

/// A clarification question to ask the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    /// The question text.
    pub question: String,
    /// Available options.
    pub options: Vec<ClarificationOption>,
    /// Which fact this clarification sets.
    pub fact_to_set: String,
}

/// An option in a clarification question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationOption {
    /// Display label.
    pub label: String,
    /// Value to store if selected.
    pub value: String,
}

impl ClarificationOption {
    /// Create new option.
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
        }
    }

    /// Is this a cancel option?
    pub fn is_cancel(&self) -> bool {
        self.value == "__cancel__"
    }

    /// Is this an "other" option?
    pub fn is_other(&self) -> bool {
        self.value == "__other__"
    }
}

impl ClarificationQuestion {
    /// Format for display (clean, no duplicates).
    pub fn display(&self) -> String {
        let mut out = self.question.clone();
        out.push('\n');
        for (i, opt) in self.options.iter().enumerate() {
            if opt.is_cancel() {
                out.push_str(&format!("  0) {}\n", opt.label));
            } else if opt.is_other() {
                out.push_str(&format!("  9) {}\n", opt.label));
            } else {
                out.push_str(&format!("  {}) {}\n", i + 1, opt.label));
            }
        }
        out
    }
}

/// Clarification loop decision.
#[derive(Debug, Clone)]
pub enum ClarificationDecision {
    /// All facts known, proceed to probes.
    ProceedToProbes,
    /// Need clarification before probes.
    NeedClarification {
        question: ClarificationQuestion,
        missing_facts: Vec<String>,
    },
    /// Unknown intent, no clarification needed.
    NotClarificationIntent,
}
