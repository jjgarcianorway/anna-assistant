//! Configuration change handler for REPL (v0.0.337).
//!
//! Handles natural language config requests like:
//! - "Anna, enable learning mode"
//! - "make Anna more casual"
//! - "hide internal communications"
//! - "my email is user@example.com"
//!
//! v0.0.239: Added natural language email setup.
//! v0.0.337: Use centralized UI printing for consistent output.

use anna_shared::config_parser::{parse_config_request, ConfigChange};
use anna_shared::email::EmailConfig;
use anna_shared::ui::{colors, kv, kv_colored, print_hint, print_label, print_section_header};
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
    // v0.0.239: Handle email changes separately
    if change.is_email_change() {
        apply_email_change(change);
        return;
    }

    // Load current profile
    let mut profile = UserProfile::load();

    // Apply the change
    change.apply(&mut profile.preferences);

    // Save profile
    if let Err(e) = profile.save() {
        print_label(
            "warn",
            &format!("Could not save preferences: {}", e),
            colors::WARN,
        );
        return;
    }

    // Show confirmation
    println!();
    print_label("config", &change.description(), colors::HEADER);

    // Show tip for related features
    show_config_tip(change);
}

/// v0.0.239: Apply email-related config change
fn apply_email_change(change: &ConfigChange) {
    let mut config = EmailConfig::load();

    match change {
        ConfigChange::Email(addr) => {
            config.set_email(addr);
            // Also save to user profile for consistency
            let mut profile = UserProfile::load();
            profile.set_email(addr);
            let _ = profile.save();
        }
        ConfigChange::ClearEmail => {
            config.clear();
        }
        _ => return,
    }

    if let Err(e) = config.save() {
        print_label(
            "warn",
            &format!("Could not save email config: {}", e),
            colors::WARN,
        );
        return;
    }

    println!();
    print_label("config", &change.description(), colors::HEADER);

    // Show tip
    if let ConfigChange::Email(_) = change {
        print_hint("When a request takes a long time, I'll email you the answer.");
    }
}

/// Show helpful tip related to the config change
fn show_config_tip(change: &ConfigChange) {
    let tip = match change {
        ConfigChange::LearningMode(true) => {
            Some("I'll explain why commands work and what they do.")
        }
        ConfigChange::LearningMode(false) => Some("To re-enable: \"Anna, enable learning mode\""),
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
        // Email tips are handled in apply_email_change
        ConfigChange::Email(_) | ConfigChange::ClearEmail => None,
        _ => None,
    };

    if let Some(t) = tip {
        print_hint(t);
    }
}

/// Show current config status
pub fn show_config_status() {
    let profile = UserProfile::load();
    let prefs = &profile.preferences;
    let pers = &prefs.personality;
    let email_config = EmailConfig::load();

    println!();
    print_section_header("settings");

    // Learning & verbosity
    if prefs.learning_mode {
        kv_colored("learning_mode", "enabled", colors::OK);
    } else {
        kv_colored("learning_mode", "disabled", colors::DIM);
    }
    kv(
        "verbosity",
        match prefs.verbosity {
            0 => "minimal",
            1 => "normal",
            _ => "detailed",
        },
    );

    // Automation
    if prefs.auto_confirm_low_risk {
        kv_colored("auto_confirm", "low-risk", colors::OK);
    } else {
        kv_colored("auto_confirm", "ask always", colors::DIM);
    }
    if prefs.show_internal_comms {
        kv_colored("internal_comms", "visible", colors::OK);
    } else {
        kv_colored("internal_comms", "hidden", colors::DIM);
    }

    // v0.0.239: Email notifications
    if let Some(ref email) = email_config.user_email {
        kv_colored("email", email, colors::OK);
    } else {
        kv_colored("email", "not set", colors::DIM);
    }

    // Personality
    println!();
    print_section_header("personality");
    kv(
        "formality",
        match pers.formality {
            0 => "casual",
            1 => "balanced",
            _ => "formal",
        },
    );
    kv(
        "humor",
        match pers.humor {
            0 => "none",
            1 => "subtle",
            _ => "playful",
        },
    );
    kv(
        "technical_depth",
        match pers.technical_depth {
            0 => "simple",
            1 => "balanced",
            _ => "expert",
        },
    );

    println!();
    print_hint("Change settings with natural language, e.g.:");
    print_hint("\"make Anna more casual\" or \"my email is user@example.com\"");
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
