//! Greeting context for LLM-based greeting generation (v0.0.314).
//!
//! Provides structured context for the translator LLM to generate
//! personalized, varied greetings while maintaining consistent content.
//! v0.0.281: Added telemetry-based health issue population.
//! v0.0.287: Added maintenance prompts for proactive suggestions.
//! v0.0.289: Added interesting facts about system, patterns, and Anna's growth.
//! v0.0.292: Added personality traits for tone-aware greetings.
//! v0.0.314: Added time-of-day awareness for appropriate greetings.

use crate::context_memory::ContextMemory;
use crate::health_alerts::{generate_alerts, AlertSeverity};
use crate::interesting_facts::InterestingFacts;
use crate::learning_stats::LearningStats;
use crate::maintenance_actions::generate_maintenance_actions;
use crate::snapshot::SystemSnapshot;
use crate::system_telemetry::TelemetryStore;
use crate::user_profile::{ResponsePreferences, UserProfile};
use serde::{Deserialize, Serialize};

/// Context for generating a personalized greeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingContext {
    /// Username
    pub username: String,
    /// Hours since last session (None if first time)
    pub hours_since_last: Option<u64>,
    /// Days since last session (None if first time or < 1 day)
    pub days_since_last: Option<u64>,
    /// Is this the user's first session ever?
    pub is_first_time: bool,
    /// Current streak in days (0 if no streak)
    pub streak_days: u32,
    /// Preferred editor if known (vim, nvim, nano, etc.)
    pub preferred_editor: Option<String>,
    /// Top topic of interest if any
    pub top_topic: Option<String>,
    /// Open ticket count
    pub open_tickets: u32,
    /// Summary of last session activity (if any)
    pub last_session_summary: Option<String>,
    /// System health issues (if any)
    pub health_issues: Vec<String>,
    /// LLM status (ready, starting, error)
    pub llm_status: String,
    /// v0.0.287: Proactive maintenance prompts
    #[serde(default)]
    pub maintenance_prompts: Vec<String>,
    /// v0.0.289: Interesting facts about system/patterns/growth
    #[serde(default)]
    pub interesting_facts: Vec<String>,
    /// v0.0.292: Personality settings for tone
    #[serde(default)]
    pub personality: PersonalityContext,
    /// v0.0.314: Time of day for appropriate greeting ("morning", "afternoon", "evening", "night")
    #[serde(default)]
    pub time_of_day: String,
    /// v0.0.314: Local hour (0-23) for time-aware messages
    #[serde(default)]
    pub local_hour: u8,
    /// v0.0.401: Learning progress hint (e.g., "I've learned 5 patterns")
    #[serde(default)]
    pub learning_hint: Option<String>,
    /// v0.0.401: Context memory hint (e.g., "You've been asking about disk usage a lot")
    #[serde(default)]
    pub memory_hint: Option<String>,
}

/// v0.0.292: Personality context for LLM greeting generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalityContext {
    /// Formality description (e.g., "casual", "balanced", "formal")
    pub formality: String,
    /// Humor description (e.g., "none", "subtle", "playful")
    pub humor: String,
    /// Technical depth description
    pub technical_depth: String,
}

impl Default for GreetingContext {
    fn default() -> Self {
        // v0.0.314: Detect time of day from local time
        let (time_of_day, local_hour) = detect_time_of_day();

        Self {
            username: "user".to_string(),
            hours_since_last: None,
            days_since_last: None,
            is_first_time: true,
            streak_days: 0,
            preferred_editor: None,
            top_topic: None,
            open_tickets: 0,
            last_session_summary: None,
            health_issues: Vec::new(),
            llm_status: "ready".to_string(),
            maintenance_prompts: Vec::new(),
            interesting_facts: Vec::new(),
            personality: PersonalityContext::default(),
            time_of_day,
            local_hour,
            learning_hint: None,
            memory_hint: None,
        }
    }
}

