//! User profile system for personalized Anna experience.
//!
//! v0.0.105: Tracks user preferences, patterns, and interaction history.
//!
//! Storage: ~/.anna/profile.json (per-user)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// User profile with preferences and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Username (from $USER)
    pub username: String,
    /// User's email for notifications (optional)
    pub email: Option<String>,
    /// When the profile was created
    pub created_at: DateTime<Utc>,
    /// Last interaction timestamp
    pub last_seen: DateTime<Utc>,
    /// Consecutive days with interactions
    pub streak_days: u32,
    /// Tool usage patterns (tool_name -> usage_count)
    pub tool_usage: HashMap<String, u32>,
    /// Preferred editor (detected from usage)
    pub preferred_editor: Option<String>,
    /// Preferred shell (detected from usage)
    pub preferred_shell: Option<String>,
    /// User preferences
    pub preferences: UserPreferences,
    /// Topics the user asks about most
    pub topic_interests: HashMap<String, u32>,
    /// Commands user has learned about
    pub learned_commands: Vec<String>,
}

/// User preferences for Anna behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Show learning explanations (why commands work)
    pub learning_mode: bool,
    /// Verbosity level (0=minimal, 1=normal, 2=detailed)
    pub verbosity: u8,
    /// Auto-confirm low-risk changes
    pub auto_confirm_low_risk: bool,
    /// Show internal IT communication (fly on wall)
    pub show_internal_comms: bool,
    /// Personality traits for Anna
    pub personality: PersonalityTraits,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            learning_mode: true,
            verbosity: 1,
            auto_confirm_low_risk: false,
            show_internal_comms: true,
            personality: PersonalityTraits::default(),
        }
    }
}

/// Anna's personality traits (configurable by user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    /// Formality level (0=casual, 1=balanced, 2=formal)
    pub formality: u8,
    /// Humor level (0=none, 1=subtle, 2=playful)
    pub humor: u8,
    /// Technical depth (0=simple, 1=balanced, 2=expert)
    pub technical_depth: u8,
}

impl Default for PersonalityTraits {
    fn default() -> Self {
        Self {
            formality: 1,
            humor: 1,
            technical_depth: 1,
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        let now = Utc::now();

        Self {
            username,
            email: None,
            created_at: now,
            last_seen: now,
            streak_days: 1,
            tool_usage: HashMap::new(),
            preferred_editor: None,
            preferred_shell: None,
            preferences: UserPreferences::default(),
            topic_interests: HashMap::new(),
            learned_commands: Vec::new(),
        }
    }
}

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

        // Update preferred editor if relevant
        let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code"];
        if editors.contains(&tool) {
            self.update_preferred_editor();
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
    }

    /// v0.0.108: Extract and record tools mentioned in a query
    pub fn record_tools_from_query(&mut self, query: &str) {
        let query_lower = query.to_lowercase();

        // Common tools to track
        let tools = [
            // Editors
            "vim", "nvim", "neovim", "nano", "emacs", "helix", "micro", "code", "vscode",
            // Shells
            "bash", "zsh", "fish",
            // Version control
            "git", "github", "gitlab",
            // Package managers
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
}

/// Context for generating personalized greetings
#[derive(Debug, Clone)]
pub struct GreetingContext {
    pub username: String,
    pub days_away: i64,
    pub streak_days: u32,
    pub preferred_editor: Option<String>,
    pub top_topic: Option<String>,
    pub is_new_user: bool,
}

impl GreetingContext {
    /// Generate greeting message based on context
    pub fn generate_greeting(&self) -> String {
        if self.is_new_user {
            return format!(
                "Hello {}! Welcome to Anna. I'm your personal IT department. Ask me anything about your system!",
                self.username
            );
        }

        let time_part = match self.days_away {
            0 => "Good to see you again!".to_string(),
            1 => "Back again today! That's great.".to_string(),
            2..=6 => format!("It's been {} days. I hope everything is running smoothly!", self.days_away),
            _ => format!("It's been a while ({} days)! Let me check if anything happened.", self.days_away),
        };

        let streak_part = if self.streak_days > 1 {
            format!(" You're on a {} day streak!", self.streak_days)
        } else {
            String::new()
        };

        format!("Hello {}! {}{}", self.username, time_part, streak_part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = UserProfile::default();
        assert!(!profile.username.is_empty());
        assert!(profile.preferences.learning_mode);
        assert!(profile.preferences.show_internal_comms);
    }

    #[test]
    fn test_tool_usage_tracking() {
        let mut profile = UserProfile::default();
        profile.record_tool_usage("vim");
        profile.record_tool_usage("vim");
        profile.record_tool_usage("nano");

        assert_eq!(profile.tool_usage.get("vim"), Some(&2));
        assert_eq!(profile.tool_usage.get("nano"), Some(&1));
        assert_eq!(profile.preferred_editor, Some("vim".to_string()));
    }

    #[test]
    fn test_topic_tracking() {
        let mut profile = UserProfile::default();
        profile.record_topic("network");
        profile.record_topic("network");
        profile.record_topic("storage");

        assert_eq!(profile.top_topic(), Some(&"network".to_string()));
    }

    #[test]
    fn test_greeting_new_user() {
        let profile = UserProfile::default();
        let ctx = profile.greeting_context();
        assert!(ctx.is_new_user);
        let greeting = ctx.generate_greeting();
        assert!(greeting.contains("Welcome"));
    }

    #[test]
    fn test_learned_commands() {
        let mut profile = UserProfile::default();
        profile.record_learned_command("free -h");
        profile.record_learned_command("free -h"); // Duplicate
        profile.record_learned_command("df -h");

        assert_eq!(profile.learned_commands.len(), 2);
    }
}
