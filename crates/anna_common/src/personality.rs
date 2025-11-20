//! Personality Configuration - Anna's 16-Personalities style trait system
//!
//! v5.7.0-beta.86: Database-backed 16-trait system
//! Trait-based personality model with slider values (0-10)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Database integration for personality persistence
use crate::context::db::ContextDb;

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
            "patient_vs_urgent" => match value {
                0..=3 => "Urgent. Quick responses, gets to the point fast.",
                4..=6 => "Balanced pace. Thorough but efficient.",
                7..=10 => "Patient. Takes time to explain thoroughly step-by-step.",
                _ => "Unknown",
            },
            "humble_vs_confident" => match value {
                0..=3 => "Confident. Assertive recommendations.",
                4..=6 => "Balanced self-assurance.",
                7..=10 => "Humble. Acknowledges uncertainty, suggests rather than demands.",
                _ => "Unknown",
            },
            "formal_vs_casual" => match value {
                0..=3 => "Casual. Friendly, relaxed communication.",
                4..=6 => "Professional but approachable.",
                7..=10 => "Formal. Maintains professional boundaries.",
                _ => "Unknown",
            },
            "empathetic_vs_logical" => match value {
                0..=3 => "Logical. Facts and data drive decisions.",
                4..=6 => "Balanced. Logic with understanding of impact.",
                7..=10 => "Empathetic. Considers user feelings and concerns first.",
                _ => "Unknown",
            },
            "protective_vs_empowering" => match value {
                0..=3 => "Empowering. Trusts user to make decisions.",
                4..=6 => "Balanced safety and autonomy.",
                7..=10 => "Protective. Emphasizes safety, provides strong warnings.",
                _ => "Unknown",
            },
            "traditional_vs_innovative" => match value {
                0..=3 => "Innovative. Suggests modern tools and approaches.",
                4..=6 => "Balanced. Arch Way with modern tools.",
                7..=10 => "Traditional. Prefers established, proven methods.",
                _ => "Unknown",
            },
            "collaborative_vs_independent" => match value {
                0..=3 => "Independent. Encourages self-sufficiency.",
                4..=6 => "Balanced. Guides but respects user choices.",
                7..=10 => "Collaborative. Works closely with user on decisions.",
                _ => "Unknown",
            },
            "perfectionist_vs_pragmatic" => match value {
                0..=3 => "Pragmatic. Quick solutions that work.",
                4..=6 => "Balanced. Thorough but practical.",
                7..=10 => "Perfectionist. Comprehensive, detailed solutions.",
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
    // Beta.86: Full 16-trait personality system
    vec![
        // Original 8 traits from beta.83
        PersonalityTrait::new("introvert_vs_extrovert", "Introvert vs Extrovert", 3),  // Reserved, speaks when it matters
        PersonalityTrait::new("calm_vs_excitable", "Calm vs Excitable", 8),             // Calm, reassuring tone
        PersonalityTrait::new("direct_vs_diplomatic", "Direct vs Diplomatic", 7),       // Clear and direct
        PersonalityTrait::new("playful_vs_serious", "Playful vs Serious", 6),           // Occasional light humor
        PersonalityTrait::new("cautious_vs_bold", "Cautious vs Bold", 6),               // Balanced risk approach
        PersonalityTrait::new("minimalist_vs_verbose", "Minimalist vs Verbose", 7),     // Concise but complete
        PersonalityTrait::new("analytical_vs_intuitive", "Analytical vs Intuitive", 8), // Structured, logical
        PersonalityTrait::new("reassuring_vs_challenging", "Reassuring vs Challenging", 6), // Supportive but honest

        // New 8 traits for beta.86
        PersonalityTrait::new("patient_vs_urgent", "Patient vs Urgent", 7),             // Takes time to explain thoroughly
        PersonalityTrait::new("humble_vs_confident", "Humble vs Confident", 6),         // Balanced self-assurance
        PersonalityTrait::new("formal_vs_casual", "Formal vs Casual", 5),               // Professional but approachable
        PersonalityTrait::new("empathetic_vs_logical", "Empathetic vs Logical", 7),     // Logic-driven with understanding
        PersonalityTrait::new("protective_vs_empowering", "Protective vs Empowering", 6), // Balanced safety and autonomy
        PersonalityTrait::new("traditional_vs_innovative", "Traditional vs Innovative", 5), // Arch Way with modern tools
        PersonalityTrait::new("collaborative_vs_independent", "Collaborative vs Independent", 6), // Guides but respects choices
        PersonalityTrait::new("perfectionist_vs_pragmatic", "Perfectionist vs Pragmatic", 6), // Thorough but practical
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

    /// Validate trait interactions to prevent conflicting personality states
    /// Returns Ok(()) if all interactions are valid, Err with list of conflicts otherwise
    pub fn validate_interactions(&self) -> Result<(), Vec<String>> {
        let mut conflicts = Vec::new();

        // Helper to get trait value safely
        let get_val = |key: &str| -> Option<u8> {
            self.get_trait(key).map(|t| t.value)
        };

        // Conflict 1: Can't be both very introverted and very bold
        if let (Some(intro), Some(bold)) = (
            get_val("introvert_vs_extrovert"),
            get_val("cautious_vs_bold"),
        ) {
            if intro >= 8 && bold <= 2 {
                conflicts.push("Conflicting: Very introverted (reserved) but very bold (risk-taking)".to_string());
            }
        }

        // Conflict 2: Can't be both very calm and very excitable
        if let (Some(_calm), Some(_excit)) = (
            get_val("calm_vs_excitable"),
            get_val("calm_vs_excitable"),  // Note: excitable is inverse of calm
        ) {
            // This is actually the same trait, so no conflict possible
            // Leaving as example of how to check related traits
        }

        // Conflict 3: Can't be both very minimalist and very verbose
        if let (Some(_minimal), Some(_verbose)) = (
            get_val("minimalist_vs_verbose"),
            get_val("minimalist_vs_verbose"),  // Same trait, opposite poles
        ) {
            // These are opposite poles of the same trait, so no conflict check needed
            // Minimalist=10 means not verbose, minimalist=0 means very verbose
        }

        // Conflict 4: Can't be both very analytical and very intuitive
        // (These are opposite poles of the same trait, so no separate check needed)

        // Conflict 5: Can't be both very protective and very empowering
        // (These are opposite poles of the same trait)

        // Conflict 6: Can't be both perfectionist and very urgent
        if let (Some(perfect), Some(urgent)) = (
            get_val("perfectionist_vs_pragmatic"),
            get_val("patient_vs_urgent"),
        ) {
            if perfect >= 8 && urgent <= 2 {
                conflicts.push("Conflicting: Perfectionist (thorough) but very urgent (rushes)".to_string());
            }
        }

        // Conflict 7: Can't be both very humble and very confident
        // (These are opposite poles of the same trait)

        // Conflict 8: Can't be both very formal and very playful
        if let (Some(formal), Some(playful)) = (
            get_val("formal_vs_casual"),
            get_val("playful_vs_serious"),
        ) {
            if formal >= 8 && playful >= 8 {
                conflicts.push("Conflicting: Very formal but very playful (humor)".to_string());
            }
        }

        // Conflict 9: Can't be both very logical and very empathetic
        // (These are opposite poles of the same trait)

        // Conflict 10: Can't be both very challenging and very reassuring
        // (These are opposite poles of the same trait)

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(conflicts)
        }
    }

    /// Load personality configuration from database
    /// Returns default config if no traits found in database
    pub async fn load_from_db(db: &ContextDb) -> Result<Self> {
        let conn = db.conn();
        let traits = tokio::task::spawn_blocking(move || -> Result<Vec<PersonalityTrait>> {
            let conn_guard = conn.blocking_lock();
            let mut stmt = conn_guard.prepare(
                "SELECT trait_key, trait_name, value FROM personality ORDER BY id"
            )?;

            let traits: Result<Vec<PersonalityTrait>, _> = stmt
                .query_map([], |row| {
                    let key: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let value: u8 = row.get::<_, i64>(2)? as u8;
                    Ok(PersonalityTrait::new(&key, &name, value))
                })?
                .collect();

            traits.map_err(|e| anyhow::anyhow!("Failed to load personality traits: {}", e))
        })
        .await??;

        if traits.is_empty() {
            // No traits in database, return default
            Ok(Self::default())
        } else {
            Ok(Self {
                traits,
                active: true,
            })
        }
    }

    /// Save personality configuration to database
    /// Uses UPSERT (INSERT OR REPLACE) to handle updates
    pub async fn save_to_db(&self, db: &ContextDb) -> Result<()> {
        let traits = self.traits.clone();
        let conn = db.conn();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut conn_guard = conn.blocking_lock();
            let tx = conn_guard.transaction()?;

            for trait_item in &traits {
                tx.execute(
                    "INSERT OR REPLACE INTO personality (trait_key, trait_name, value, updated_at)
                     VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
                    [&trait_item.key, &trait_item.name, &trait_item.value.to_string()],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
        .await??;

        Ok(())
    }

    /// Migrate personality configuration from TOML file to database
    /// This is a one-time migration for existing installations
    pub async fn migrate_from_toml(db: &ContextDb) -> Result<()> {
        // Load from TOML if it exists
        let toml_config = if let Some(path) = Self::user_config_path() {
            if path.exists() {
                Self::load_from_path(&path)?
            } else {
                // No TOML file, nothing to migrate
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // Save to database
        toml_config.save_to_db(db).await?;

        // Optionally backup the TOML file
        if let Some(path) = Self::user_config_path() {
            let backup_path = path.with_extension("toml.migrated");
            std::fs::rename(&path, &backup_path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_personality() {
        let config = PersonalityConfig::default();
        assert!(config.active);
        assert_eq!(config.traits.len(), 16);  // Beta.86: Upgraded from 8 to 16 traits
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
        assert_eq!(trait_ref.unwrap().value, 3);  // Beta.83: Changed from 8 to 3 to match INTERNAL_PROMPT.md
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
