//! Configuration change handler for REPL (v0.0.237).
//!
//! Handles natural language config requests like:
//! - "Anna, enable learning mode"
//! - "make Anna more casual"
//! - "hide internal communications"

use anna_shared::config_parser::{parse_config_request, ConfigChange};
use anna_shared::ui::colors;
use anna_shared::user_profile::UserProfile;

/// Result of attempting to handle a config request
pub enum ConfigResult {
    /// Successfully handled as config change
    Handled,
    /// Not a config request, continue normal processing
    NotConfig,
}

/// Try to handle input as a config change request
/// Returns Handled if it was a config command, NotConfig otherwise
pub fn try_handle_config(input: &str) -> ConfigResult {
    // Try to parse as config request
    if let Some(change) = parse_config_request(input) {
        apply_config_change(&change);
        ConfigResult::Handled
    } else {
        ConfigResult::NotConfig
    }
}

/// Apply a config change to the user's profile
fn apply_config_change(change: &ConfigChange) {
    // Load current profile
    let mut profile = UserProfile::load();

    // Apply the change
    change.apply(&mut profile.preferences);

    // Save profile
    if let Err(e) = profile.save() {
        println!(
            "{}Warning: Could not save preferences: {}{}",
            colors::WARN,
            e,
            colors::RESET
        );
        return;
    }

    // Show confirmation
    println!();
    println!(
        "{}[config]{} {}",
        colors::CYAN,
        colors::RESET,
        change.description()
    );

    // Show tip for related features
    show_config_tip(change);
}

/// Show helpful tip related to the config change
fn show_config_tip(change: &ConfigChange) {
    let tip = match change {
        ConfigChange::LearningMode(true) => {
            Some("I'll explain why commands work and what they do.")
        }
        ConfigChange::LearningMode(false) => {
            Some("To re-enable: \"Anna, enable learning mode\"")
        }
        ConfigChange::ShowInternalComms(true) => {
            Some("You'll see the IT department chatter during requests.")
        }
        ConfigChange::ShowInternalComms(false) => {
            Some("To see the team again: \"show internal comms\"")
        }
        ConfigChange::Formality(0) => Some("Hey! I'll be more chill now."),
        ConfigChange::Formality(2) => Some("Understood. I shall maintain a professional tone."),
        ConfigChange::Humor(2) => Some("This is going to be fun! :)"),
        ConfigChange::Humor(0) => Some("All business, got it."),
        ConfigChange::AutoConfirmLowRisk(true) => {
            Some("I'll apply safe changes automatically. High-risk still needs approval.")
        }
        ConfigChange::Verbosity(2) => Some("I'll provide comprehensive explanations."),
        ConfigChange::Verbosity(0) => Some("Short and sweet from now on."),
        _ => None,
    };

    if let Some(t) = tip {
        println!("  {}{}{}", colors::DIM, t, colors::RESET);
    }
}

/// Show current config status
pub fn show_config_status() {
    let profile = UserProfile::load();
    let prefs = &profile.preferences;
    let pers = &prefs.personality;

    println!();
    println!("{}Current Settings:{}", colors::HEADER, colors::RESET);
    println!();

    // Learning & verbosity
    println!(
        "  Learning mode:    {}",
        if prefs.learning_mode {
            format!("{}enabled{}", colors::OK, colors::RESET)
        } else {
            format!("{}disabled{}", colors::DIM, colors::RESET)
        }
    );
    println!(
        "  Verbosity:        {}",
        match prefs.verbosity {
            0 => "minimal",
            1 => "normal",
            _ => "detailed",
        }
    );

    // Automation
    println!(
        "  Auto-confirm:     {}",
        if prefs.auto_confirm_low_risk {
            format!("{}low-risk{}", colors::OK, colors::RESET)
        } else {
            format!("{}ask always{}", colors::DIM, colors::RESET)
        }
    );
    println!(
        "  Internal comms:   {}",
        if prefs.show_internal_comms {
            format!("{}visible{}", colors::OK, colors::RESET)
        } else {
            format!("{}hidden{}", colors::DIM, colors::RESET)
        }
    );

    // Personality
    println!();
    println!("{}Personality:{}", colors::HEADER, colors::RESET);
    println!(
        "  Formality:        {}",
        match pers.formality {
            0 => "casual",
            1 => "balanced",
            _ => "formal",
        }
    );
    println!(
        "  Humor:            {}",
        match pers.humor {
            0 => "none",
            1 => "subtle",
            _ => "playful",
        }
    );
    println!(
        "  Technical depth:  {}",
        match pers.technical_depth {
            0 => "simple",
            1 => "balanced",
            _ => "expert",
        }
    );

    println!();
    println!(
        "{}Change settings with natural language, e.g.:{}",
        colors::DIM,
        colors::RESET
    );
    println!(
        "{}  \"make Anna more casual\" or \"enable learning mode\"{}",
        colors::DIM,
        colors::RESET
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_handle_config_learning() {
        // This would need a mock profile, so just test the parsing
        let change = parse_config_request("enable learning mode");
        assert!(change.is_some());
        assert!(matches!(change.unwrap(), ConfigChange::LearningMode(true)));
    }

    #[test]
    fn test_not_config() {
        let change = parse_config_request("how much disk space");
        assert!(change.is_none());
    }
}
