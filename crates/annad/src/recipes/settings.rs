//! Conversation settings via natural language.
//! v0.0.998: Initial implementation
//!
//! Allows users to change Anna's behavior through natural language:
//! - "be more verbose" / "be brief"
//! - "show commands" / "hide commands"
//! - "enable confirmations" / "disable confirmations"

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

use super::RecipeResult;

/// User's conversation preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationSettings {
    /// Verbosity level: "brief", "normal", "verbose"
    pub verbosity: String,
    /// Show command execution steps
    pub show_commands: bool,
    /// Ask for confirmation before making changes
    pub ask_confirmations: bool,
    /// Show wiki search steps
    pub show_wiki: bool,
    /// Show LLM prompts (for debugging)
    pub show_prompts: bool,
}

impl ConversationSettings {
    pub fn new() -> Self {
        Self {
            verbosity: "normal".to_string(),
            show_commands: true,
            ask_confirmations: true,
            show_wiki: true,
            show_prompts: false,
        }
    }

    fn settings_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna/conversation_settings.json")
    }

    pub fn load() -> Self {
        let path = Self::settings_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::new()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Try to match a settings-related request
/// v0.2.5: Fixed false positives - requires explicit Anna/conversation context
pub fn try_recipe(q: &str) -> Option<RecipeResult> {
    let q_lower = q.to_lowercase();

    // v0.2.5: Must contain context indicating this is about Anna's behavior, not system info
    // Words that indicate user is talking TO Anna about her behavior
    let anna_context = q_lower.contains("be ") || q_lower.contains("anna") ||
        q_lower.contains("your ") || q_lower.contains("you ") ||
        q_lower.contains("conversation") || q_lower.contains("answer") ||
        q_lower.contains("response") || q_lower.contains("explain");

    // Verbosity settings - must have context indicating this is about Anna's output
    if anna_context {
        if q_lower.contains("verbose") || q_lower.contains("more detail") {
            return Some(set_verbosity("verbose"));
        }
        if q_lower.contains("brief") || q_lower.contains("concise") || q_lower.contains("short") {
            return Some(set_verbosity("brief"));
        }
    }
    if q_lower.contains("normal") && q_lower.contains("verbosity") {
        return Some(set_verbosity("normal"));
    }

    // Command display - "show commands" vs "show your commands"
    // Only match when clearly about Anna's command display
    if q_lower.contains("show") && q_lower.contains("command") && anna_context {
        return Some(set_show_commands(true));
    }
    if (q_lower.contains("hide") || q_lower.contains("don't show")) && q_lower.contains("command") {
        return Some(set_show_commands(false));
    }

    // Confirmation settings - must have skip/disable context
    if q_lower.contains("confirm") {
        if q_lower.contains("skip") || (q_lower.contains("disable") && anna_context) {
            return Some(set_confirmations(false));
        }
        if q_lower.contains("enable") && anna_context {
            return Some(set_confirmations(true));
        }
    }

    // Wiki display - specific to Anna's wiki search feature
    if q_lower.contains("wiki") && q_lower.contains("search") {
        if q_lower.contains("hide") || q_lower.contains("don't") {
            return Some(set_show_wiki(false));
        }
        if q_lower.contains("show") {
            return Some(set_show_wiki(true));
        }
    }

    // Show current settings - must be specifically about Anna's settings
    // v0.2.5: Only match "anna settings", "your settings", "conversation settings"
    // NOT "DNS configuration" or "system config"
    if q_lower.contains("setting") || q_lower.contains("preference") {
        if anna_context || q_lower.contains("anna") {
            if q_lower.contains("show") || q_lower.contains("what") || q_lower.contains("current") {
                return Some(show_settings());
            }
        }
    }

    // Reset settings
    if q_lower.contains("reset") && (q_lower.contains("setting") || q_lower.contains("preference")) {
        if anna_context || q_lower.contains("default") {
            return Some(reset_settings());
        }
    }

    None
}

fn set_verbosity(level: &str) -> RecipeResult {
    let mut settings = ConversationSettings::load();
    settings.verbosity = level.to_string();

    match settings.save() {
        Ok(_) => {
            info!("Changed verbosity to: {}", level);
            let msg = match level {
                "brief" => "Got it! I'll keep my answers short and to the point.",
                "verbose" => "Understood! I'll provide more detailed explanations.",
                _ => "Okay, back to normal verbosity.",
            };
            RecipeResult {
                success: true,
                message: msg.to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Couldn't save that setting: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}

fn set_show_commands(show: bool) -> RecipeResult {
    let mut settings = ConversationSettings::load();
    settings.show_commands = show;

    match settings.save() {
        Ok(_) => {
            info!("Changed show_commands to: {}", show);
            let msg = if show {
                "I'll show the commands I run from now on."
            } else {
                "Got it, I'll hide the command execution details."
            };
            RecipeResult {
                success: true,
                message: msg.to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Couldn't save that setting: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}

fn set_confirmations(enabled: bool) -> RecipeResult {
    let mut settings = ConversationSettings::load();
    settings.ask_confirmations = enabled;

    match settings.save() {
        Ok(_) => {
            info!("Changed ask_confirmations to: {}", enabled);
            let msg = if enabled {
                "I'll ask for confirmation before making changes."
            } else {
                "I'll skip confirmation prompts. Be careful - I'll make changes directly!"
            };
            RecipeResult {
                success: true,
                message: msg.to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Couldn't save that setting: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}

fn set_show_wiki(show: bool) -> RecipeResult {
    let mut settings = ConversationSettings::load();
    settings.show_wiki = show;

    match settings.save() {
        Ok(_) => {
            info!("Changed show_wiki to: {}", show);
            let msg = if show {
                "I'll show when I'm searching the Arch Wiki."
            } else {
                "I'll search the wiki silently in the background."
            };
            RecipeResult {
                success: true,
                message: msg.to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Couldn't save that setting: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}

fn show_settings() -> RecipeResult {
    let settings = ConversationSettings::load();

    let mut msg = String::from("Current conversation settings:\n\n");
    msg.push_str(&format!("  Verbosity: {}\n", settings.verbosity));
    msg.push_str(&format!("  Show commands: {}\n", if settings.show_commands { "yes" } else { "no" }));
    msg.push_str(&format!("  Ask confirmations: {}\n", if settings.ask_confirmations { "yes" } else { "no" }));
    msg.push_str(&format!("  Show wiki searches: {}\n", if settings.show_wiki { "yes" } else { "no" }));
    msg.push_str("\nYou can change these by saying things like:\n");
    msg.push_str("  - \"be more verbose\"\n");
    msg.push_str("  - \"be brief\"\n");
    msg.push_str("  - \"hide commands\"\n");
    msg.push_str("  - \"skip confirmations\"\n");
    msg.push_str("  - \"reset settings\"");

    RecipeResult {
        success: true,
        message: msg,
        needs_confirmation: false,
        confirmation_prompt: None,
    }
}

fn reset_settings() -> RecipeResult {
    let settings = ConversationSettings::new();

    match settings.save() {
        Ok(_) => {
            info!("Reset conversation settings to defaults");
            RecipeResult {
                success: true,
                message: "Settings reset to defaults.".to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Couldn't reset settings: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}

/// Get current settings (for use by other modules)
pub fn get_settings() -> ConversationSettings {
    ConversationSettings::load()
}
