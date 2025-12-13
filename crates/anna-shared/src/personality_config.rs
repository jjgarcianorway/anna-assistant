// v0.0.542: Personality Config (Phase 118)
// Configurable personality traits via natural language per VISION.md

use serde::{Deserialize, Serialize};

/// Formality level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FormalityLevel {
    Casual,
    #[default]
    Professional,
    Formal,
    Technical,
}

impl std::fmt::Display for FormalityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Casual => write!(f, "Casual"),
            Self::Professional => write!(f, "Professional"),
            Self::Formal => write!(f, "Formal"),
            Self::Technical => write!(f, "Technical"),
        }
    }
}

/// Friendliness level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FriendlinessLevel {
    Reserved,
    #[default]
    Balanced,
    Friendly,
    Enthusiastic,
}

impl std::fmt::Display for FriendlinessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reserved => write!(f, "Reserved"),
            Self::Balanced => write!(f, "Balanced"),
            Self::Friendly => write!(f, "Friendly"),
            Self::Enthusiastic => write!(f, "Enthusiastic"),
        }
    }
}

/// Humor level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum HumorLevel {
    None,
    #[default]
    Occasional,
    Frequent,
}

impl std::fmt::Display for HumorLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Occasional => write!(f, "Occasional"),
            Self::Frequent => write!(f, "Frequent"),
        }
    }
}

/// Verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum VerbosityLevel {
    Minimal,
    Concise,
    #[default]
    Normal,
    Detailed,
    Verbose,
}

impl std::fmt::Display for VerbosityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "Minimal"),
            Self::Concise => write!(f, "Concise"),
            Self::Normal => write!(f, "Normal"),
            Self::Detailed => write!(f, "Detailed"),
            Self::Verbose => write!(f, "Verbose"),
        }
    }
}

/// Explanation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ExplanationStyle {
    JustAnswer,
    #[default]
    Brief,
    Educational,
    StepByStep,
}

impl std::fmt::Display for ExplanationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JustAnswer => write!(f, "Just Answer"),
            Self::Brief => write!(f, "Brief"),
            Self::Educational => write!(f, "Educational"),
            Self::StepByStep => write!(f, "Step by Step"),
        }
    }
}

/// Full personality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    pub formality: FormalityLevel,
    pub friendliness: FriendlinessLevel,
    pub humor: HumorLevel,
    pub verbosity: VerbosityLevel,
    pub explanation_style: ExplanationStyle,
    pub use_emojis: bool,
    pub use_metaphors: bool,
    pub custom_name: Option<String>,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            formality: FormalityLevel::default(),
            friendliness: FriendlinessLevel::default(),
            humor: HumorLevel::default(),
            verbosity: VerbosityLevel::default(),
            explanation_style: ExplanationStyle::default(),
            use_emojis: false,
            use_metaphors: true,
            custom_name: None,
        }
    }
}

impl PersonalityConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set formality
    pub fn with_formality(mut self, level: FormalityLevel) -> Self {
        self.formality = level;
        self
    }

    /// Set friendliness
    pub fn with_friendliness(mut self, level: FriendlinessLevel) -> Self {
        self.friendliness = level;
        self
    }

    /// Set humor
    pub fn with_humor(mut self, level: HumorLevel) -> Self {
        self.humor = level;
        self
    }

    /// Set verbosity
    pub fn with_verbosity(mut self, level: VerbosityLevel) -> Self {
        self.verbosity = level;
        self
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Formality changes
        if lower.contains("more formal") || lower.contains("be formal") {
            self.formality = FormalityLevel::Formal;
            return Some("I'll be more formal from now on.".to_string());
        }
        if lower.contains("less formal") || lower.contains("be casual") {
            self.formality = FormalityLevel::Casual;
            return Some("Sure, I'll keep it casual!".to_string());
        }
        if lower.contains("be professional") {
            self.formality = FormalityLevel::Professional;
            return Some("Professional mode activated.".to_string());
        }
        if lower.contains("be technical") {
            self.formality = FormalityLevel::Technical;
            return Some("Technical mode enabled.".to_string());
        }

        // Friendliness changes
        if lower.contains("be friendlier") || lower.contains("more friendly") {
            self.friendliness = FriendlinessLevel::Friendly;
            return Some("I'll be friendlier! Nice to chat with you.".to_string());
        }
        if lower.contains("be enthusiastic") || lower.contains("more enthusiastic") {
            self.friendliness = FriendlinessLevel::Enthusiastic;
            return Some("Awesome! I'm so excited to help!".to_string());
        }
        if lower.contains("be reserved") || lower.contains("less friendly") {
            self.friendliness = FriendlinessLevel::Reserved;
            return Some("Understood. I'll be more reserved.".to_string());
        }

        // Verbosity changes
        if lower.contains("be concise") || lower.contains("shorter answers") {
            self.verbosity = VerbosityLevel::Concise;
            return Some("Got it - concise answers.".to_string());
        }
        if lower.contains("be verbose") || lower.contains("detailed answers") || lower.contains("more detail") {
            self.verbosity = VerbosityLevel::Detailed;
            return Some("I'll provide more detailed explanations from now on.".to_string());
        }
        if lower.contains("minimal") || lower.contains("just the answer") {
            self.verbosity = VerbosityLevel::Minimal;
            self.explanation_style = ExplanationStyle::JustAnswer;
            return Some("Minimal mode. Just answers.".to_string());
        }

        // Humor changes
        if lower.contains("no humor") || lower.contains("be serious") {
            self.humor = HumorLevel::None;
            return Some("Humor disabled. All business now.".to_string());
        }
        if lower.contains("more humor") || lower.contains("be funny") {
            self.humor = HumorLevel::Frequent;
            return Some("I'll try to add more humor! Though I promise nothing about the quality...".to_string());
        }

        // Emoji toggle
        if lower.contains("use emoji") || lower.contains("enable emoji") {
            self.use_emojis = true;
            return Some("Emojis enabled! 🎉".to_string());
        }
        if lower.contains("no emoji") || lower.contains("disable emoji") {
            self.use_emojis = false;
            return Some("Emojis disabled.".to_string());
        }

        // Explanation style
        if lower.contains("explain step") || lower.contains("step by step") {
            self.explanation_style = ExplanationStyle::StepByStep;
            return Some("I'll explain things step by step.".to_string());
        }
        if lower.contains("educational") || lower.contains("teach me") {
            self.explanation_style = ExplanationStyle::Educational;
            return Some("Educational mode enabled. I'll explain the 'why' behind things.".to_string());
        }

        None
    }

    /// Get greeting style based on personality
    pub fn greeting_style(&self) -> &'static str {
        match (self.formality, self.friendliness) {
            (FormalityLevel::Formal, _) => "Good day",
            (FormalityLevel::Casual, FriendlinessLevel::Enthusiastic) => "Hey there",
            (FormalityLevel::Casual, _) => "Hi",
            (FormalityLevel::Technical, _) => "Greetings",
            (_, FriendlinessLevel::Enthusiastic) => "Hello there",
            (_, FriendlinessLevel::Friendly) => "Hello",
            _ => "Hello",
        }
    }

    /// Should include explanations?
    pub fn should_explain(&self) -> bool {
        !matches!(self.explanation_style, ExplanationStyle::JustAnswer)
            && !matches!(self.verbosity, VerbosityLevel::Minimal)
    }
}

