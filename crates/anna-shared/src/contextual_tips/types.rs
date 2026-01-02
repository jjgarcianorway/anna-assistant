//! Types for the contextual tips system.

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
