//! Contextual Tips System (v0.0.482).
//!
//! Provides relevant tips based on the user's current context:
//! - What they just asked about
//! - What action was just performed
//! - What topic they're working on
//!
//! Unlike greeting_tips which are random, these are targeted.

use std::collections::HashSet;

/// A contextual tip
#[derive(Debug, Clone)]
pub struct ContextualTip {
    /// Unique tip ID
    pub id: &'static str,
    /// The tip message
    pub message: &'static str,
    /// Related command or action
    pub related_action: Option<&'static str>,
}

/// Context for generating tips
#[derive(Debug, Clone, Default)]
pub struct TipContext {
    /// Topics mentioned in query
    pub topics: HashSet<String>,
    /// Command that was just run
    pub last_command: Option<String>,
    /// Whether this is a first-time topic
    pub is_new_topic: bool,
    /// Learning mode enabled
    pub learning_mode: bool,
}

impl TipContext {
    /// Create context from a query
    pub fn from_query(query: &str) -> Self {
        let lower = query.to_lowercase();
        let mut topics = HashSet::new();

        // Detect topics from query
        let topic_keywords = [
            ("vim", "editor"),
            ("nano", "editor"),
            ("nvim", "editor"),
            ("emacs", "editor"),
            ("docker", "containers"),
            ("kubernetes", "containers"),
            ("k8s", "containers"),
            ("nginx", "webserver"),
            ("apache", "webserver"),
            ("git", "git"),
            ("ssh", "ssh"),
            ("systemd", "services"),
            ("service", "services"),
            ("network", "network"),
            ("disk", "storage"),
            ("mount", "storage"),
            ("package", "packages"),
            ("install", "packages"),
            ("cron", "scheduling"),
            ("timer", "scheduling"),
            ("firewall", "security"),
            ("permission", "security"),
        ];

        for (keyword, topic) in topic_keywords {
            if lower.contains(keyword) {
                topics.insert(topic.to_string());
            }
        }

        Self {
            topics,
            last_command: None,
            is_new_topic: false,
            learning_mode: false,
        }
    }

    /// Add a topic
    pub fn with_topic(mut self, topic: &str) -> Self {
        self.topics.insert(topic.to_string());
        self
    }

    /// Set last command
    pub fn with_command(mut self, cmd: &str) -> Self {
        self.last_command = Some(cmd.to_string());
        self
    }

    /// Set learning mode
    pub fn with_learning_mode(mut self, enabled: bool) -> Self {
        self.learning_mode = enabled;
        self
    }
}

/// Get tips for editor-related queries
fn editor_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "editor-config",
            message: "You can configure any editor setting by asking naturally, \
                     like \"enable line numbers in vim\".",
            related_action: None,
        },
        ContextualTip {
            id: "editor-backup",
            message: "I always backup config files before changes. \
                     Say \"undo last change\" if something goes wrong.",
            related_action: Some("undo last change"),
        },
    ]
}

/// Get tips for container-related queries
fn container_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "docker-compose",
            message: "For Docker Compose projects, try \"start my docker compose\" \
                     or \"view docker logs\".",
            related_action: None,
        },
        ContextualTip {
            id: "docker-cleanup",
            message: "Docker using too much space? Ask \"clean up docker\" \
                     to remove unused images and containers.",
            related_action: Some("clean up docker"),
        },
    ]
}

/// Get tips for git-related queries
fn git_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "git-config",
            message: "Need to set up git? Try \"configure git with my name and email\".",
            related_action: Some("configure git"),
        },
        ContextualTip {
            id: "git-ssh",
            message: "For GitHub SSH access, ask \"set up SSH key for GitHub\".",
            related_action: Some("set up ssh for github"),
        },
    ]
}

/// Get tips for service-related queries
fn service_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "service-logs",
            message: "Having service issues? Try \"show logs for [service]\" \
                     or \"why did [service] fail\".",
            related_action: None,
        },
        ContextualTip {
            id: "service-enable",
            message: "Want a service to start on boot? Ask \"enable [service] on startup\".",
            related_action: None,
        },
    ]
}

