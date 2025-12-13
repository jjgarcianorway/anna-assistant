// v0.0.541: Tips System (Phase 117)
// Tips for greetings about config options per VISION.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Tips system manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TipsSystem {
    tips: HashMap<String, Tip>,
    show_tips: bool,
    max_daily_tips: u32,
    tips_shown_today: u32,
    last_tip_date: Option<DateTime<Utc>>,
}

impl Default for TipsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TipsSystem {
    /// Create new tips system
    pub fn new() -> Self {
        let mut system = Self {
            tips: HashMap::new(),
            show_tips: true,
            max_daily_tips: 3,
            tips_shown_today: 0,
            last_tip_date: None,
        };
        system.load_default_tips();
        system
    }

    /// Load default tips
    fn load_default_tips(&mut self) {
        let default_tips = vec![
            Tip::new("personality", "Customize Anna's Personality",
                "You can change Anna's personality traits through natural language. Try 'Anna, be more formal' or 'Anna, be friendlier'.")
                .with_category(TipCategory::Personality)
                .with_priority(TipPriority::High),
            Tip::new("learning", "Learning Mode",
                "Enable learning mode to see explanations of why Anna runs commands. Say 'Anna, enable learning mode'.")
                .with_category(TipCategory::LearningMode)
                .with_priority(TipPriority::High),
            Tip::new("risk", "Risk Level Configuration",
                "Adjust confirmation requirements with risk levels. 'Anna, skip confirmations for low-risk tasks'.")
                .with_category(TipCategory::RiskLevel)
                .with_priority(TipPriority::Normal),
            Tip::new("verbosity", "Verbosity Control",
                "Control how much detail Anna provides. Try 'Anna, be more concise' or 'Anna, give detailed explanations'.")
                .with_category(TipCategory::Verbosity)
                .with_priority(TipPriority::Normal),
            Tip::new("debug", "Debug Mode",
                "Toggle debug mode to see technical details. 'Anna, enable debug mode' shows JSON and internal workings.")
                .with_category(TipCategory::DisplayMode)
                .with_priority(TipPriority::Normal),
            Tip::new("notifications", "Notification Settings",
                "Configure how Anna notifies you. 'Anna, email me when long tasks complete'.")
                .with_category(TipCategory::Notification)
                .with_priority(TipPriority::Normal),
            Tip::new("stats", "View Statistics",
                "Check Anna's performance with 'annactl stats'. See XP, tickets closed, and fun statistics!")
                .with_category(TipCategory::Feature)
                .with_priority(TipPriority::Low),
            Tip::new("citations", "Citations Matter",
                "Anna always cites sources from Arch Wiki, man pages, and --help. Trust but verify!")
                .with_category(TipCategory::BestPractice)
                .with_priority(TipPriority::Low),
            Tip::new("idle", "Idle Time Research",
                "Anna can research complex questions during idle time and email you the results.")
                .with_category(TipCategory::Feature)
                .with_priority(TipPriority::Normal),
            Tip::new("alarms", "Custom Alarms",
                "Set monitoring alarms: 'Anna, notify me every Monday at 9 about storage progression'.")
                .with_category(TipCategory::Notification)
                .with_priority(TipPriority::Normal),
        ];

        for tip in default_tips {
            self.tips.insert(tip.id.clone(), tip);
        }
    }

    /// Add custom tip
    pub fn add_tip(&mut self, tip: Tip) {
        self.tips.insert(tip.id.clone(), tip);
    }

    /// Get tip by ID
    pub fn get(&self, id: &str) -> Option<&Tip> {
        self.tips.get(id)
    }

    /// Get next tip to show (rotates through tips)
    pub fn next_tip(&mut self) -> Option<&Tip> {
        if !self.show_tips {
            return None;
        }

        // Reset daily counter if new day
        self.check_daily_reset();

        if self.tips_shown_today >= self.max_daily_tips {
            return None;
        }

        // Find tip with lowest show count that's enabled
        let tip_id = self.tips.values()
            .filter(|t| t.enabled)
            .min_by_key(|t| t.shown_count)
            .map(|t| t.id.clone())?;

        if let Some(tip) = self.tips.get_mut(&tip_id) {
            tip.mark_shown();
            self.tips_shown_today += 1;
            self.last_tip_date = Some(Utc::now());
        }

        self.tips.get(&tip_id)
    }

    /// Get random tip from category
    pub fn tip_from_category(&mut self, category: &TipCategory) -> Option<&Tip> {
        let tip_id = self.tips.values()
            .filter(|t| t.enabled && &t.category == category)
            .min_by_key(|t| t.shown_count)
            .map(|t| t.id.clone())?;

        if let Some(tip) = self.tips.get_mut(&tip_id) {
            tip.mark_shown();
        }

        self.tips.get(&tip_id)
    }

