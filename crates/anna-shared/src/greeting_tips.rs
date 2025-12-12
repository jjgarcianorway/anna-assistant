//! Greeting Tips System (v0.0.468).
//!
//! Shows helpful tips about Anna's configuration options during greetings.
//! Per VISION.md: "Greetings can show tips about config options - personality
//! traits, learning mode, risk levels, etc."
//!
//! Tips are:
//! - Based on current user settings (suggest what they're not using)
//! - Randomized with variety (different tips each greeting)
//! - Non-intrusive (only show occasionally, not every greeting)

use crate::user_profile::{UserPreferences, UserProfile};

/// A tip about configuration options
#[derive(Debug, Clone)]
pub struct GreetingTip {
    /// Unique identifier for deduplication
    pub id: &'static str,
    /// Category of the tip
    pub category: TipCategory,
    /// The tip message (Anna's voice)
    pub message: String,
    /// Natural language command to change the setting
    pub command_hint: &'static str,
}

/// Tip categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipCategory {
    /// Learning mode tips
    Learning,
    /// Personality configuration
    Personality,
    /// Risk and confirmation settings
    Safety,
    /// Display and verbosity settings
    Display,
    /// Notification settings
    Notifications,
}

/// Get all available tips for current profile state
pub fn get_available_tips(profile: &UserProfile) -> Vec<GreetingTip> {
    let prefs = &profile.preferences;
    let mut tips = Vec::new();

    // Learning mode tips
    if !prefs.learning_mode {
        tips.push(GreetingTip {
            id: "tip-learning-on",
            category: TipCategory::Learning,
            message: "Want to understand why commands work? Enable learning mode \
                     and I'll explain each step."
                .into(),
            command_hint: "enable learning mode",
        });
    } else {
        tips.push(GreetingTip {
            id: "tip-learning-off",
            category: TipCategory::Learning,
            message: "Learning mode is on. If you already know your way around, \
                     you can say \"disable learning\" for quicker answers."
                .into(),
            command_hint: "disable learning",
        });
    }

    // Verbosity tips
    match prefs.verbosity {
        0 => tips.push(GreetingTip {
            id: "tip-verbose-up",
            category: TipCategory::Display,
            message: "I'm keeping answers brief. If you want more detail, \
                     say \"more verbose\" or \"detailed answers\"."
                .into(),
            command_hint: "more verbose",
        }),
        2 => tips.push(GreetingTip {
            id: "tip-verbose-down",
            category: TipCategory::Display,
            message: "I'm giving detailed explanations. If you prefer quicker answers, \
                     try \"shorter answers\" or \"be brief\"."
                .into(),
            command_hint: "shorter answers",
        }),
        _ => {}
    }

    // Auto-confirm tips
    if !prefs.auto_confirm_low_risk {
        tips.push(GreetingTip {
            id: "tip-auto-confirm-on",
            category: TipCategory::Safety,
            message: "I ask before every change. If you trust me with low-risk fixes, \
                     say \"enable auto-confirm\" and I'll apply safe changes automatically."
                .into(),
            command_hint: "enable auto-confirm",
        });
    } else {
        tips.push(GreetingTip {
            id: "tip-auto-confirm-off",
            category: TipCategory::Safety,
            message: "Auto-confirm is on for low-risk changes. If you want to review \
                     everything first, say \"disable auto-confirm\"."
                .into(),
            command_hint: "disable auto-confirm",
        });
    }

    // Internal comms tips
    if !prefs.show_internal_comms {
        tips.push(GreetingTip {
            id: "tip-comms-on",
            category: TipCategory::Display,
            message: "Want to see my team discussing your requests? \
                     Say \"show internal comms\" for a fly-on-the-wall experience."
                .into(),
            command_hint: "show internal comms",
        });
    }

    // Personality: Formality
    add_personality_tips(&mut tips, prefs);

    // Email notification tip
    if profile.email.is_none() {
        tips.push(GreetingTip {
            id: "tip-email",
            category: TipCategory::Notifications,
            message: "For long-running tasks, I can email you when done. \
                     Just tell me your email address when you're ready."
                .into(),
            command_hint: "my email is ...",
        });
    }

    tips
}

