//! Proactive suggestions after answering questions.
//!
//! Analyzes the question and answer to suggest related improvements or next steps.

use anna_shared::agent::detect_domains;
use tracing::debug;

/// A proactive suggestion for the user.
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Short title
    pub title: String,
    /// Description of what the user could do
    pub description: String,
    /// The question they could ask
    pub followup_question: String,
}

/// Generate proactive suggestions based on the question and answer.
pub fn generate_suggestions(question: &str, answer: &str, commands: &[String]) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    let q_lower = question.to_lowercase();
    let a_lower = answer.to_lowercase();

    // Disk-related suggestions
    if q_lower.contains("disk") || q_lower.contains("storage") || q_lower.contains("space") {
        if a_lower.contains("80%") || a_lower.contains("90%") || a_lower.contains("full") {
            suggestions.push(Suggestion {
                title: "Clean up disk".to_string(),
                description: "Your disk is getting full. I can help clean up package cache and old files.".to_string(),
                followup_question: "clean up disk space".to_string(),
            });
        }
        if !a_lower.contains("ncdu") {
            suggestions.push(Suggestion {
                title: "Find large files".to_string(),
                description: "I can show you which directories are using the most space.".to_string(),
                followup_question: "what is using the most disk space".to_string(),
            });
        }
    }

    // Memory-related suggestions
    if q_lower.contains("memory") || q_lower.contains("ram") || q_lower.contains("swap") {
        if a_lower.contains("high") || a_lower.contains("low free") {
            suggestions.push(Suggestion {
                title: "Find memory hogs".to_string(),
                description: "I can show you which processes are using the most memory.".to_string(),
                followup_question: "what is using the most memory".to_string(),
            });
        }
    }

    // Package/update-related suggestions
    if q_lower.contains("update") || q_lower.contains("upgrade") || q_lower.contains("pacman") {
        if !a_lower.contains("orphan") {
            suggestions.push(Suggestion {
                title: "Check orphans".to_string(),
                description: "You might have orphaned packages that can be removed.".to_string(),
                followup_question: "check for orphaned packages".to_string(),
            });
        }
    }

    // Service-related suggestions
    if q_lower.contains("service") || q_lower.contains("systemd") || q_lower.contains("failed") {
        if a_lower.contains("failed") || a_lower.contains("error") {
            suggestions.push(Suggestion {
                title: "Check logs".to_string(),
                description: "I can show you the logs for the failed service.".to_string(),
                followup_question: "show logs for failed services".to_string(),
            });
        }
    }

    // Network-related suggestions
    if q_lower.contains("wifi") || q_lower.contains("network") || q_lower.contains("internet") {
        if a_lower.contains("not connected") || a_lower.contains("no wifi") {
            suggestions.push(Suggestion {
                title: "Scan networks".to_string(),
                description: "I can scan for available WiFi networks.".to_string(),
                followup_question: "scan for wifi networks".to_string(),
            });
        }
    }

    // Security-related suggestions
    if q_lower.contains("security") || q_lower.contains("firewall") || q_lower.contains("ssh") {
        suggestions.push(Suggestion {
            title: "Security audit".to_string(),
            description: "I can run a basic security check on your system.".to_string(),
            followup_question: "run a security check".to_string(),
        });
    }

    // Performance-related suggestions
    if q_lower.contains("slow") || q_lower.contains("performance") || q_lower.contains("speed") {
        if !a_lower.contains("boot") {
            suggestions.push(Suggestion {
                title: "Check boot time".to_string(),
                description: "I can analyze what's slowing down your boot.".to_string(),
                followup_question: "what is slowing down my boot".to_string(),
            });
        }
        suggestions.push(Suggestion {
            title: "System optimization".to_string(),
            description: "I can suggest optimizations for your system.".to_string(),
            followup_question: "how can I optimize my system".to_string(),
        });
    }

    // Limit to top 2 most relevant suggestions
    suggestions.truncate(2);

    debug!("Generated {} suggestions for question", suggestions.len());
    suggestions
}

/// Format suggestions for display.
pub fn format_suggestions(suggestions: &[Suggestion]) -> Option<String> {
    if suggestions.is_empty() {
        return None;
    }

    let mut lines = vec!["\n---".to_string(), "You might also want to:".to_string()];

    for (i, s) in suggestions.iter().enumerate() {
        lines.push(format!("  {}. {} - \"{}\"", i + 1, s.title, s.followup_question));
    }

    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_suggestions() {
        let suggestions = generate_suggestions(
            "how much disk space do I have",
            "Your disk is 85% full with only 20GB free.",
            &["df -h".to_string()],
        );
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.followup_question.contains("disk")));
    }

    #[test]
    fn test_no_suggestions_for_simple() {
        let suggestions = generate_suggestions(
            "what is my IP address",
            "Your IP is 192.168.1.100",
            &["ip addr".to_string()],
        );
        // Simple factual questions shouldn't trigger suggestions
        assert!(suggestions.is_empty());
    }
}
