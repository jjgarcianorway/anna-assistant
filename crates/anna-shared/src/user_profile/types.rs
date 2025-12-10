//! User profile type definitions (v0.0.292).
//!
//! v0.0.236: Added pattern history for trend detection.
//! v0.0.238: Added session history for "since last time" summaries.
//! v0.0.292: Added ResponsePreferences for preference-aware formatting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::patterns::PatternHistory;
use super::session::{SessionHistory, SessionSummary};

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
    /// v0.0.236: Pattern history for trend detection
    #[serde(default)]
    pub pattern_history: PatternHistory,
    /// v0.0.238: Session history for "since last time" summaries
    #[serde(default)]
    pub session_history: SessionHistory,
    /// v0.0.238: Current session (not persisted until session ends)
    #[serde(skip)]
    pub current_session: Option<SessionSummary>,
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
            // v0.0.265: Default to false - internal comms are confusing for most users
            show_internal_comms: false,
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
            pattern_history: PatternHistory::default(),
            session_history: SessionHistory::default(),
            current_session: None,
        }
    }
}

/// v0.0.292: Response preferences for LLM formatting
/// Extracted from UserPreferences for easy serialization to JSON context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePreferences {
    /// Show learning explanations (why commands work)
    pub learning_mode: bool,
    /// Verbosity level (0=minimal, 1=normal, 2=detailed)
    pub verbosity: u8,
    /// Formality level (0=casual, 1=balanced, 2=formal)
    pub formality: u8,
    /// Humor level (0=none, 1=subtle, 2=playful)
    pub humor: u8,
    /// Technical depth (0=simple, 1=balanced, 2=expert)
    pub technical_depth: u8,
}

impl Default for ResponsePreferences {
    fn default() -> Self {
        Self {
            learning_mode: true,
            verbosity: 1,
            formality: 1,
            humor: 1,
            technical_depth: 1,
        }
    }
}

impl ResponsePreferences {
    /// Load from user profile
    pub fn from_profile(profile: &UserProfile) -> Self {
        Self {
            learning_mode: profile.preferences.learning_mode,
            verbosity: profile.preferences.verbosity,
            formality: profile.preferences.personality.formality,
            humor: profile.preferences.personality.humor,
            technical_depth: profile.preferences.personality.technical_depth,
        }
    }

    /// Load from disk (convenience method)
    pub fn load() -> Self {
        let profile = UserProfile::load();
        Self::from_profile(&profile)
    }

    /// Get verbosity description for LLM context
    pub fn verbosity_desc(&self) -> &'static str {
        match self.verbosity {
            0 => "minimal - be very brief, just essential info",
            2 => "detailed - include full explanations and context",
            _ => "normal - balanced detail level",
        }
    }

    /// Get formality description for LLM context
    pub fn formality_desc(&self) -> &'static str {
        match self.formality {
            0 => "casual - relaxed, conversational tone",
            2 => "formal - professional, precise language",
            _ => "balanced - friendly but professional",
        }
    }

    /// Get humor description for LLM context
    pub fn humor_desc(&self) -> &'static str {
        match self.humor {
            0 => "none - stick to facts only",
            2 => "playful - include light humor when appropriate",
            _ => "subtle - occasional light touch",
        }
    }

    /// Get technical depth description for LLM context
    pub fn technical_depth_desc(&self) -> &'static str {
        match self.technical_depth {
            0 => "simple - avoid jargon, explain basics",
            2 => "expert - assume technical knowledge, use precise terms",
            _ => "balanced - explain concepts but don't over-simplify",
        }
    }
}
