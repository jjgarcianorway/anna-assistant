//! User profile type definitions (v0.0.217).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
