// v0.0.571: Settings Hook Definition
// Individual hook configuration and logic

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::context::HookContext;
use super::types::{HookPriority, HookTrigger};

/// A settings hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsHook {
    /// Unique ID
    pub id: u64,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Trigger point
    pub trigger: HookTrigger,
    /// Categories to watch (empty = all)
    pub categories: Vec<SettingsCategory>,
    /// Priority
    pub priority: HookPriority,
    /// Is enabled
    pub enabled: bool,
    /// Is built-in
    pub builtin: bool,
    /// Execution count
    pub exec_count: u32,
}

impl SettingsHook {
    /// Create new hook
    pub fn new(
        id: u64,
        name: impl Into<String>,
        description: impl Into<String>,
        trigger: HookTrigger,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            trigger,
            categories: Vec::new(),
            priority: HookPriority::Normal,
            enabled: true,
            builtin: false,
            exec_count: 0,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: HookPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Watch specific category
    pub fn watch_category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Mark as built-in
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }

    /// Check if hook should fire for context
    pub fn should_fire(&self, context: &HookContext) -> bool {
        if !self.enabled || self.trigger != context.trigger {
            return false;
        }

        // If no categories specified, fire for all
        if self.categories.is_empty() {
            return true;
        }

        // Check if category matches
        if let Some(cat) = &context.category {
            self.categories.contains(cat)
        } else {
            true
        }
    }

    /// Record execution
    pub fn record_exec(&mut self) {
        self.exec_count += 1;
    }
}
