//! Query Pattern Analyzer Types
//!
//! Type definitions for query pattern analysis.

use serde::{Deserialize, Serialize};

/// Query pattern category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PatternCategory {
    #[default]
    Question,
    Command,
    Status,
    Config,
    Troubleshoot,
    Install,
    Other,
}

impl PatternCategory {
    pub fn name(&self) -> &'static str {
        match self {
            PatternCategory::Question => "Question",
            PatternCategory::Command => "Command",
            PatternCategory::Status => "Status",
            PatternCategory::Config => "Config",
            PatternCategory::Troubleshoot => "Troubleshoot",
            PatternCategory::Install => "Install",
            PatternCategory::Other => "Other",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            PatternCategory::Question => "?",
            PatternCategory::Command => "!",
            PatternCategory::Status => "○",
            PatternCategory::Config => "⚙",
            PatternCategory::Troubleshoot => "⚡",
            PatternCategory::Install => "+",
            PatternCategory::Other => "·",
        }
    }
}

/// Pattern confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConfidenceLevel {
    #[default]
    Low,
    Medium,
    High,
    Certain,
}

impl ConfidenceLevel {
    pub fn name(&self) -> &'static str {
        match self {
            ConfidenceLevel::Low => "Low",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::High => "High",
            ConfidenceLevel::Certain => "Certain",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            ConfidenceLevel::Low => 25,
            ConfidenceLevel::Medium => 50,
            ConfidenceLevel::High => 75,
            ConfidenceLevel::Certain => 100,
        }
    }
}

/// A query pattern record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPattern {
    /// Pattern template (e.g., "how to {action} {target}")
    pub template: String,
    /// Category
    pub category: PatternCategory,
    /// Confidence level
    pub confidence: ConfidenceLevel,
    /// Number of matches
    pub match_count: u64,
    /// Success rate (0-100)
    pub success_rate: u8,
    /// Example queries that matched
    pub examples: Vec<String>,
    /// Last matched timestamp
    pub last_match: u64,
}
