// v0.0.571: Settings Hooks (Phase 147)
// Callbacks and hooks for settings changes

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Hook trigger point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookTrigger {
    /// Before a change is applied
    BeforeChange,
    /// After a change is applied
    AfterChange,
    /// Before settings are loaded
    BeforeLoad,
    /// After settings are loaded
    AfterLoad,
    /// Before settings are saved
    BeforeSave,
    /// After settings are saved
    AfterSave,
    /// On validation
    OnValidate,
}

impl std::fmt::Display for HookTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeChange => write!(f, "Before Change"),
            Self::AfterChange => write!(f, "After Change"),
            Self::BeforeLoad => write!(f, "Before Load"),
            Self::AfterLoad => write!(f, "After Load"),
            Self::BeforeSave => write!(f, "Before Save"),
            Self::AfterSave => write!(f, "After Save"),
            Self::OnValidate => write!(f, "On Validate"),
        }
    }
}

/// Hook result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookResult {
    /// Continue with operation
    Continue,
    /// Skip the operation
    Skip,
    /// Abort the operation
    Abort,
    /// Modify and continue
    Modify,
}

impl std::fmt::Display for HookResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continue => write!(f, "Continue"),
            Self::Skip => write!(f, "Skip"),
            Self::Abort => write!(f, "Abort"),
            Self::Modify => write!(f, "Modify"),
        }
    }
}

/// Hook priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HookPriority {
    /// Run first
    High = 0,
    /// Run normally
    Normal = 50,
    /// Run last
    Low = 100,
}

impl Default for HookPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for HookPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "High"),
            Self::Normal => write!(f, "Normal"),
            Self::Low => write!(f, "Low"),
        }
    }
}

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

/// Hook manager
#[derive(Debug, Clone, Default)]
pub struct HookManager {
    /// Registered hooks
    hooks: Vec<SettingsHook>,
    /// Next ID
    next_id: u64,
    /// Execution history
    history: Vec<HookExecution>,
    /// Max history size
    max_history: usize,
}

