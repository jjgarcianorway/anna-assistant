// v0.0.571: Hook Context and Execution
// Context and execution records for hooks

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{HookResult, HookTrigger};

/// Hook execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// Trigger that fired
    pub trigger: HookTrigger,
    /// Category affected (if any)
    pub category: Option<SettingsCategory>,
    /// Field affected (if any)
    pub field: Option<String>,
    /// Old value (if applicable)
    pub old_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HookContext {
    /// Create new context
    pub fn new(trigger: HookTrigger) -> Self {
        Self {
            trigger,
            category: None,
            field: None,
            old_value: None,
            new_value: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Set category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set field
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Set old value
    pub fn with_old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set new value
    pub fn with_new_value(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// Create change context
    pub fn change(category: SettingsCategory, field: &str) -> Self {
        Self::new(HookTrigger::BeforeChange)
            .with_category(category)
            .with_field(field)
    }
}

/// Hook execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecution {
    /// Hook ID
    pub hook_id: u64,
    /// Hook name
    pub hook_name: String,
    /// Context
    pub context: HookContext,
    /// Result
    pub result: HookResult,
    /// Duration in microseconds
    pub duration_us: u64,
    /// Error message if any
    pub error: Option<String>,
}
