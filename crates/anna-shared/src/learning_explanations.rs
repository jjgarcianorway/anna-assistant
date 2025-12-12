//! Learning mode explanations (v0.0.457).
//!
//! When learning_mode is enabled, Anna explains:
//! - Why commands are being run
//! - How commands work
//! - What the output means
//!
//! v0.0.457: Initial implementation per VISION.md Phase 36.

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
        output.push_str("💡 Learning mode:\n");

        for exp in &self.explanations {
            match exp {
                Explanation::WhyRunning { reason, .. } => {
                    output.push_str(&format!("  Why: {}\n", reason));
                }
                Explanation::HowItWorks {
                    explanation,
                    key_flags,
                    ..
                } => {
                    output.push_str(&format!("  How: {}\n", explanation));
                    for flag in key_flags {
                        output.push_str(&format!("    {} - {}\n", flag.flag, flag.meaning));
                    }
                }
                Explanation::OutputMeaning {
                    output_type,
                    interpretation,
                } => {
                    output.push_str(&format!("  {} means: {}\n", output_type, interpretation));
                }
                Explanation::Citation { source, excerpt } => {
                    output.push_str(&format!("  [{}] {}\n", source, excerpt));
                }
            }
        }

        output
    }
}

/// Known command explanations database
pub struct CommandExplainer;

impl CommandExplainer {
    /// Get explanation for common commands
    pub fn explain(command: &str) -> Option<LearningContext> {
        let cmd_base = command.split_whitespace().next()?;

        let mut ctx = LearningContext::new(command);

        match cmd_base {
            "df" => {
                ctx.add_why("Checking disk space usage on mounted filesystems");
                ctx.add_how(
                    "df (disk free) shows how much space is available on each mounted filesystem",
                    vec![
                        FlagExplanation {
                            flag: "-h".to_string(),
                            meaning: "Human-readable sizes (GB, MB)".to_string(),
                        },
                        FlagExplanation {
                            flag: "-T".to_string(),
                            meaning: "Show filesystem type".to_string(),
                        },
                    ],
                );
                ctx.add_output_meaning(
                    "Use%",
                    "Percentage of disk space used. Above 90% may cause issues",
                );
            }
            "free" => {
                ctx.add_why("Checking memory (RAM) usage");
                ctx.add_how(
                    "free shows total, used, and available memory",
                    vec![
                        FlagExplanation {
                            flag: "-h".to_string(),
                            meaning: "Human-readable sizes".to_string(),
                        },
                        FlagExplanation {
                            flag: "-m".to_string(),
                            meaning: "Show in megabytes".to_string(),
                        },
                    ],
                );
                ctx.add_output_meaning(
                    "available",
                    "Memory that can be used without swapping. More important than 'free'",
                );
            }
            "lsblk" => {
                ctx.add_why("Listing block devices (disks and partitions)");
                ctx.add_how(
                    "lsblk shows storage devices in a tree structure",
                    vec![
                        FlagExplanation {
                            flag: "-f".to_string(),
                            meaning: "Show filesystem info".to_string(),
                        },
                        FlagExplanation {
                            flag: "-o".to_string(),
                            meaning: "Specify output columns".to_string(),
                        },
                    ],
                );
            }
            "systemctl" => {
                if command.contains("status") {
                    ctx.add_why("Checking the status of a systemd service");
                    ctx.add_output_meaning(
                        "Active: active (running)",
                        "Service is running normally",
                    );
                    ctx.add_output_meaning(
                        "Active: failed",
                        "Service crashed or failed to start",
                    );
                } else if command.contains("list-units") {
                    ctx.add_why("Listing all systemd units and their states");
                }
            }
            "journalctl" => {
                ctx.add_why("Reading system logs from the journal");
                ctx.add_how(
                    "journalctl queries the systemd journal for log entries",
                    vec![
                        FlagExplanation {
                            flag: "-u".to_string(),
                            meaning: "Filter by unit/service".to_string(),
                        },
                        FlagExplanation {
                            flag: "-b".to_string(),
                            meaning: "Show logs since boot".to_string(),
                        },
                        FlagExplanation {
                            flag: "-p".to_string(),
                            meaning: "Filter by priority (err, warning)".to_string(),
                        },
                    ],
                );
            }
            "ip" => {
                ctx.add_why("Querying network interface information");
                ctx.add_how(
                    "ip command manages network interfaces, addresses, and routing",
                    vec![
                        FlagExplanation {
                            flag: "addr".to_string(),
                            meaning: "Show IP addresses".to_string(),
                        },
                        FlagExplanation {
                            flag: "link".to_string(),
                            meaning: "Show link-layer info".to_string(),
                        },
                        FlagExplanation {
                            flag: "route".to_string(),
                            meaning: "Show routing table".to_string(),
                        },
                    ],
                );
            }
            "lscpu" => {
                ctx.add_why("Getting CPU information");
                ctx.add_how(
                    "lscpu displays CPU architecture information from /proc/cpuinfo",
                    vec![],
                );
            }
            "uname" => {
                ctx.add_why("Getting system/kernel information");
                ctx.add_how(
                    "uname prints system information",
                    vec![
                        FlagExplanation {
                            flag: "-r".to_string(),
                            meaning: "Kernel release version".to_string(),
                        },
                        FlagExplanation {
                            flag: "-a".to_string(),
                            meaning: "All information".to_string(),
                        },
                    ],
                );
            }
            "cat" => {
                ctx.add_why("Reading file contents");
                ctx.add_how("cat outputs the contents of files", vec![]);
            }
            "sensors" => {
                ctx.add_why("Reading hardware sensor data (temperature, fans, voltage)");
                ctx.add_how(
                    "sensors displays readings from lm-sensors compatible chips",
                    vec![],
                );
            }
            "lspci" => {
                ctx.add_why("Listing PCI devices (graphics, network, sound cards)");
                ctx.add_how(
                    "lspci shows all PCI buses and devices",
                    vec![
                        FlagExplanation {
                            flag: "-v".to_string(),
                            meaning: "Verbose output".to_string(),
                        },
                        FlagExplanation {
                            flag: "-k".to_string(),
                            meaning: "Show kernel drivers".to_string(),
                        },
                    ],
                );
            }
            "pacman" => {
                ctx.add_why("Arch Linux package management");
                ctx.add_how(
                    "pacman is the Arch Linux package manager",
                    vec![
                        FlagExplanation {
                            flag: "-S".to_string(),
                            meaning: "Sync/install packages".to_string(),
                        },
                        FlagExplanation {
                            flag: "-Q".to_string(),
                            meaning: "Query local database".to_string(),
                        },
                        FlagExplanation {
                            flag: "-R".to_string(),
                            meaning: "Remove packages".to_string(),
                        },
                    ],
                );
            }
            _ => return None,
        }

        Some(ctx)
    }
}

/// Check if learning mode should add explanations
pub fn should_explain(learning_mode: bool, command: &str) -> Option<LearningContext> {
    if !learning_mode {
        return None;
    }
    CommandExplainer::explain(command)
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
        assert!(output.contains("Why:"));
        assert!(output.contains("man test"));
    }
}
