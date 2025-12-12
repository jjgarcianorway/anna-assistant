//! Learning mode explanations.
//!
//! When learning_mode is enabled, Anna explains:
//! - Why commands are being run
//! - How commands work
//! - What the output means
//!
//! v0.0.457: Initial implementation per VISION.md Phase 36.
//! v0.0.477: Split into modules, added 15+ new command explanations.

mod commands;

pub use commands::CommandExplainer;

use serde::{Deserialize, Serialize};

/// Types of explanations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Explanation {
    /// Why we're running this command
    WhyRunning {
        command: String,
        reason: String,
    },
    /// How this command works
    HowItWorks {
        command: String,
        explanation: String,
        key_flags: Vec<FlagExplanation>,
    },
    /// What the output means
    OutputMeaning {
        output_type: String,
        interpretation: String,
    },
    /// Citation reference
    Citation {
        source: String,
        excerpt: String,
    },
}

/// Explanation for a command flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagExplanation {
    pub flag: String,
    pub meaning: String,
}

/// Learning context for a command
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningContext {
    /// The command being explained
    pub command: String,
    /// Explanations generated
    pub explanations: Vec<Explanation>,
}

impl LearningContext {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            explanations: vec![],
        }
    }

    /// Add why we're running this
    pub fn add_why(&mut self, reason: &str) {
        self.explanations.push(Explanation::WhyRunning {
            command: self.command.clone(),
            reason: reason.to_string(),
        });
    }

    /// Add how it works
    pub fn add_how(&mut self, explanation: &str, flags: Vec<FlagExplanation>) {
        self.explanations.push(Explanation::HowItWorks {
            command: self.command.clone(),
            explanation: explanation.to_string(),
            key_flags: flags,
        });
    }

    /// Add output interpretation
    pub fn add_output_meaning(&mut self, output_type: &str, interpretation: &str) {
        self.explanations.push(Explanation::OutputMeaning {
            output_type: output_type.to_string(),
            interpretation: interpretation.to_string(),
        });
    }

    /// Add citation
    pub fn add_citation(&mut self, source: &str, excerpt: &str) {
        self.explanations.push(Explanation::Citation {
            source: source.to_string(),
            excerpt: excerpt.to_string(),
        });
    }

    /// Format for display (learning mode output)
    pub fn format_display(&self) -> String {
        if self.explanations.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        output.push_str("[learning]\n");

        for exp in &self.explanations {
            match exp {
                Explanation::WhyRunning { reason, .. } => {
                    output.push_str(&format!("  why   {}\n", reason));
                }
                Explanation::HowItWorks {
                    explanation,
                    key_flags,
                    ..
                } => {
                    output.push_str(&format!("  how   {}\n", explanation));
                    for flag in key_flags {
                        output.push_str(&format!("        {} - {}\n", flag.flag, flag.meaning));
                    }
                }
                Explanation::OutputMeaning {
                    output_type,
                    interpretation,
                } => {
                    output.push_str(&format!("  {}  {}\n", output_type, interpretation));
                }
                Explanation::Citation { source, excerpt } => {
                    output.push_str(&format!("  [{}] {}\n", source, excerpt));
                }
            }
        }

        output
    }

    /// Get count of explanations
    pub fn count(&self) -> usize {
        self.explanations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.explanations.is_empty()
    }
}

/// Check if learning mode should add explanations
pub fn should_explain(learning_mode: bool, command: &str) -> Option<LearningContext> {
    if !learning_mode {
        return None;
    }
    CommandExplainer::explain(command)
}

/// Get explanation without checking learning mode
pub fn get_explanation(command: &str) -> Option<LearningContext> {
    CommandExplainer::explain(command)
}

/// List all commands that have explanations
pub fn list_explained_commands() -> Vec<&'static str> {
    vec![
        "df", "free", "lsblk", "systemctl", "journalctl", "ip", "lscpu",
        "uname", "cat", "sensors", "lspci", "pacman", "ps", "top", "htop",
        "grep", "find", "chmod", "chown", "du", "mount", "ss", "netstat",
        "ping", "curl", "wget", "tar", "git", "docker",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_df() {
        let ctx = CommandExplainer::explain("df -h").unwrap();
        assert!(!ctx.explanations.is_empty());
        let display = ctx.format_display();
        assert!(display.contains("disk space"));
    }

    #[test]
    fn test_explain_free() {
        let ctx = CommandExplainer::explain("free -h").unwrap();
        let display = ctx.format_display();
        assert!(display.contains("memory"));
    }

    #[test]
    fn test_explain_systemctl() {
        let ctx = CommandExplainer::explain("systemctl status nginx").unwrap();
        let display = ctx.format_display();
        assert!(display.contains("service"));
    }

    #[test]
    fn test_explain_new_commands() {
        assert!(CommandExplainer::explain("grep pattern").is_some());
        assert!(CommandExplainer::explain("find /").is_some());
        assert!(CommandExplainer::explain("docker ps").is_some());
        assert!(CommandExplainer::explain("git status").is_some());
    }

    #[test]
    fn test_unknown_command() {
        let ctx = CommandExplainer::explain("unknowncommand");
        assert!(ctx.is_none());
    }

    #[test]
    fn test_should_explain() {
        assert!(should_explain(true, "df -h").is_some());
        assert!(should_explain(false, "df -h").is_none());
    }

    #[test]
    fn test_learning_context_format() {
        let mut ctx = LearningContext::new("test");
        ctx.add_why("Testing explanation system");
        ctx.add_citation("man test", "test - check file types");

        let output = ctx.format_display();
        assert!(output.contains("why"));
        assert!(output.contains("man test"));
    }

    #[test]
    fn test_list_explained_commands() {
        let commands = list_explained_commands();
        assert!(commands.contains(&"df"));
        assert!(commands.contains(&"docker"));
        assert!(commands.len() >= 25);
    }
}
