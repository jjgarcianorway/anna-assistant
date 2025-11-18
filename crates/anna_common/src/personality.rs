//! Personality Configuration - Anna's 16-Personalities style trait system
//!
//! v5.7.0-beta.55: Telemetry and Internal Dialogue Upgrade
//! Trait-based personality model with slider values (0-10)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single personality trait with slider value (0-10)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTrait {
    /// Trait key (e.g., "introvert_vs_extrovert")
    pub key: String,

    /// Display name (e.g., "Introvert vs Extrovert")
    pub name: String,

    /// Value from 0 (first pole) to 10 (second pole)
    /// Example: 0 = very extrovert, 10 = very introvert
    pub value: u8,

    /// Description of what this value means
    pub meaning: String,
}

impl PersonalityTrait {
    /// Create a new trait
    pub fn new(key: &str, name: &str, value: u8) -> Self {
        let meaning = Self::compute_meaning(key, value);
        Self {
            key: key.to_string(),
            name: name.to_string(),
            value: value.clamp(0, 10),
            meaning,
        }
    }

    /// Update value and recompute meaning
    pub fn set_value(&mut self, value: u8) {
        self.value = value.clamp(0, 10);
        self.meaning = Self::compute_meaning(&self.key, self.value);
    }

    /// Get visual bar representation (10 chars)
    pub fn bar(&self) -> String {
        let filled = (self.value as usize).min(10);
        let empty = 10 - filled;
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }

    /// Compute meaning based on trait key and value
    fn compute_meaning(key: &str, value: u8) -> String {
        match key {
            "introvert_vs_extrovert" => match value {
                0..=3 => "Extrovert style. Frequent, chatty communication.",
                4..=6 => "Balanced. Communicates when needed.",
                7..=10 => "Introvert style. Reserved, focused messages.",
                _ => "Unknown",
            },
            "cautious_vs_bold" => match value {
                0..=3 => "Bold. Takes calculated risks confidently.",
                4..=6 => "Balanced risk approach.",
                7..=10 => "Cautious. Always proposes backups and safety checks.",
                _ => "Unknown",
            },
            "direct_vs_diplomatic" => match value {
                0..=3 => "Diplomatic. Gentle, considerate phrasing.",
                4..=6 => "Balanced communication style.",
                7..=10 => "Direct. Clear, straightforward language.",
                _ => "Unknown",
            },
            "playful_vs_serious" => match value {
                0..=3 => "Serious. Professional, no humor.",
                4..=6 => "Balanced. Occasional light humor.",
                7..=10 => "Playful. Frequent wit and irony.",
                _ => "Unknown",
            },
            "minimalist_vs_verbose" => match value {
                0..=3 => "Verbose. Detailed explanations.",
                4..=6 => "Balanced. Adequate detail.",
                7..=10 => "Minimalist. Concise, essential information only.",
                _ => "Unknown",
            },
            "calm_vs_excitable" => match value {
                0..=3 => "Excitable. Energetic, enthusiastic responses.",
                4..=6 => "Balanced energy. Professional calm.",
                7..=10 => "Calm. Reassuring, measured tone.",
                _ => "Unknown",
            },
            "analytical_vs_intuitive" => match value {
                0..=3 => "Intuitive. Pattern-based, gut-feeling approach.",
                4..=6 => "Balanced. Logic with intuition.",
                7..=10 => "Analytical. Structured, logical thinking.",
                _ => "Unknown",
            },
            "reassuring_vs_challenging" => match value {
                0..=3 => "Challenging. Questions assumptions, plays devil's advocate.",
                4..=6 => "Balanced. Supportive but honest.",
                7..=10 => "Reassuring. Encouraging, validating tone.",
                _ => "Unknown",
            },
            _ => "Unknown trait",
        }.to_string()
    }
}