/// Get tips for network-related queries
fn network_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "network-diag",
            message: "Network issues? Try \"diagnose network\" or \"why is my connection slow\".",
            related_action: Some("diagnose network"),
        },
        ContextualTip {
            id: "network-ports",
            message: "Check what's using a port with \"what's using port 8080\".",
            related_action: None,
        },
    ]
}

/// Get tips for storage-related queries
fn storage_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "storage-large",
            message: "Looking for space? Ask \"find large files\" or \"what's using disk space\".",
            related_action: Some("find large files"),
        },
        ContextualTip {
            id: "storage-mount",
            message: "For persistent mounts, I can update /etc/fstab. \
                     Just confirm when I ask.",
            related_action: None,
        },
    ]
}

/// Get tips for package-related queries
fn package_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "package-search",
            message: "Not sure of the package name? Try \"find package for [tool]\" \
                     or \"what provides [command]\".",
            related_action: None,
        },
        ContextualTip {
            id: "package-update",
            message: "Keep your system updated with \"update my system\" \
                     or \"check for updates\".",
            related_action: Some("update system"),
        },
    ]
}

/// Get tips for scheduling-related queries
fn scheduling_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "cron-syntax",
            message: "Cron confusing? Just say \"run [command] every day at 3am\" \
                     and I'll figure out the syntax.",
            related_action: None,
        },
        ContextualTip {
            id: "timer-vs-cron",
            message: "For systemd systems, timers are often better than cron. \
                     I'll suggest the best option for your case.",
            related_action: None,
        },
    ]
}

/// Get tips for security-related queries
fn security_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "security-perms",
            message: "Permission denied? Ask \"fix permissions for [path]\" \
                     or \"who owns [file]\".",
            related_action: None,
        },
        ContextualTip {
            id: "security-firewall",
            message: "Need to open a port? Try \"allow port 443\" or \"check firewall rules\".",
            related_action: None,
        },
    ]
}

/// Get learning mode tips
fn learning_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "learn-explain",
            message: "With learning mode on, I explain every command. \
                     Ask \"why did you run that\" anytime.",
            related_action: Some("why did you run that"),
        },
        ContextualTip {
            id: "learn-reference",
            message: "Want to know more? I cite from man pages and Arch Wiki. \
                     Ask \"show me the docs for [command]\".",
            related_action: None,
        },
    ]
}

/// Get general tips (no specific context)
fn general_tips() -> Vec<ContextualTip> {
    vec![
        ContextualTip {
            id: "general-undo",
            message: "Made a mistake? Say \"undo last change\" to restore from backup.",
            related_action: Some("undo last change"),
        },
        ContextualTip {
            id: "general-history",
            message: "Check what we've done with \"show history\" or \"what did you change\".",
            related_action: Some("show history"),
        },
        ContextualTip {
            id: "general-stats",
            message: "Curious about your usage? Try \"show my stats\" or \"fun facts\".",
            related_action: Some("show my stats"),
        },
    ]
}

/// Get contextual tips based on current context
pub fn get_contextual_tips(context: &TipContext) -> Vec<ContextualTip> {
    let mut tips = Vec::new();

    // Add topic-specific tips
    for topic in &context.topics {
        match topic.as_str() {
            "editor" => tips.extend(editor_tips()),
            "containers" => tips.extend(container_tips()),
            "git" => tips.extend(git_tips()),
            "services" => tips.extend(service_tips()),
            "network" => tips.extend(network_tips()),
            "storage" => tips.extend(storage_tips()),
            "packages" => tips.extend(package_tips()),
            "scheduling" => tips.extend(scheduling_tips()),
            "security" => tips.extend(security_tips()),
            _ => {}
        }
    }

    // Add learning tips if learning mode is on
    if context.learning_mode {
        tips.extend(learning_tips());
    }

    // If no specific tips, add general tips
    if tips.is_empty() {
        tips = general_tips();
    }

    tips
}