impl HookManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Create with default hooks
    pub fn with_defaults() -> Self {
        let mut mgr = Self::new();
        mgr.add_default_hooks();
        mgr
    }

    /// Add default hooks
    fn add_default_hooks(&mut self) {
        // Log all changes hook
        self.register(
            SettingsHook::new(
                0,
                "Log Changes",
                "Log all settings changes",
                HookTrigger::AfterChange,
            )
            .with_priority(HookPriority::Low)
            .builtin(),
        );

        // Validate on save hook
        self.register(
            SettingsHook::new(
                0,
                "Validate Before Save",
                "Validate settings before saving",
                HookTrigger::BeforeSave,
            )
            .with_priority(HookPriority::High)
            .builtin(),
        );

        // Notify on change hook
        self.register(
            SettingsHook::new(
                0,
                "Notify Changes",
                "Send notifications on settings changes",
                HookTrigger::AfterChange,
            )
            .with_priority(HookPriority::Normal)
            .builtin(),
        );
    }

    /// Register a hook
    pub fn register(&mut self, mut hook: SettingsHook) -> u64 {
        hook.id = self.next_id;
        self.next_id += 1;
        let id = hook.id;
        self.hooks.push(hook);
        id
    }

    /// Unregister a hook
    pub fn unregister(&mut self, id: u64) -> Option<SettingsHook> {
        if let Some(pos) = self.hooks.iter().position(|h| h.id == id && !h.builtin) {
            Some(self.hooks.remove(pos))
        } else {
            None
        }
    }

    /// Get hook by ID
    pub fn get(&self, id: u64) -> Option<&SettingsHook> {
        self.hooks.iter().find(|h| h.id == id)
    }

    /// Enable/disable hook
    pub fn set_enabled(&mut self, id: u64, enabled: bool) -> bool {
        if let Some(h) = self.hooks.iter_mut().find(|h| h.id == id) {
            h.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// List all hooks
    pub fn list(&self) -> &[SettingsHook] {
        &self.hooks
    }

    /// List hooks for a trigger
    pub fn for_trigger(&self, trigger: HookTrigger) -> Vec<&SettingsHook> {
        let mut hooks: Vec<_> = self.hooks
            .iter()
            .filter(|h| h.enabled && h.trigger == trigger)
            .collect();
        hooks.sort_by_key(|h| h.priority);
        hooks
    }

    /// Fire hooks for a context
    pub fn fire(&mut self, context: &HookContext) -> Vec<HookResult> {
        let mut results = Vec::new();
        let hooks_to_fire: Vec<u64> = self.hooks
            .iter()
            .filter(|h| h.should_fire(context))
            .map(|h| h.id)
            .collect();

        for id in hooks_to_fire {
            if let Some(hook) = self.hooks.iter_mut().find(|h| h.id == id) {
                hook.record_exec();

                // Simulate hook execution
                let result = HookResult::Continue;

                // Record execution
                let exec = HookExecution {
                    hook_id: hook.id,
                    hook_name: hook.name.clone(),
                    context: context.clone(),
                    result,
                    duration_us: 0,
                    error: None,
                };
                self.add_to_history(exec);

                results.push(result);
            }
        }

        results
    }

    /// Add execution to history
    fn add_to_history(&mut self, exec: HookExecution) {
        self.history.push(exec);
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get execution history
    pub fn history(&self) -> &[HookExecution] {
        &self.history
    }

    /// Get recent executions
    pub fn recent(&self, count: usize) -> Vec<&HookExecution> {
        self.history.iter().rev().take(count).collect()
    }

    /// Count hooks
    pub fn count(&self) -> usize {
        self.hooks.len()
    }
}

/// Format hooks for display
pub fn format_hooks(manager: &HookManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Hooks ===\n\n");

    if manager.count() == 0 {
        output.push_str("No hooks registered.\n");
        return output;
    }

    for trigger in [
        HookTrigger::BeforeChange,
        HookTrigger::AfterChange,
        HookTrigger::BeforeLoad,
        HookTrigger::AfterLoad,
        HookTrigger::BeforeSave,
        HookTrigger::AfterSave,
        HookTrigger::OnValidate,
    ] {
        let hooks = manager.for_trigger(trigger);
        if !hooks.is_empty() {
            output.push_str(&format!("{}:\n", trigger));
            for h in hooks {
                let status = if h.enabled { "enabled" } else { "disabled" };
                output.push_str(&format!(
                    "  • {} [{}] - {} (executed {} times)\n",
                    h.name, status, h.priority, h.exec_count
                ));
            }
            output.push('\n');
        }
    }

    output
}

/// Check if query is about hooks
pub fn is_hooks_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("hook")
        || lower.contains("callback")
        || lower.contains("on change")
        || lower.contains("trigger")
}

/// Fun fact about hooks
pub fn hooks_fun_fact() -> &'static str {
    "Settings hooks let you run custom actions whenever settings change!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_trigger_display() {
        assert_eq!(format!("{}", HookTrigger::BeforeChange), "Before Change");
        assert_eq!(format!("{}", HookTrigger::AfterSave), "After Save");
    }

    #[test]
    fn test_hook_result_display() {
        assert_eq!(format!("{}", HookResult::Continue), "Continue");
        assert_eq!(format!("{}", HookResult::Abort), "Abort");
    }

    #[test]
    fn test_hook_priority_display() {
        assert_eq!(format!("{}", HookPriority::High), "High");
        assert_eq!(format!("{}", HookPriority::Normal), "Normal");
    }

    #[test]
    fn test_hook_context_new() {
        let ctx = HookContext::new(HookTrigger::BeforeChange);
        assert_eq!(ctx.trigger, HookTrigger::BeforeChange);
        assert!(ctx.category.is_none());
    }

    #[test]
    fn test_hook_context_builder() {
        let ctx = HookContext::new(HookTrigger::AfterChange)
            .with_category(SettingsCategory::Risk)
            .with_field("level");
        assert_eq!(ctx.category, Some(SettingsCategory::Risk));
        assert_eq!(ctx.field, Some("level".to_string()));
    }

    #[test]
    fn test_settings_hook_new() {
        let hook = SettingsHook::new(1, "Test", "Test hook", HookTrigger::AfterChange);
        assert_eq!(hook.id, 1);
        assert!(hook.enabled);
        assert!(!hook.builtin);
    }

    #[test]
    fn test_settings_hook_should_fire() {
        let hook = SettingsHook::new(1, "Test", "Test", HookTrigger::AfterChange);
        let ctx = HookContext::new(HookTrigger::AfterChange);
        assert!(hook.should_fire(&ctx));
    }

    #[test]
    fn test_settings_hook_should_not_fire_wrong_trigger() {
        let hook = SettingsHook::new(1, "Test", "Test", HookTrigger::BeforeChange);
        let ctx = HookContext::new(HookTrigger::AfterChange);
        assert!(!hook.should_fire(&ctx));
    }

    #[test]
    fn test_hook_manager_new() {
        let manager = HookManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_hook_manager_with_defaults() {
        let manager = HookManager::with_defaults();
        assert!(manager.count() >= 3);
    }

    #[test]
    fn test_hook_manager_register() {
        let mut manager = HookManager::new();
        let hook = SettingsHook::new(0, "Test", "Test", HookTrigger::AfterChange);
        let id = manager.register(hook);
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_hook_manager_fire() {
        let mut manager = HookManager::with_defaults();
        let ctx = HookContext::new(HookTrigger::AfterChange);
        let results = manager.fire(&ctx);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_hook_manager_for_trigger() {
        let manager = HookManager::with_defaults();
        let hooks = manager.for_trigger(HookTrigger::AfterChange);
        assert!(!hooks.is_empty());
    }

    #[test]
    fn test_format_hooks() {
        let manager = HookManager::with_defaults();
        let output = format_hooks(&manager);
        assert!(output.contains("Hooks"));
    }

    #[test]
    fn test_is_hooks_query() {
        assert!(is_hooks_query("settings hooks"));
        assert!(is_hooks_query("callback for change"));
        assert!(!is_hooks_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = hooks_fun_fact();
        assert!(fact.contains("hook"));
    }
}