    /// Check and reset daily counter
    fn check_daily_reset(&mut self) {
        if let Some(last) = self.last_tip_date {
            let now = Utc::now();
            if now.date_naive() != last.date_naive() {
                self.tips_shown_today = 0;
            }
        }
    }

    /// Enable/disable tips
    pub fn set_enabled(&mut self, enabled: bool) {
        self.show_tips = enabled;
    }

    /// Set max daily tips
    pub fn set_max_daily(&mut self, max: u32) {
        self.max_daily_tips = max;
    }

    /// Get tips by category
    pub fn by_category(&self, category: &TipCategory) -> Vec<&Tip> {
        self.tips.values().filter(|t| &t.category == category).collect()
    }

    /// Get featured tips
    pub fn featured(&self) -> Vec<&Tip> {
        self.tips.values()
            .filter(|t| t.priority == TipPriority::Featured && t.enabled)
            .collect()
    }

    /// Total tip count
    pub fn total(&self) -> usize {
        self.tips.len()
    }

    /// Category stats
    pub fn category_stats(&self) -> HashMap<TipCategory, u32> {
        let mut counts = HashMap::new();
        for tip in self.tips.values() {
            *counts.entry(tip.category.clone()).or_default() += 1;
        }
        counts
    }

    /// Tips remaining today
    pub fn remaining_today(&self) -> u32 {
        self.max_daily_tips.saturating_sub(self.tips_shown_today)
    }
}

/// Format tip for display
pub fn format_tip(tip: &Tip) -> String {
    format!("Tip: {}\n{}", tip.title, tip.content)
}

/// Format tip compact (for greeting)
pub fn format_tip_compact(tip: &Tip) -> String {
    format!("Tip: {}", tip.content)
}

/// Format tips summary
pub fn format_tips_summary(system: &TipsSystem) -> String {
    let mut output = String::new();
    output.push_str("=== Tips System ===\n\n");

    output.push_str(&format!("Total Tips: {}\n", system.total()));
    output.push_str(&format!("Tips Enabled: {}\n", system.show_tips));
    output.push_str(&format!("Max Daily: {}\n", system.max_daily_tips));
    output.push_str(&format!("Remaining Today: {}\n", system.remaining_today()));

    output.push_str("\nBy Category:\n");
    for (cat, count) in system.category_stats() {
        output.push_str(&format!("  {}: {}\n", cat, count));
    }

    output
}

/// Check if query is tips-related
pub fn is_tips_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("tip")
        || lower.contains("hint")
        || lower.contains("suggestion")
        || lower.contains("did you know")
        || lower.contains("configure anna")
}

/// Fun fact about tips
pub fn tips_fun_fact() -> &'static str {
    "Anna's tips system helps you discover configuration options you might not know about - like personality traits, learning modes, and risk levels!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tip_category_default() {
        let cat = TipCategory::default();
        assert_eq!(cat, TipCategory::Feature);
    }

    #[test]
    fn test_tip_priority_default() {
        let priority = TipPriority::default();
        assert_eq!(priority, TipPriority::Normal);
    }

    #[test]
    fn test_tips_system_creation() {
        let system = TipsSystem::new();
        assert!(system.total() > 0);
    }

    #[test]
    fn test_add_tip() {
        let mut system = TipsSystem::new();
        let initial = system.total();
        system.add_tip(Tip::new("test", "Test Tip", "Test content"));
        assert_eq!(system.total(), initial + 1);
    }

    #[test]
    fn test_next_tip() {
        let mut system = TipsSystem::new();
        let tip = system.next_tip();
        assert!(tip.is_some());
    }

    #[test]
    fn test_mark_shown() {
        let mut tip = Tip::new("test", "Test", "Content");
        assert_eq!(tip.shown_count, 0);
        tip.mark_shown();
        assert_eq!(tip.shown_count, 1);
        assert!(tip.last_shown.is_some());
    }

    #[test]
    fn test_category_stats() {
        let system = TipsSystem::new();
        let stats = system.category_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_remaining_today() {
        let system = TipsSystem::new();
        assert_eq!(system.remaining_today(), system.max_daily_tips);
    }

    #[test]
    fn test_is_tips_query() {
        assert!(is_tips_query("Show me a tip"));
        assert!(is_tips_query("Any hints?"));
        assert!(!is_tips_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = tips_fun_fact();
        assert!(fact.contains("tip") || fact.contains("configuration"));
    }
}