/// v0.0.314: Detect time of day from system clock
fn detect_time_of_day() -> (String, u8) {
    use chrono::{Local, Timelike};

    let now = Local::now();
    let hour = now.hour() as u8;

    let period = match hour {
        5..=11 => "morning",
        12..=16 => "afternoon",
        17..=20 => "evening",
        _ => "night",
    };

    (period.to_string(), hour)
}

impl GreetingContext {
    /// Populate health issues from telemetry store
    pub fn with_telemetry(mut self, store: &TelemetryStore) -> Self {
        let alerts = generate_alerts(store);

        // Add critical and warning alerts to health issues
        for alert in alerts.iter().filter(|a| !a.dismissed) {
            let prefix = match alert.severity {
                AlertSeverity::Critical => "[!]",
                AlertSeverity::Warning => "[*]",
                AlertSeverity::Info => continue, // Skip info in greetings
            };
            self.health_issues
                .push(format!("{} {}", prefix, alert.message));
        }

        // Add health score insight if low
        let score = store.health_score();
        if score < 70 {
            self.health_issues
                .push(format!("System health score: {}% (below optimal)", score));
        }

        self
    }

    /// Get a summary of health status for LLM context
    pub fn health_summary(&self) -> Option<String> {
        if self.health_issues.is_empty() {
            return None;
        }

        let critical = self
            .health_issues
            .iter()
            .filter(|s| s.starts_with("[!]"))
            .count();
        let warnings = self
            .health_issues
            .iter()
            .filter(|s| s.starts_with("[*]"))
            .count();

        if critical > 0 || warnings > 0 {
            Some(format!(
                "{} critical, {} warnings detected",
                critical, warnings
            ))
        } else {
            Some(format!("{} system notices", self.health_issues.len()))
        }
    }

    /// v0.0.287: Add maintenance prompts from snapshot and telemetry
    pub fn with_maintenance(
        mut self,
        snapshot: &SystemSnapshot,
        telemetry: Option<&TelemetryStore>,
    ) -> Self {
        let actions = generate_maintenance_actions(snapshot, telemetry);

        // Add only critical/urgent actions (urgency <= 2) as prompts
        for action in actions.iter().filter(|a| a.urgency <= 2).take(2) {
            self.maintenance_prompts
                .push(format!("{}: \"{}\"", action.title, action.anna_query));
        }

        self
    }

    /// Check if there are actionable maintenance items
    pub fn has_maintenance(&self) -> bool {
        !self.maintenance_prompts.is_empty()
    }

    /// v0.0.289: Add interesting facts from system state
    pub fn with_interesting_facts(mut self, snapshot: &SystemSnapshot) -> Self {
        let facts = InterestingFacts::from_current_state(snapshot);
        // Get top 3 most interesting facts for the greeting
        self.interesting_facts = facts.as_strings(3);
        self
    }

    /// Check if there are interesting facts to share
    pub fn has_interesting_facts(&self) -> bool {
        !self.interesting_facts.is_empty()
    }

    /// v0.0.292: Add personality context from user profile
    pub fn with_personality(mut self, profile: &UserProfile) -> Self {
        let prefs = ResponsePreferences::from_profile(profile);
        self.personality = PersonalityContext {
            formality: prefs.formality_desc().to_string(),
            humor: prefs.humor_desc().to_string(),
            technical_depth: prefs.technical_depth_desc().to_string(),
        };
        self
    }

    /// v0.0.292: Check if personality has been configured
    pub fn has_personality(&self) -> bool {
        !self.personality.formality.is_empty()
    }

    /// v0.0.401: Add learning progress hint from stats
    pub fn with_learning_stats(mut self) -> Self {
        let stats = LearningStats::collect();
        self.learning_hint = stats.brief_hint();
        self
    }

    /// v0.0.401: Add context memory hints
    pub fn with_context_memory(mut self) -> Self {
        let memory = ContextMemory::load();
        self.memory_hint = memory.continuity_greeting();
        // Also extract top topic if we have frequent patterns
        if self.top_topic.is_none() {
            if let Some(topic) = memory.frequent_topics().first() {
                self.top_topic = Some(topic.topic.clone());
            }
        }
        // Extract preferred editor if known
        if self.preferred_editor.is_none() {
            self.preferred_editor = memory.preferred_editor.clone();
        }
        self
    }

