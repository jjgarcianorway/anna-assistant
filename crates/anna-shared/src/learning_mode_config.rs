// v0.0.544: Learning Mode Config (Phase 120)
// Learning mode settings per VISION.md - explain why/how commands work

use serde::{Deserialize, Serialize};

/// Learning mode level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LearningModeLevel {
    #[default]
    Off,
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

impl std::fmt::Display for LearningModeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Basic => write!(f, "Basic"),
            Self::Intermediate => write!(f, "Intermediate"),
            Self::Advanced => write!(f, "Advanced"),
            Self::Expert => write!(f, "Expert"),
        }
    }
}

/// Explanation depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ExplanationDepth {
    None,
    #[default]
    WhatItDoes,
    WhyItWorks,
    HowItWorks,
    DeepDive,
}

impl std::fmt::Display for ExplanationDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::WhatItDoes => write!(f, "What It Does"),
            Self::WhyItWorks => write!(f, "Why It Works"),
            Self::HowItWorks => write!(f, "How It Works"),
            Self::DeepDive => write!(f, "Deep Dive"),
        }
    }
}

/// Topics to explain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExplanationTopic {
    Commands,
    ConfigFiles,
    SystemConcepts,
    NetworkConcepts,
    Security,
    BestPractices,
    Troubleshooting,
    All,
}

impl std::fmt::Display for ExplanationTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Commands => write!(f, "Commands"),
            Self::ConfigFiles => write!(f, "Config Files"),
            Self::SystemConcepts => write!(f, "System Concepts"),
            Self::NetworkConcepts => write!(f, "Network Concepts"),
            Self::Security => write!(f, "Security"),
            Self::BestPractices => write!(f, "Best Practices"),
            Self::Troubleshooting => write!(f, "Troubleshooting"),
            Self::All => write!(f, "All Topics"),
        }
    }
}

/// Learning mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningModeConfig {
    pub level: LearningModeLevel,
    pub explanation_depth: ExplanationDepth,
    pub explain_commands: bool,
    pub explain_config_changes: bool,
    pub explain_why: bool,
    pub show_man_references: bool,
    pub show_wiki_links: bool,
    pub interactive_quiz: bool,
}

impl Default for LearningModeConfig {
    fn default() -> Self {
        Self {
            level: LearningModeLevel::Off,
            explanation_depth: ExplanationDepth::WhatItDoes,
            explain_commands: false,
            explain_config_changes: false,
            explain_why: false,
            show_man_references: false,
            show_wiki_links: false,
            interactive_quiz: false,
        }
    }
}

impl LearningModeConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create enabled config at basic level
    pub fn basic() -> Self {
        Self {
            level: LearningModeLevel::Basic,
            explanation_depth: ExplanationDepth::WhatItDoes,
            explain_commands: true,
            explain_config_changes: true,
            explain_why: false,
            show_man_references: false,
            show_wiki_links: false,
            interactive_quiz: false,
        }
    }

    /// Create enabled config at intermediate level
    pub fn intermediate() -> Self {
        Self {
            level: LearningModeLevel::Intermediate,
            explanation_depth: ExplanationDepth::WhyItWorks,
            explain_commands: true,
            explain_config_changes: true,
            explain_why: true,
            show_man_references: true,
            show_wiki_links: true,
            interactive_quiz: false,
        }
    }

    /// Create enabled config at advanced level
    pub fn advanced() -> Self {
        Self {
            level: LearningModeLevel::Advanced,
            explanation_depth: ExplanationDepth::HowItWorks,
            explain_commands: true,
            explain_config_changes: true,
            explain_why: true,
            show_man_references: true,
            show_wiki_links: true,
            interactive_quiz: true,
        }
    }

    /// Is learning mode enabled?
    pub fn is_enabled(&self) -> bool {
        !matches!(self.level, LearningModeLevel::Off)
    }

    /// Enable learning mode
    pub fn enable(&mut self) {
        if self.level == LearningModeLevel::Off {
            self.level = LearningModeLevel::Basic;
            self.explain_commands = true;
            self.explain_config_changes = true;
        }
    }

    /// Disable learning mode
    pub fn disable(&mut self) {
        self.level = LearningModeLevel::Off;
        self.explain_commands = false;
        self.explain_config_changes = false;
        self.explain_why = false;
    }

    /// Should explain this command?
    pub fn should_explain_command(&self) -> bool {
        self.is_enabled() && self.explain_commands
    }

    /// Should explain config changes?
    pub fn should_explain_config(&self) -> bool {
        self.is_enabled() && self.explain_config_changes
    }

    /// Should explain why?
    pub fn should_explain_why(&self) -> bool {
        self.is_enabled() && self.explain_why
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Enable/disable
        if lower.contains("enable learning") || lower.contains("learning mode on") || lower.contains("teach me") {
            self.enable();
            return Some("Learning mode enabled! I'll explain what commands do.".to_string());
        }
        if lower.contains("disable learning") || lower.contains("learning mode off") || lower.contains("stop teaching") {
            self.disable();
            return Some("Learning mode disabled.".to_string());
        }

        // Level changes
        if lower.contains("basic learning") || lower.contains("simple explanations") {
            *self = Self::basic();
            return Some("Basic learning mode - I'll explain what commands do.".to_string());
        }
        if lower.contains("intermediate learning") || lower.contains("more explanation") {
            *self = Self::intermediate();
            return Some("Intermediate learning - I'll explain why things work.".to_string());
        }
        if lower.contains("advanced learning") || lower.contains("full explanation") || lower.contains("deep learning") {
            *self = Self::advanced();
            return Some("Advanced learning - Full explanations with references.".to_string());
        }

        // Specific toggles
        if lower.contains("explain why") || lower.contains("tell me why") {
            self.explain_why = true;
            return Some("I'll explain why commands are used.".to_string());
        }
        if lower.contains("show references") || lower.contains("show man pages") {
            self.show_man_references = true;
            return Some("I'll include man page references.".to_string());
        }
        if lower.contains("show wiki") || lower.contains("arch wiki links") {
            self.show_wiki_links = true;
            return Some("I'll include Arch Wiki links.".to_string());
        }

        None
    }

    /// Get explanation prefix based on depth
    pub fn explanation_prefix(&self) -> &'static str {
        match self.explanation_depth {
            ExplanationDepth::None => "",
            ExplanationDepth::WhatItDoes => "This command",
            ExplanationDepth::WhyItWorks => "We use this because",
            ExplanationDepth::HowItWorks => "Here's how this works:",
            ExplanationDepth::DeepDive => "Let me explain in detail:",
        }
    }
}

