// v0.0.559: Settings CLI Interface - Parse Result (Phase 135)
// Result type for command parsing with confidence scoring

use serde::{Deserialize, Serialize};

use super::command::SettingsCommand;

/// Command parse result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Parsed command
    pub command: SettingsCommand,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Alternative interpretations
    pub alternatives: Vec<SettingsCommand>,
}

impl ParseResult {
    /// Create a new parse result
    pub fn new(command: SettingsCommand, confidence: f32) -> Self {
        Self {
            command,
            confidence,
            alternatives: vec![],
        }
    }

    /// Add an alternative interpretation
    pub fn with_alternative(mut self, alt: SettingsCommand) -> Self {
        self.alternatives.push(alt);
        self
    }

    /// Is this a confident match?
    pub fn is_confident(&self) -> bool {
        self.confidence >= 0.7
    }
}