/// Add personality-related tips
fn add_personality_tips(tips: &mut Vec<GreetingTip>, prefs: &UserPreferences) {
    // Formality
    match prefs.personality.formality {
        0 => tips.push(GreetingTip {
            id: "tip-formal",
            category: TipCategory::Personality,
            message: "I'm being casual. If you prefer a more professional tone, \
                     say \"be formal\" or \"be professional\"."
                .into(),
            command_hint: "be formal",
        }),
        2 => tips.push(GreetingTip {
            id: "tip-casual",
            category: TipCategory::Personality,
            message: "I'm in professional mode. For a more relaxed vibe, \
                     try \"be casual\" or \"be friendly\"."
                .into(),
            command_hint: "be casual",
        }),
        _ => {}
    }

    // Humor
    match prefs.personality.humor {
        0 => tips.push(GreetingTip {
            id: "tip-humor-on",
            category: TipCategory::Personality,
            message: "I'm keeping things serious. If you'd like some humor, \
                     say \"be funny\" or \"enable humor\"."
                .into(),
            command_hint: "be funny",
        }),
        2 => tips.push(GreetingTip {
            id: "tip-humor-off",
            category: TipCategory::Personality,
            message: "I'm in playful mode. For just-the-facts answers, \
                     say \"no jokes\" or \"serious only\"."
                .into(),
            command_hint: "no jokes",
        }),
        _ => {}
    }

    // Technical depth
    match prefs.personality.technical_depth {
        0 => tips.push(GreetingTip {
            id: "tip-expert",
            category: TipCategory::Personality,
            message: "I'm explaining things simply. If you're experienced, \
                     try \"expert mode\" for more technical depth."
                .into(),
            command_hint: "expert mode",
        }),
        2 => tips.push(GreetingTip {
            id: "tip-simple",
            category: TipCategory::Personality,
            message: "I'm in expert mode. If something's unclear, \
                     say \"explain simply\" or \"beginner mode\"."
                .into(),
            command_hint: "explain simply",
        }),
        _ => {}
    }
}

/// Select a random tip based on seed (typically based on timestamp)
pub fn select_tip(tips: &[GreetingTip], seed: u64) -> Option<&GreetingTip> {
    if tips.is_empty() {
        return None;
    }
    let idx = (seed as usize) % tips.len();
    tips.get(idx)
}

/// Should we show a tip this greeting? (probability-based)
/// Shows tip roughly 1 in 3 greetings
pub fn should_show_tip(seed: u64) -> bool {
    seed % 3 == 0
}

/// Format a tip for display in greeting
pub fn format_tip_for_greeting(tip: &GreetingTip) -> String {
    format!(
        "[tip] {} (say \"{}\")",
        tip.message, tip.command_hint
    )
}

/// Get a single random tip for the current greeting
pub fn get_random_greeting_tip(profile: &UserProfile, seed: u64) -> Option<String> {
    if !should_show_tip(seed) {
        return None;
    }

    let tips = get_available_tips(profile);
    select_tip(&tips, seed).map(format_tip_for_greeting)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_tips() {
        let profile = UserProfile::default();
        let tips = get_available_tips(&profile);
        // Should have at least some tips for default profile
        assert!(!tips.is_empty());
    }

    #[test]
    fn test_learning_mode_tips() {
        let mut profile = UserProfile::default();

        // With learning mode ON (default)
        let tips = get_available_tips(&profile);
        assert!(tips.iter().any(|t| t.id == "tip-learning-off"));

        // With learning mode OFF
        profile.preferences.learning_mode = false;
        let tips = get_available_tips(&profile);
        assert!(tips.iter().any(|t| t.id == "tip-learning-on"));
    }

    #[test]
    fn test_select_tip() {
        let tips = vec![
            GreetingTip {
                id: "a",
                category: TipCategory::Learning,
                message: "A".into(),
                command_hint: "a",
            },
            GreetingTip {
                id: "b",
                category: TipCategory::Learning,
                message: "B".into(),
                command_hint: "b",
            },
        ];

        // Different seeds should (eventually) select different tips
        let t1 = select_tip(&tips, 0);
        let t2 = select_tip(&tips, 1);
        assert!(t1.is_some());
        assert!(t2.is_some());
        assert_ne!(t1.unwrap().id, t2.unwrap().id);
    }

    #[test]
    fn test_should_show_tip() {
        // Should be roughly 1 in 3
        let shows: Vec<bool> = (0..9).map(should_show_tip).collect();
        let count = shows.iter().filter(|&&x| x).count();
        assert_eq!(count, 3); // Exactly 3 out of 9 (0, 3, 6)
    }

    #[test]
    fn test_format_tip() {
        let tip = GreetingTip {
            id: "test",
            category: TipCategory::Learning,
            message: "Test message".into(),
            command_hint: "do thing",
        };
        let formatted = format_tip_for_greeting(&tip);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("do thing"));
    }

    #[test]
    fn test_personality_tips() {
        let mut profile = UserProfile::default();

        // Formal mode
        profile.preferences.personality.formality = 2;
        let tips = get_available_tips(&profile);
        assert!(tips.iter().any(|t| t.id == "tip-casual"));

        // Casual mode
        profile.preferences.personality.formality = 0;
        let tips = get_available_tips(&profile);
        assert!(tips.iter().any(|t| t.id == "tip-formal"));
    }

    #[test]
    fn test_verbosity_tips() {
        let mut profile = UserProfile::default();

        // Minimal verbosity
        profile.preferences.verbosity = 0;
        let tips = get_available_tips(&profile);
        assert!(tips.iter().any(|t| t.id == "tip-verbose-up"));

        // Detailed verbosity
        profile.preferences.verbosity = 2;
        let tips = get_available_tips(&profile);
        assert!(tips.iter().any(|t| t.id == "tip-verbose-down"));
    }
}