/// Personality configuration with 16-personalities style traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// Trait sliders (0-10 scale)
    #[serde(default = "default_traits")]
    pub traits: Vec<PersonalityTrait>,

    /// Whether personality system is active
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_traits() -> Vec<PersonalityTrait> {
    // Beta.83: Aligned with INTERNAL_PROMPT.md specification
    vec![
        PersonalityTrait::new("introvert_vs_extrovert", "Introvert vs Extrovert", 3),  // Reserved, speaks when it matters
        PersonalityTrait::new("calm_vs_excitable", "Calm vs Excitable", 8),             // Calm, reassuring tone
        PersonalityTrait::new("direct_vs_diplomatic", "Direct vs Diplomatic", 7),       // Clear and direct
        PersonalityTrait::new("playful_vs_serious", "Playful vs Serious", 6),           // Occasional light humor
        PersonalityTrait::new("cautious_vs_bold", "Cautious vs Bold", 6),               // Balanced risk approach
        PersonalityTrait::new("minimalist_vs_verbose", "Minimalist vs Verbose", 7),     // Concise but complete
        PersonalityTrait::new("analytical_vs_intuitive", "Analytical vs Intuitive", 8), // Structured, logical
        PersonalityTrait::new("reassuring_vs_challenging", "Reassuring vs Challenging", 6), // Supportive but honest
    ]
}

fn default_active() -> bool {
    true
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            traits: default_traits(),
            active: default_active(),
        }
    }
}

impl PersonalityConfig {
    /// Load personality config from file
    /// Checks ~/.config/anna/personality.toml first, then /etc/anna/personality.toml
    pub fn load() -> Self {
        // Try user config first
        if let Some(path) = Self::user_config_path() {
            if let Ok(config) = Self::load_from_path(&path) {
                return config;
            }
        }

        // Try system config
        if let Ok(config) = Self::load_from_path(&PathBuf::from("/etc/anna/personality.toml")) {
            return config;
        }

        // Default if neither exists
        Self::default()
    }