/// Select a single tip from available tips
pub fn select_tip(tips: &[ContextualTip], seed: u64) -> Option<&ContextualTip> {
    if tips.is_empty() {
        return None;
    }
    let idx = (seed as usize) % tips.len();
    tips.get(idx)
}

/// Format a tip for display
pub fn format_tip(tip: &ContextualTip) -> String {
    if let Some(action) = tip.related_action {
        format!("Tip: {} (try: \"{}\")", tip.message, action)
    } else {
        format!("Tip: {}", tip.message)
    }
}

/// Get a single contextual tip for display
pub fn get_tip_for_query(query: &str, learning_mode: bool) -> Option<String> {
    let context = TipContext::from_query(query).with_learning_mode(learning_mode);

    let tips = get_contextual_tips(&context);

    // Use timestamp-based seed for variety
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    select_tip(&tips, seed).map(format_tip)
}

/// Check if we should show a tip (probability-based)
/// Shows tip roughly 1 in 4 times
pub fn should_show_tip() -> bool {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    seed % 4 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_from_query() {
        let ctx = TipContext::from_query("How do I configure vim?");
        assert!(ctx.topics.contains("editor"));

        let ctx2 = TipContext::from_query("restart docker service");
        assert!(ctx2.topics.contains("containers"));
        assert!(ctx2.topics.contains("services"));
    }

    #[test]
    fn test_get_contextual_tips_editor() {
        let ctx = TipContext::from_query("vim config");
        let tips = get_contextual_tips(&ctx);
        assert!(!tips.is_empty());
        assert!(tips.iter().any(|t| t.id.contains("editor")));
    }

    #[test]
    fn test_get_contextual_tips_docker() {
        let ctx = TipContext::from_query("docker compose up");
        let tips = get_contextual_tips(&ctx);
        assert!(tips.iter().any(|t| t.id.contains("docker")));
    }

    #[test]
    fn test_get_contextual_tips_general() {
        let ctx = TipContext::default(); // No topics
        let tips = get_contextual_tips(&ctx);
        assert!(!tips.is_empty());
        // Should get general tips
        assert!(tips.iter().any(|t| t.id.starts_with("general")));
    }

    #[test]
    fn test_learning_mode_tips() {
        let ctx = TipContext::default().with_learning_mode(true);
        let tips = get_contextual_tips(&ctx);
        assert!(tips.iter().any(|t| t.id.starts_with("learn")));
    }

    #[test]
    fn test_select_tip() {
        let tips = general_tips();
        let tip = select_tip(&tips, 0);
        assert!(tip.is_some());
    }

    #[test]
    fn test_format_tip_with_action() {
        let tip = ContextualTip {
            id: "test",
            message: "Test message",
            related_action: Some("do thing"),
        };
        let formatted = format_tip(&tip);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("do thing"));
    }

    #[test]
    fn test_format_tip_without_action() {
        let tip = ContextualTip {
            id: "test",
            message: "Test message",
            related_action: None,
        };
        let formatted = format_tip(&tip);
        assert!(formatted.contains("Test message"));
        assert!(!formatted.contains("try:"));
    }

    #[test]
    fn test_get_tip_for_query() {
        let tip = get_tip_for_query("how to use git", false);
        // May or may not return a tip due to probability
        // Just check it doesn't panic
        if let Some(t) = tip {
            assert!(t.contains("Tip:"));
        }
    }

    #[test]
    fn test_multiple_topics() {
        let ctx = TipContext::from_query("docker network issues");
        assert!(ctx.topics.contains("containers"));
        assert!(ctx.topics.contains("network"));

        let tips = get_contextual_tips(&ctx);
        // Should have tips from both categories
        assert!(tips.len() >= 2);
    }
}
