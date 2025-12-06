//! Personalized greetings and context-aware dialogue (v0.0.89).
//!
//! Provides time-of-day awareness, user personalization, and domain-specific
//! dialogue for a more natural Anna experience.

use crate::teams::Team;

/// Time of day for greeting personalization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Morning,   // 5-12
    Afternoon, // 12-17
    Evening,   // 17-21
    Night,     // 21-5
}

impl TimeOfDay {
    /// Get current time of day from system clock
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(43200); // noon fallback

        // Simple hour calculation (UTC-based, adjust for local would need chrono)
        let hour = ((secs / 3600) % 24) as u8;
        Self::from_hour(hour)
    }

    /// Get time of day from hour (0-23)
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            5..=11 => TimeOfDay::Morning,
            12..=16 => TimeOfDay::Afternoon,
            17..=20 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }

    /// Get appropriate greeting prefix
    pub fn greeting_prefix(&self) -> &'static str {
        match self {
            TimeOfDay::Morning => "Good morning",
            TimeOfDay::Afternoon => "Good afternoon",
            TimeOfDay::Evening => "Good evening",
            TimeOfDay::Night => "Hello",
        }
    }
}

/// Get current username from environment
pub fn current_user() -> Option<String> {
    std::env::var("USER").ok().or_else(|| std::env::var("USERNAME").ok())
}

/// Personalized Anna greeting for REPL/session start
pub fn anna_session_greeting(seed: u64) -> String {
    let time = TimeOfDay::now();
    let user = current_user();

    let base = time.greeting_prefix();

    let greetings = if let Some(name) = user {
        vec![
            format!("{}, {}! How can I help you today?", base, name),
            format!("{}, {}. What can I do for you?", base, name),
            format!("{}, {}! Anna Service Desk at your service.", base, name),
            format!("{}! Welcome back, {}.", base, name),
        ]
    } else {
        vec![
            format!("{}! How can I help you today?", base),
            format!("{}. What can I do for you?", base),
            format!("{}! Anna Service Desk at your service.", base),
            format!("{}! Welcome to Anna.", base),
        ]
    };

    greetings[(seed as usize) % greetings.len()].clone()
}

/// Anna's late night/weekend comment (adds personality)
pub fn anna_off_hours_comment(seed: u64) -> Option<String> {
    let time = TimeOfDay::now();

    if time == TimeOfDay::Night {
        let comments = [
            "Burning the midnight oil, I see!",
            "Late night debugging session?",
            "Working late - I'm here if you need me.",
        ];
        Some(comments[(seed as usize) % comments.len()].to_string())
    } else {
        None
    }
}

/// Anna's comment when user returns after long absence
pub fn anna_welcome_back(hours_away: u64, seed: u64) -> String {
    let phrases = if hours_away > 168 {
        // More than a week
        vec![
            "It's been a while! Good to see you again.",
            "Welcome back! Missed you around here.",
            "Long time no see! Hope all is well.",
        ]
    } else if hours_away > 24 {
        // More than a day
        vec![
            "Welcome back!",
            "Good to see you again.",
            "Back for more? Let's get started.",
        ]
    } else {
        vec![
            "Welcome back.",
            "Ready when you are.",
            "What can I help with?",
        ]
    };

    phrases[(seed as usize) % phrases.len()].to_string()
}

/// Anna's contextual phrases when finishing a query
pub fn anna_followup_prompt(domain: &str, seed: u64) -> String {
    let phrases = match domain.to_lowercase().as_str() {
        "storage" | "disk" => vec![
            "Need me to check anything else about your storage?",
            "Anything else disk-related I can help with?",
            "Let me know if you need more storage details.",
        ],
        "memory" | "ram" | "performance" | "cpu" => vec![
            "Want me to dig deeper into the performance data?",
            "I can run more diagnostics if you'd like.",
            "Need any other performance insights?",
        ],
        "network" | "wifi" => vec![
            "Any other network questions?",
            "I can check more connectivity details if needed.",
            "Let me know if you need network troubleshooting.",
        ],
        "service" | "services" => vec![
            "Should I check any other services?",
            "I can restart or troubleshoot services if you need.",
            "Anything else service-related?",
        ],
        "hardware" | "audio" => vec![
            "Want me to check other hardware components?",
            "I can run more hardware diagnostics.",
            "Any other device questions?",
        ],
        _ => vec![
            "Anything else I can help with?",
            "Let me know if you need anything else.",
            "I'm here if you have more questions.",
        ],
    };

    phrases[(seed as usize) % phrases.len()].to_string()
}

/// Anna's reassurance when probe takes time
pub fn anna_patience_phrase(team: Team, seed: u64) -> String {
    let phrases = match team {
        Team::Storage => vec![
            "Scanning the filesystems...",
            "Reading disk data, one moment...",
            "Checking all mount points...",
        ],
        Team::Performance => vec![
            "Gathering performance metrics...",
            "Analyzing system load...",
            "Collecting CPU and memory data...",
        ],
        Team::Network => vec![
            "Testing network interfaces...",
            "Checking connectivity...",
            "Scanning network configuration...",
        ],
        Team::Services => vec![
            "Querying systemd...",
            "Checking service states...",
            "Reviewing unit statuses...",
        ],
        Team::Logs => vec![
            "Searching through logs...",
            "Analyzing journal entries...",
            "Scanning recent events...",
        ],
        _ => vec![
            "Working on it...",
            "Just a moment...",
            "Gathering the data...",
        ],
    };

    phrases[(seed as usize) % phrases.len()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_of_day() {
        assert_eq!(TimeOfDay::from_hour(8), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(19), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(23), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(3), TimeOfDay::Night);
    }

    #[test]
    fn test_greeting_prefix() {
        assert_eq!(TimeOfDay::Morning.greeting_prefix(), "Good morning");
        assert_eq!(TimeOfDay::Afternoon.greeting_prefix(), "Good afternoon");
        assert_eq!(TimeOfDay::Evening.greeting_prefix(), "Good evening");
        assert_eq!(TimeOfDay::Night.greeting_prefix(), "Hello");
    }

    #[test]
    fn test_session_greeting() {
        let greeting = anna_session_greeting(0);
        assert!(greeting.contains("Good") || greeting.contains("Hello"));
    }

    #[test]
    fn test_welcome_back_varies_by_time() {
        let short = anna_welcome_back(12, 0);
        let medium = anna_welcome_back(48, 0);
        let long = anna_welcome_back(200, 0);

        assert!(short.len() > 5);
        assert!(medium.len() > 5);
        assert!(long.contains("while") || long.contains("back") || long.contains("Long"));
    }

    #[test]
    fn test_followup_prompt() {
        let storage = anna_followup_prompt("storage", 0);
        assert!(storage.contains("storage") || storage.contains("disk"));

        let general = anna_followup_prompt("unknown", 0);
        assert!(general.contains("else") || general.contains("help"));
    }

    #[test]
    fn test_patience_phrase() {
        let storage = anna_patience_phrase(Team::Storage, 0);
        assert!(storage.contains("..."));

        let network = anna_patience_phrase(Team::Network, 0);
        assert!(network.contains("..."));
    }
}