    /// Load from specific path
    fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: PersonalityConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save to user config
    pub fn save(&self) -> Result<()> {
        let path = Self::user_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get user config path: ~/.config/anna/personality.toml
    fn user_config_path() -> Option<PathBuf> {
        let config_dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else {
            let home = std::env::var("HOME").ok()?;
            PathBuf::from(home).join(".config")
        };

        Some(config_dir.join("anna").join("personality.toml"))
    }

    /// Get a trait by key
    pub fn get_trait(&self, key: &str) -> Option<&PersonalityTrait> {
        self.traits.iter().find(|t| t.key == key)
    }

    /// Get a mutable trait by key
    pub fn get_trait_mut(&mut self, key: &str) -> Option<&mut PersonalityTrait> {
        self.traits.iter_mut().find(|t| t.key == key)
    }

    /// Set a trait value
    pub fn set_trait(&mut self, key: &str, value: u8) -> Result<()> {
        if let Some(trait_ref) = self.get_trait_mut(key) {
            trait_ref.set_value(value);
            Ok(())
        } else {
            anyhow::bail!("Unknown trait key: {}", key)
        }
    }

    /// Adjust a trait by delta (-10 to +10)
    pub fn adjust_trait(&mut self, key: &str, delta: i8) -> Result<()> {
        if let Some(trait_ref) = self.get_trait_mut(key) {
            let new_value = (trait_ref.value as i8 + delta).clamp(0, 10) as u8;
            trait_ref.set_value(new_value);
            Ok(())
        } else {
            anyhow::bail!("Unknown trait key: {}", key)
        }
    }

    /// Render personality view for LLM prompts
    pub fn render_personality_view(&self) -> String {
        if !self.active {
            return "[ANNA_PERSONALITY_VIEW]\nactive: false\n[/ANNA_PERSONALITY_VIEW]\n".to_string();
        }

        let mut view = String::from("[ANNA_PERSONALITY_VIEW]\n");
        view.push_str("active: true\n");
        view.push_str("traits:\n");

        for trait_item in &self.traits {
            view.push_str(&format!("  - name: \"{}\"\n", trait_item.name));
            view.push_str(&format!("    key: \"{}\"\n", trait_item.key));
            view.push_str(&format!("    value: {}\n", trait_item.value));
            view.push_str(&format!("    bar: \"{}\"\n", trait_item.bar()));
            view.push_str(&format!("    meaning: \"{}\"\n", trait_item.meaning));
        }

        view.push_str("\ncommentary: |\n");
        view.push_str("  You can say things like \"be more direct\" or \"more minimalist\" and I will\n");
        view.push_str("  adjust these traits and show you the new map.\n");
        view.push_str("[/ANNA_PERSONALITY_VIEW]\n");

        view
    }

    /// Parse natural language trait adjustment request
    /// Examples: "be more direct", "less serious", "set minimalist to 8"
    pub fn parse_adjustment(&self, request: &str) -> Option<(String, i8)> {
        let request_lower = request.to_lowercase();

        // Pattern: "more X" or "X more"
        if request_lower.contains("more") {
            for trait_item in &self.traits {
                let trait_words: Vec<&str> = trait_item.key.split('_').collect();
                for word in &trait_words {
                    if request_lower.contains(word) {
                        return Some((trait_item.key.clone(), 2));
                    }
                }
            }
        }

        // Pattern: "less X" or "X less"
        if request_lower.contains("less") {
            for trait_item in &self.traits {
                let trait_words: Vec<&str> = trait_item.key.split('_').collect();
                for word in &trait_words {
                    if request_lower.contains(word) {
                        return Some((trait_item.key.clone(), -2));
                    }
                }
            }
        }

        // Pattern: "set X to N"
        // This would need more complex parsing, left for future enhancement

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_personality() {
        let config = PersonalityConfig::default();
        assert!(config.active);
        assert_eq!(config.traits.len(), 8);
    }

    #[test]
    fn test_trait_creation() {
        let trait_item = PersonalityTrait::new("introvert_vs_extrovert", "Introvert vs Extrovert", 8);
        assert_eq!(trait_item.key, "introvert_vs_extrovert");
        assert_eq!(trait_item.value, 8);
        assert!(!trait_item.meaning.is_empty());
    }

    #[test]
    fn test_trait_value_clamping() {
        let mut trait_item = PersonalityTrait::new("test", "Test", 5);

        // Test upper bound
        trait_item.set_value(15);
        assert_eq!(trait_item.value, 10);

        // Test lower bound
        trait_item.set_value(200); // Will overflow to 0
        trait_item.set_value(0);
        assert_eq!(trait_item.value, 0);
    }

    #[test]
    fn test_trait_bar_rendering() {
        let trait_item = PersonalityTrait::new("test", "Test", 7);
        let bar = trait_item.bar();
        assert_eq!(bar.chars().filter(|&c| c == '█').count(), 7);
        assert_eq!(bar.chars().filter(|&c| c == '░').count(), 3);
    }

    #[test]
    fn test_get_trait() {
        let config = PersonalityConfig::default();
        let trait_ref = config.get_trait("introvert_vs_extrovert");
        assert!(trait_ref.is_some());
        assert_eq!(trait_ref.unwrap().value, 8);
    }

    #[test]
    fn test_set_trait() {
        let mut config = PersonalityConfig::default();
        let result = config.set_trait("direct_vs_diplomatic", 9);
        assert!(result.is_ok());

        let trait_ref = config.get_trait("direct_vs_diplomatic");
        assert_eq!(trait_ref.unwrap().value, 9);
    }

    #[test]
    fn test_adjust_trait() {
        let mut config = PersonalityConfig::default();

        // Get initial value
        let initial = config.get_trait("playful_vs_serious").unwrap().value;

        // Adjust up
        config.adjust_trait("playful_vs_serious", 2).unwrap();
        assert_eq!(config.get_trait("playful_vs_serious").unwrap().value, initial + 2);

        // Adjust down
        config.adjust_trait("playful_vs_serious", -1).unwrap();
        assert_eq!(config.get_trait("playful_vs_serious").unwrap().value, initial + 1);
    }

    #[test]
    fn test_render_personality_view() {
        let config = PersonalityConfig::default();
        let view = config.render_personality_view();

        assert!(view.contains("[ANNA_PERSONALITY_VIEW]"));
        assert!(view.contains("active: true"));
        assert!(view.contains("introvert_vs_extrovert"));
        assert!(view.contains("[/ANNA_PERSONALITY_VIEW]"));
    }

    #[test]
    fn test_parse_adjustment() {
        let config = PersonalityConfig::default();

        // Test "more X" pattern
        let result = config.parse_adjustment("be more direct");
        assert!(result.is_some());
        let (key, delta) = result.unwrap();
        assert_eq!(key, "direct_vs_diplomatic");
        assert_eq!(delta, 2);

        // Test "less X" pattern
        let result = config.parse_adjustment("less serious");
        assert!(result.is_some());
        let (key, delta) = result.unwrap();
        assert_eq!(key, "playful_vs_serious");
        assert_eq!(delta, -2);
    }
}