/// Format learning config
pub fn format_learning_config(config: &LearningModeConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Learning Mode Configuration ===\n\n");

    output.push_str(&format!("Level: {}\n", config.level));
    output.push_str(&format!("Enabled: {}\n", config.is_enabled()));
    output.push_str(&format!("Explanation Depth: {}\n", config.explanation_depth));
    output.push_str(&format!("Explain Commands: {}\n", config.explain_commands));
    output.push_str(&format!("Explain Config Changes: {}\n", config.explain_config_changes));
    output.push_str(&format!("Explain Why: {}\n", config.explain_why));
    output.push_str(&format!("Show Man References: {}\n", config.show_man_references));
    output.push_str(&format!("Show Wiki Links: {}\n", config.show_wiki_links));

    output
}

/// Check if query is learning-related
pub fn is_learning_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("learning mode")
        || lower.contains("teach me")
        || lower.contains("explain")
        || lower.contains("how does")
        || lower.contains("why does")
        || lower.contains("what does")
}

/// Fun fact about learning mode
pub fn learning_mode_fun_fact() -> &'static str {
    "Learning mode helps you understand Linux better! When enabled, Anna explains what commands do and why she chose them."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_default() {
        let level = LearningModeLevel::default();
        assert_eq!(level, LearningModeLevel::Off);
    }

    #[test]
    fn test_config_default() {
        let config = LearningModeConfig::default();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let mut config = LearningModeConfig::new();
        assert!(!config.is_enabled());
        config.enable();
        assert!(config.is_enabled());
        config.disable();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_basic_preset() {
        let config = LearningModeConfig::basic();
        assert_eq!(config.level, LearningModeLevel::Basic);
        assert!(config.explain_commands);
    }

    #[test]
    fn test_advanced_preset() {
        let config = LearningModeConfig::advanced();
        assert_eq!(config.level, LearningModeLevel::Advanced);
        assert!(config.explain_why);
        assert!(config.show_man_references);
    }

    #[test]
    fn test_apply_enable() {
        let mut config = LearningModeConfig::new();
        let result = config.apply_change("enable learning mode");
        assert!(result.is_some());
        assert!(config.is_enabled());
    }

    #[test]
    fn test_apply_advanced() {
        let mut config = LearningModeConfig::new();
        config.apply_change("advanced learning please");
        assert_eq!(config.level, LearningModeLevel::Advanced);
    }

    #[test]
    fn test_should_explain() {
        let config = LearningModeConfig::basic();
        assert!(config.should_explain_command());
        assert!(config.should_explain_config());
        assert!(!config.should_explain_why());
    }

    #[test]
    fn test_is_learning_query() {
        assert!(is_learning_query("Enable learning mode"));
        assert!(is_learning_query("Teach me about this"));
        assert!(!is_learning_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = learning_mode_fun_fact();
        assert!(fact.contains("Learning") || fact.contains("explain"));
    }
}
