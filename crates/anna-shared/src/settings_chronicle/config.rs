// v0.0.692: Settings Chronicle Config (Phase 268)
// Chronicle configuration

use serde::{Deserialize, Serialize};

use super::types::ChronicleMode;

/// Chronicle config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleConfig {
    /// Track mode
    pub mode: ChronicleMode,
    /// Enabled
    pub enabled: bool,
    /// Max history
    pub max_history: usize,
    /// Track patterns
    pub patterns: Vec<String>,
}

impl ChronicleConfig {
    /// Create new config
    pub fn new(mode: ChronicleMode) -> Self {
        Self {
            mode,
            enabled: true,
            max_history: 100,
            patterns: Vec::new(),
        }
    }

    /// Set max history
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Add pattern
    pub fn add_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }
}

impl Default for ChronicleConfig {
    fn default() -> Self {
        Self::new(ChronicleMode::All)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ChronicleConfig::new(ChronicleMode::All);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = ChronicleConfig::new(ChronicleMode::Pattern)
            .max_history(50)
            .add_pattern("app.");
        assert_eq!(c.max_history, 50);
        assert_eq!(c.patterns.len(), 1);
    }
}
