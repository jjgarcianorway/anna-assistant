// v0.0.541: Tips System Types (Phase 117)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Tip category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TipCategory {
    Personality,
    LearningMode,
    RiskLevel,
    Verbosity,
    DisplayMode,
    Notification,
    Performance,
    Keyboard,
    Feature,
    BestPractice,
    Custom(String),
}

impl Default for TipCategory {
    fn default() -> Self {
        Self::Feature
    }
}

impl std::fmt::Display for TipCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Personality => write!(f, "Personality"),
            Self::LearningMode => write!(f, "Learning Mode"),
            Self::RiskLevel => write!(f, "Risk Level"),
            Self::Verbosity => write!(f, "Verbosity"),
            Self::DisplayMode => write!(f, "Display Mode"),
            Self::Notification => write!(f, "Notification"),
            Self::Performance => write!(f, "Performance"),
            Self::Keyboard => write!(f, "Keyboard"),
            Self::Feature => write!(f, "Feature"),
            Self::BestPractice => write!(f, "Best Practice"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Tip priority for selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TipPriority {
    Low,
    #[default]
    Normal,
    High,
    Featured,
}

impl std::fmt::Display for TipPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Featured => write!(f, "Featured"),
        }
    }
}

/// Single tip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tip {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: TipCategory,
    pub priority: TipPriority,
    pub shown_count: u32,
    pub last_shown: Option<DateTime<Utc>>,
    pub enabled: bool,
}

impl Tip {
    /// Create new tip
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            category: TipCategory::default(),
            priority: TipPriority::default(),
            shown_count: 0,
            last_shown: None,
            enabled: true,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: TipCategory) -> Self {
        self.category = category;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: TipPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Mark as shown
    pub fn mark_shown(&mut self) {
        self.shown_count += 1;
        self.last_shown = Some(Utc::now());
    }
}
