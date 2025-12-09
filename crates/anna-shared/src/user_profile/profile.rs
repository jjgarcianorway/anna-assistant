//! UserProfile implementation (v0.0.238).
//!
//! v0.0.236: Pattern history tracking for trend detection.
//! v0.0.238: Session history for "since last time" summaries.

use chrono::Utc;
use std::fs;
use std::path::PathBuf;

use super::greeting::GreetingContext;
use super::patterns::{EditorTrendInsight, TopicTrendInsight};
use super::session::SessionSummary;
use super::types::UserProfile;

impl UserProfile {
    /// Get profile path
    pub fn profile_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".anna").join("profile.json")
    }

    /// Load profile from disk or create default
    pub fn load() -> Self {
        let path = Self::profile_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(profile) => return profile,
                    Err(_) => {} // Fall through to default
                },
                Err(_) => {} // Fall through to default
            }
        }
        Self::default()
    }

    /// Save profile to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::profile_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// Update last seen and check streak
    pub fn record_interaction(&mut self) {
        let now = Utc::now();
        let last_date = self.last_seen.date_naive();
        let today = now.date_naive();

        let days_diff = (today - last_date).num_days();

        if days_diff == 1 {
            // Consecutive day
            self.streak_days += 1;
        } else if days_diff > 1 {
            // Streak broken
            self.streak_days = 1;
        }
        // Same day = no change

        self.last_seen = now;
    }

    /// v0.0.106: Record a session start (alias for record_interaction)
    pub fn record_session(&mut self) {
        self.record_interaction();
    }

    /// Record tool usage (e.g., "vim", "nano", "htop")
    pub fn record_tool_usage(&mut self, tool: &str) {
        *self.tool_usage.entry(tool.to_string()).or_insert(0) += 1;

        // v0.0.236: Track in pattern history for trends
        self.pattern_history.record_tool(tool);

        // Update preferred editor if relevant
        let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code"];
        if editors.contains(&tool) {
            let old_editor = self.preferred_editor.clone();
            self.update_preferred_editor();
            // Track editor changes for trend detection
            if old_editor != self.preferred_editor {
                if let Some(ref new) = self.preferred_editor {
                    self.pattern_history
                        .record_editor_change(old_editor.as_deref(), new);
                }
            }
        }

        // Update preferred shell if relevant
        let shells = ["bash", "zsh", "fish"];
        if shells.contains(&tool) {
            self.update_preferred_shell();
        }
    }

    fn update_preferred_editor(&mut self) {
        let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code"];
        let best = self
            .tool_usage
            .iter()
            .filter(|(k, _)| editors.contains(&k.as_str()))
            .max_by_key(|(_, v)| *v);

        if let Some((editor, _)) = best {
            self.preferred_editor = Some(editor.clone());
        }
    }

    fn update_preferred_shell(&mut self) {
        let shells = ["bash", "zsh", "fish"];
        let best = self
            .tool_usage
            .iter()
            .filter(|(k, _)| shells.contains(&k.as_str()))
            .max_by_key(|(_, v)| *v);

        if let Some((shell, _)) = best {
            self.preferred_shell = Some(shell.clone());
        }
    }

    /// Record topic interest (e.g., "network", "storage")
    pub fn record_topic(&mut self, topic: &str) {
        *self.topic_interests.entry(topic.to_string()).or_insert(0) += 1;
        // v0.0.236: Track in pattern history for trends
        self.pattern_history.record_topic(topic);
    }

    /// v0.0.236: Get editor trend insight (if any significant trend detected)
    pub fn editor_trend(&self) -> Option<EditorTrendInsight> {
        self.pattern_history
            .editor_trend_insight(self.preferred_editor.as_deref(), &self.tool_usage)
    }

    /// v0.0.236: Get topic trend insight (if any significant trend detected)
    pub fn topic_trend(&self) -> Option<TopicTrendInsight> {
        self.pattern_history.topic_trend_insight(&self.topic_interests)
    }

    /// v0.0.236: Cleanup old pattern data
    pub fn cleanup_patterns(&mut self) {
        self.pattern_history.cleanup_old_data();
    }

    /// v0.0.108: Extract and record tools mentioned in a query
    pub fn record_tools_from_query(&mut self, query: &str) {
        let query_lower = query.to_lowercase();

        // Common tools to track
        let tools = [
            // Editors
            "vim", "nvim", "neovim", "nano", "emacs", "helix", "micro", "code", "vscode",
            // Shells
            "bash", "zsh", "fish", // Version control
            "git", "github", "gitlab", // Package managers
            "pacman", "apt", "dnf", "yum", "brew", "npm", "cargo", "pip",
            // System tools
            "systemctl", "journalctl", "htop", "top", "docker", "podman",
            // Network tools
            "ssh", "curl", "wget", "ping", "traceroute", "netstat", "ss",
            // File tools
            "rsync", "tar", "zip", "grep", "find", "awk", "sed",
        ];

        for tool in tools {
            if query_lower.contains(tool) {
                self.record_tool_usage(tool);
            }
        }
    }

    /// Get most interested topic
    pub fn top_topic(&self) -> Option<&String> {
        self.topic_interests
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k)
    }

    /// Record a learned command
    pub fn record_learned_command(&mut self, cmd: &str) {
        if !self.learned_commands.contains(&cmd.to_string()) {
            self.learned_commands.push(cmd.to_string());
        }
    }

    /// Get days since last interaction
    pub fn days_since_last(&self) -> i64 {
        let now = Utc::now();
        (now.date_naive() - self.last_seen.date_naive()).num_days()
    }

    /// Get a greeting context based on profile
    pub fn greeting_context(&self) -> GreetingContext {
        let days_away = self.days_since_last();

        GreetingContext {
            username: self.username.clone(),
            days_away,
            streak_days: self.streak_days,
            preferred_editor: self.preferred_editor.clone(),
            top_topic: self.top_topic().cloned(),
            is_new_user: self.tool_usage.is_empty() && self.topic_interests.is_empty(),
        }
    }

    /// Set email for notifications
    pub fn set_email(&mut self, email: &str) {
        self.email = Some(email.to_string());
    }

    // ==================== v0.0.238: Session Management ====================

    /// Start a new session
    pub fn start_session(&mut self) {
        self.current_session = Some(SessionSummary::new());
    }

    /// End the current session and add to history
    pub fn end_session(&mut self) {
        if let Some(session) = self.current_session.take() {
            self.session_history.add_session(session);
        }
    }

    /// Record a query in the current session
    pub fn record_session_query(&mut self, topic: Option<&str>) {
        if let Some(ref mut session) = self.current_session {
            session.record_query(topic);
        }
    }

    /// Record a command learned in the current session
    pub fn record_session_command(&mut self, command: &str) {
        if let Some(ref mut session) = self.current_session {
            session.record_command_learned(command);
        }
    }

    /// Record a recipe executed in the current session
    pub fn record_session_recipe(&mut self, recipe_id: &str) {
        if let Some(ref mut session) = self.current_session {
            session.record_recipe(recipe_id);
        }
    }

    /// Get "since last time" summary message
    pub fn since_last_time(&self) -> Option<String> {
        self.session_history.since_last_time()
    }
}
