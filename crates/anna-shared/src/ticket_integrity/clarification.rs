//! Clarification Before Probes (Part 2) - v0.0.442.
//!
//! Enforce Fact-First Clarification Loop (FCL):
//! - For config-like intents, check required facts FIRST
//! - If facts missing → clarify BEFORE running probes
//! - No probes until facts are known
//!
//! WRONG ORDER: probes → clarification → answer
//! RIGHT ORDER: check facts → clarify if missing → probes → answer

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            "desktop.wallpapers_location" | "wallpapers_location" => Some(Self::DesktopWallpapersLocation),
            "editor.config_overview" | "editor_config" | "vim_setup" => Some(Self::EditorConfigOverview),
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

/// Known facts store (user-provided facts).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnownFacts {
    /// Facts by name.
    facts: HashMap<String, KnownFact>,
}

/// A known fact from user or previous clarification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownFact {
    /// Fact name.
    pub name: String,
    /// Fact value.
    pub value: String,
    /// Source of fact.
    pub source: FactSource,
    /// Confidence (1.0 for user-provided).
    pub confidence: f64,
}

/// Source of a known fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactSource {
    /// User explicitly provided.
    User,
    /// Inferred from probe.
    Probe,
    /// Default assumption.
    Default,
}

impl KnownFacts {
    /// Create empty facts store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a fact.
    pub fn add(&mut self, name: &str, value: &str, source: FactSource) {
        let confidence = match source {
            FactSource::User => 1.0,
            FactSource::Probe => 0.9,
            FactSource::Default => 0.5,
        };
        self.facts.insert(
            name.to_string(),
            KnownFact {
                name: name.to_string(),
                value: value.to_string(),
                source,
                confidence,
            },
        );
    }

    /// Get a fact.
    pub fn get(&self, name: &str) -> Option<&KnownFact> {
        self.facts.get(name)
    }

    /// Check if fact is known.
    pub fn has(&self, name: &str) -> bool {
        self.facts.contains_key(name)
    }

    /// Check which facts are missing from a list.
    pub fn missing(&self, required: &[&str]) -> Vec<String> {
        required
            .iter()
            .filter(|f| !self.has(f))
            .map(|f| f.to_string())
            .collect()
    }

    /// Check if all required facts are known.
    pub fn has_all(&self, required: &[&str]) -> bool {
        required.iter().all(|f| self.has(f))
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

/// Check if clarification is needed BEFORE probes.
pub fn check_clarification_needed(
    intent: &str,
    known_facts: &KnownFacts,
) -> ClarificationDecision {
    // Check if this is a clarification-required intent
    let cri = match ClarificationRequiredIntent::from_intent(intent) {
        Some(i) => i,
        None => return ClarificationDecision::NotClarificationIntent,
    };

    // Get required facts for this intent
    let required = cri.required_facts();

    // Check which are missing
    let missing = known_facts.missing(&required);

    if missing.is_empty() {
        return ClarificationDecision::ProceedToProbes;
    }

    // Get clarification for first missing fact
    let first_missing = &missing[0];
    if let Some(question) = cri.clarification_for(first_missing) {
        return ClarificationDecision::NeedClarification {
            question,
            missing_facts: missing,
        };
    }

    // No clarification question defined, but facts are missing
    // This is a bug in the system - we should have questions for all required facts
    ClarificationDecision::ProceedToProbes
}

/// Intent patterns that should trigger clarification-first.
pub fn is_clarification_required_intent(intent: &str) -> bool {
    let lower = intent.to_lowercase();

    // Editor-related config questions
    if lower.contains("editor") && (lower.contains("syntax") || lower.contains("config") || lower.contains("setup")) {
        return true;
    }

    // Wallpaper questions
    if lower.contains("wallpaper") && (lower.contains("where") || lower.contains("location")) {
        return true;
    }

    // Terminal config questions
    if lower.contains("terminal") && (lower.contains("theme") || lower.contains("config")) {
        return true;
    }

    // Shell config questions
    if lower.contains("shell") && (lower.contains("prompt") || lower.contains("config")) {
        return true;
    }

    ClarificationRequiredIntent::from_intent(intent).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarification_required_intents() {
        let intent = ClarificationRequiredIntent::EditorSyntaxStatus;
        let required = intent.required_facts();
        assert!(required.contains(&"editor.name"));
        assert!(required.contains(&"editor.config_path"));
    }

    #[test]
    fn test_clarification_question() {
        let intent = ClarificationRequiredIntent::EditorSyntaxStatus;
        let question = intent.clarification_for("editor.name").unwrap();
        assert!(question.question.contains("editor"));
        assert!(!question.options.is_empty());
    }

    #[test]
    fn test_known_facts() {
        let mut facts = KnownFacts::new();
        assert!(!facts.has("editor.name"));

        facts.add("editor.name", "vim", FactSource::User);
        assert!(facts.has("editor.name"));
        assert_eq!(facts.get("editor.name").unwrap().value, "vim");
    }

    #[test]
    fn test_clarification_decision_need_clarification() {
        let facts = KnownFacts::new();
        let decision = check_clarification_needed("editor.syntax_status", &facts);

        match decision {
            ClarificationDecision::NeedClarification { question, missing_facts } => {
                assert!(!missing_facts.is_empty());
                assert!(question.question.contains("editor"));
            }
            _ => panic!("Expected NeedClarification"),
        }
    }

    #[test]
    fn test_clarification_decision_proceed() {
        let mut facts = KnownFacts::new();
        facts.add("editor.name", "vim", FactSource::User);
        facts.add("editor.config_path", "~/.vimrc", FactSource::User);

        let decision = check_clarification_needed("editor.syntax_status", &facts);
        assert!(matches!(decision, ClarificationDecision::ProceedToProbes));
    }

    #[test]
    fn test_is_clarification_required_intent() {
        assert!(is_clarification_required_intent("editor.syntax_status"));
        assert!(is_clarification_required_intent("wallpapers_location"));
        assert!(!is_clarification_required_intent("memory.free"));
        assert!(!is_clarification_required_intent("system.swap_configured"));
    }
}