    /// v0.0.401: Check if learning hints are available
    pub fn has_learning(&self) -> bool {
        self.learning_hint.is_some()
    }

    /// v0.0.401: Check if memory hints are available
    pub fn has_memory(&self) -> bool {
        self.memory_hint.is_some()
    }
}

/// Response from greeting generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingResponse {
    /// The generated greeting text (multi-line)
    pub greeting: String,
    /// Whether this was LLM-generated or fallback
    pub is_llm_generated: bool,
}

impl GreetingResponse {
    /// Create a fallback greeting when LLM is unavailable
    pub fn fallback(ctx: &GreetingContext) -> Self {
        let mut lines = Vec::new();

        // v0.0.314: Time-aware greeting prefix
        let time_greeting = match ctx.time_of_day.as_str() {
            "morning" => "Good morning",
            "afternoon" => "Good afternoon",
            "evening" => "Good evening",
            "night" => "Hi", // Late night, keep it simple
            _ => "Hello",
        };

        // Basic greeting
        if ctx.is_first_time {
            lines.push(format!("{}, {}!", time_greeting, ctx.username));
            lines.push(String::new());
            lines.push("Welcome! I'm Anna, your local IT department.".to_string());
            lines.push("Just ask me anything about your system.".to_string());
        } else if let Some(days) = ctx.days_since_last {
            if days >= 1 {
                lines.push(format!("{}, {}.", time_greeting, ctx.username));
                lines.push(String::new());
                let word = if days == 1 { "day" } else { "days" };
                lines.push(format!("It's been {} {} since we last spoke.", days, word));
            } else {
                lines.push(format!(
                    "{}, {}! Welcome back.",
                    time_greeting, ctx.username
                ));
            }
        } else if let Some(hours) = ctx.hours_since_last {
            if hours > 12 {
                lines.push(format!("{}, {}.", time_greeting, ctx.username));
                lines.push(format!("It's been about {} hours.", hours));
            } else if hours > 1 {
                lines.push(format!(
                    "{}, {}! Welcome back.",
                    time_greeting, ctx.username
                ));
            } else {
                lines.push(format!("Hello again, {}!", ctx.username));
            }
        } else {
            lines.push(format!(
                "{}, {}! Welcome back.",
                time_greeting, ctx.username
            ));
        }

        // Open tickets
        if ctx.open_tickets > 0 {
            lines.push(String::new());
            let word = if ctx.open_tickets == 1 {
                "ticket"
            } else {
                "tickets"
            };
            lines.push(format!("You have {} open {}.", ctx.open_tickets, word));
        }

        // Health issues
        if !ctx.health_issues.is_empty() {
            lines.push(String::new());
            lines.push("System notices:".to_string());
            for issue in ctx.health_issues.iter().take(3) {
                lines.push(format!("  - {}", issue));
            }
        }

        // v0.0.287: Maintenance prompts
        if !ctx.maintenance_prompts.is_empty() {
            lines.push(String::new());
            lines.push("Quick actions you might want:".to_string());
            for prompt in ctx.maintenance_prompts.iter().take(2) {
                lines.push(format!("  - {}", prompt));
            }
        }

        // v0.0.289: Interesting facts
        if !ctx.interesting_facts.is_empty() {
            lines.push(String::new());
            // Pick one random fact to keep greeting concise
            let fact = &ctx.interesting_facts[0];
            lines.push(format!("By the way: {}", fact));
        }

        // v0.0.401: Learning progress hint
        if let Some(ref hint) = ctx.learning_hint {
            lines.push(String::new());
            lines.push(hint.clone());
        }

        // v0.0.401: Context memory hint
        if let Some(ref hint) = ctx.memory_hint {
            lines.push(String::new());
            lines.push(hint.clone());
        }

        // Closing
        lines.push(String::new());
        lines.push("What can I help you with?".to_string());

        Self {
            greeting: lines.join("\n"),
            is_llm_generated: false,
        }
    }
}