/// Format personality config
pub fn format_personality(config: &PersonalityConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Personality Configuration ===\n\n");

    output.push_str(&format!("Formality: {}\n", config.formality));
    output.push_str(&format!("Friendliness: {}\n", config.friendliness));
    output.push_str(&format!("Humor: {}\n", config.humor));
    output.push_str(&format!("Verbosity: {}\n", config.verbosity));
    output.push_str(&format!("Explanation Style: {}\n", config.explanation_style));
    output.push_str(&format!("Emojis: {}\n", if config.use_emojis { "Enabled" } else { "Disabled" }));
    output.push_str(&format!("Metaphors: {}\n", if config.use_metaphors { "Enabled" } else { "Disabled" }));

    if let Some(name) = &config.custom_name {
        output.push_str(&format!("Custom Name: {}\n", name));
    }

    output
}

/// Check if query is personality-related
pub fn is_personality_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("personality")
        || lower.contains("be more")
        || lower.contains("be less")
        || lower.contains("be formal")
        || lower.contains("be casual")
        || lower.contains("be friendly")
        || lower.contains("be serious")
        || lower.contains("use emoji")
        || lower.contains("no emoji")
}

/// Fun fact about personality
pub fn personality_fun_fact() -> &'static str {
    "Anna's personality is fully customizable through natural language! Try 'Anna, be more formal' or 'Anna, be friendlier'."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formality_default() {
        let level = FormalityLevel::default();
        assert_eq!(level, FormalityLevel::Professional);
    }

    #[test]
    fn test_friendliness_default() {
        let level = FriendlinessLevel::default();
        assert_eq!(level, FriendlinessLevel::Balanced);
    }

    #[test]
    fn test_personality_config_default() {
        let config = PersonalityConfig::default();
        assert_eq!(config.formality, FormalityLevel::Professional);
        assert!(!config.use_emojis);
    }

    #[test]
    fn test_apply_formal_change() {
        let mut config = PersonalityConfig::new();
        let result = config.apply_change("Anna, be more formal");
        assert!(result.is_some());
        assert_eq!(config.formality, FormalityLevel::Formal);
    }

    #[test]
    fn test_apply_casual_change() {
        let mut config = PersonalityConfig::new();
        let result = config.apply_change("Be casual please");
        assert!(result.is_some());
        assert_eq!(config.formality, FormalityLevel::Casual);
    }

    #[test]
    fn test_apply_friendly_change() {
        let mut config = PersonalityConfig::new();
        let result = config.apply_change("Be friendlier");
        assert!(result.is_some());
        assert_eq!(config.friendliness, FriendlinessLevel::Friendly);
    }

    #[test]
    fn test_emoji_toggle() {
        let mut config = PersonalityConfig::new();
        assert!(!config.use_emojis);
        config.apply_change("use emoji please");
        assert!(config.use_emojis);
    }

    #[test]
    fn test_greeting_style() {
        let formal = PersonalityConfig::new().with_formality(FormalityLevel::Formal);
        assert_eq!(formal.greeting_style(), "Good day");

        let casual = PersonalityConfig::new().with_formality(FormalityLevel::Casual);
        assert_eq!(casual.greeting_style(), "Hi");
    }

    #[test]
    fn test_is_personality_query() {
        assert!(is_personality_query("Change Anna's personality"));
        assert!(is_personality_query("Be more formal"));
        assert!(!is_personality_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = personality_fun_fact();
        assert!(fact.contains("personality") || fact.contains("customizable"));
    }
}
