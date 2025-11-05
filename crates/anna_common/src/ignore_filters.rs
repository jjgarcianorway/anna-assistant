//! Ignore filters - User can hide categories or priorities
//!
//! Users can ignore:
//! - Entire categories (e.g., "Desktop Customization")
//! - Priority levels (e.g., all "Cosmetic" items)
//! - Individual advice items (already handled by UserFeedbackLog)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::{Priority, Advice};

/// User's ignore preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IgnoreFilters {
    /// Categories to ignore (exact match)
    pub ignored_categories: Vec<String>,

    /// Priority levels to ignore
    pub ignored_priorities: Vec<Priority>,

    /// When filters were last updated
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl IgnoreFilters {
    /// Get config file path
    fn config_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config")
            })
            .join("anna");

        config_dir.join("ignore_filters.json")
    }

    /// Load ignore filters
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let json = std::fs::read_to_string(&path)
            .context("Failed to read ignore filters")?;

        let filters: Self = serde_json::from_str(&json)
            .context("Failed to parse ignore filters")?;

        Ok(filters)
    }

    /// Save ignore filters
    pub fn save(&mut self) -> Result<()> {
        let path = Self::config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        self.updated_at = Some(chrono::Utc::now());

        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&path, json).context("Failed to write ignore filters")?;

        Ok(())
    }

    /// Ignore a category
    pub fn ignore_category(&mut self, category: &str) {
        if !self.ignored_categories.contains(&category.to_string()) {
            self.ignored_categories.push(category.to_string());
        }
    }

    /// Un-ignore a category
    pub fn unignore_category(&mut self, category: &str) {
        self.ignored_categories.retain(|c| c != category);
    }

    /// Ignore a priority level
    pub fn ignore_priority(&mut self, priority: Priority) {
        if !self.ignored_priorities.contains(&priority) {
            self.ignored_priorities.push(priority);
        }
    }

    /// Un-ignore a priority level
    pub fn unignore_priority(&mut self, priority: Priority) {
        self.ignored_priorities.retain(|p| p != &priority);
    }

    /// Check if a category is ignored
    pub fn is_category_ignored(&self, category: &str) -> bool {
        self.ignored_categories.contains(&category.to_string())
    }

    /// Check if a priority is ignored
    pub fn is_priority_ignored(&self, priority: &Priority) -> bool {
        self.ignored_priorities.contains(priority)
    }

    /// Check if an advice item should be filtered out
    pub fn should_filter(&self, advice: &Advice) -> bool {
        self.is_category_ignored(&advice.category) ||
        self.is_priority_ignored(&advice.priority)
    }

    /// Filter a list of advice based on ignore settings
    pub fn filter_advice(&self, advice_list: Vec<Advice>) -> Vec<Advice> {
        advice_list.into_iter()
            .filter(|a| !self.should_filter(a))
            .collect()
    }

    /// Get list of all ignored items (for display)
    pub fn get_ignored_summary(&self) -> String {
        let mut summary = Vec::new();

        if !self.ignored_categories.is_empty() {
            summary.push(format!("📁 Ignored Categories ({}): {}",
                self.ignored_categories.len(),
                self.ignored_categories.join(", ")));
        }

        if !self.ignored_priorities.is_empty() {
            let priority_names: Vec<String> = self.ignored_priorities.iter()
                .map(|p| match p {
                    Priority::Mandatory => "Mandatory".to_string(),
                    Priority::Recommended => "Recommended".to_string(),
                    Priority::Optional => "Optional".to_string(),
                    Priority::Cosmetic => "Cosmetic".to_string(),
                })
                .collect();

            summary.push(format!("🎯 Ignored Priorities ({}): {}",
                self.ignored_priorities.len(),
                priority_names.join(", ")));
        }

        if summary.is_empty() {
            "No filters active - seeing all recommendations".to_string()
        } else {
            summary.join("\n")
        }
    }

    /// Reset all filters
    pub fn reset_all(&mut self) {
        self.ignored_categories.clear();
        self.ignored_priorities.clear();
        self.updated_at = Some(chrono::Utc::now());
    }
}
