// v0.0.571: Hook Manager
// Central management of settings hooks

use super::context::{HookContext, HookExecution};
use super::hook::SettingsHook;
use super::types::{HookPriority, HookResult, HookTrigger};

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
