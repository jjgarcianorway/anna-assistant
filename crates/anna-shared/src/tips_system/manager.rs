// v0.0.541: Tips System Manager (Phase 117)

use crate::tips_system::types::{Tip, TipCategory, TipPriority};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tips system manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TipsSystem {
    tips: HashMap<String, Tip>,
    pub(crate) show_tips: bool,
    pub(crate) max_daily_tips: u32,
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
