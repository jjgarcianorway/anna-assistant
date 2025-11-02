//! Persona system for Anna Assistant
//!
//! Personas are pre-configured bundles of UI preferences that adapt Anna's
//! communication style to different user contexts (dev, ops, gamer, minimal).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A persona defines Anna's communication style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub description: String,
    pub traits: PersonaTraits,
}

/// Traits that define how Anna communicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaTraits {
    /// Verbosity level (1-5, where 1 is terse and 5 is chatty)
    pub verbosity: u8,
    /// Whether to use emojis
    pub emojis: bool,
    /// Color intensity (1-3, where 1 is minimal and 3 is vibrant)
    pub colorfulness: u8,
    /// How often to show tips (1-5)
    pub tips_frequency: u8,
}

/// Persona selection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PersonaMode {
    Auto,  // Auto-detect based on context
    Fixed, // User explicitly set, don't change
}

/// Current persona state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaState {
    pub current: String,
    pub mode: PersonaMode,
    pub applied_at: String, // ISO 8601 timestamp
}

/// Bundled personas (from personas.yaml)
pub fn bundled_personas() -> HashMap<String, Persona> {
    let mut personas = HashMap::new();

    personas.insert(
        "dev".to_string(),
        Persona {
            name: "dev".to_string(),
            description: "For developers: detailed logs, technical terms welcome, lots of context"
                .to_string(),
            traits: PersonaTraits {
                verbosity: 4,
                emojis: true,
                colorfulness: 3,
                tips_frequency: 3,
            },
        },
    );

    personas.insert(
        "ops".to_string(),
        Persona {
            name: "ops".to_string(),
            description: "For operators: concise, actionable, status-focused".to_string(),
            traits: PersonaTraits {
                verbosity: 2,
                emojis: false,
                colorfulness: 2,
                tips_frequency: 1,
            },
        },
    );

    personas.insert(
        "gamer".to_string(),
        Persona {
            name: "gamer".to_string(),
            description: "For gamers: fun, casual, emoji-heavy, achievement-oriented".to_string(),
            traits: PersonaTraits {
                verbosity: 3,
                emojis: true,
                colorfulness: 3,
                tips_frequency: 4,
            },
        },
    );

    personas.insert(
        "minimal".to_string(),
        Persona {
            name: "minimal".to_string(),
            description: "For minimalists: terse output, no colors, no emojis, just facts"
                .to_string(),
            traits: PersonaTraits {
                verbosity: 1,
                emojis: false,
                colorfulness: 1,
                tips_frequency: 1,
            },
        },
    );

    personas
}

/// Get current persona state
pub fn get_persona_state() -> Result<PersonaState> {
    let state_path = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Cannot determine home directory")?;
    let state_path = std::path::PathBuf::from(state_path).join(".config/anna/state.json");

    if state_path.exists() {
        let content = std::fs::read_to_string(&state_path)?;
        let state: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(persona_obj) = state.get("persona") {
            let persona_state: PersonaState = serde_json::from_value(persona_obj.clone())
                .unwrap_or_else(|_| default_persona_state());
            return Ok(persona_state);
        }
    }

    Ok(default_persona_state())
}

/// Save persona state
pub fn save_persona_state(state: &PersonaState) -> Result<()> {
    let state_path = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Cannot determine home directory")?;
    let state_dir = std::path::PathBuf::from(state_path).join(".config/anna");
    let state_path = state_dir.join("state.json");

    // Ensure directory exists
    std::fs::create_dir_all(&state_dir)?;

    // Load existing state or create new
    let mut full_state: serde_json::Value = if state_path.exists() {
        let content = std::fs::read_to_string(&state_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Update persona section
    if let serde_json::Value::Object(ref mut map) = full_state {
        map.insert("persona".to_string(), serde_json::to_value(state)?);
    }

    // Write back
    let json = serde_json::to_string_pretty(&full_state)?;
    std::fs::write(&state_path, json)?;

    Ok(())
}

/// Default persona state
fn default_persona_state() -> PersonaState {
    PersonaState {
        current: "dev".to_string(), // Default to dev persona
        mode: PersonaMode::Auto,
        applied_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Set persona
pub fn set_persona(name: &str, mode: PersonaMode) -> Result<()> {
    // Validate persona exists
    let personas = bundled_personas();
    if !personas.contains_key(name) {
        bail!(
            "Unknown persona '{}'. Available: {}",
            name,
            personas
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let state = PersonaState {
        current: name.to_string(),
        mode,
        applied_at: chrono::Utc::now().to_rfc3339(),
    };

    save_persona_state(&state)?;

    // Apply persona traits to config
    apply_persona_to_config(&personas[name])?;

    Ok(())
}

/// Apply persona traits to user configuration
fn apply_persona_to_config(persona: &Persona) -> Result<()> {
    use crate::config_governance::set_user_config;

    set_user_config("ui.emojis", serde_json::json!(persona.traits.emojis))?;
    set_user_config(
        "ui.colors",
        serde_json::json!(persona.traits.colorfulness > 1),
    )?;
    set_user_config("ui.verbosity", serde_json::json!(persona.traits.verbosity))?;

    Ok(())
}

/// Get explanation for current persona choice
pub fn explain_persona_choice() -> String {
    // For now, this is a stub. In the future, this could analyze:
    // - Command history
    // - Time of day
    // - System load
    // - User's typical interaction patterns

    "I don't profile yet; I'm using your explicit selection or the default. \
     In the future, I'll learn from your patterns to suggest better fits."
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundled_personas() {
        let personas = bundled_personas();
        assert!(personas.contains_key("dev"));
        assert!(personas.contains_key("ops"));
        assert!(personas.contains_key("gamer"));
        assert!(personas.contains_key("minimal"));
    }

    #[test]
    fn test_persona_traits() {
        let personas = bundled_personas();
        let dev = &personas["dev"];
        assert_eq!(dev.traits.verbosity, 4);
        assert!(dev.traits.emojis);

        let minimal = &personas["minimal"];
        assert_eq!(minimal.traits.verbosity, 1);
        assert!(!minimal.traits.emojis);
    }
}
