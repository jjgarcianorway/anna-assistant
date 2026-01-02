// v0.0.571: Hook Utilities
// Utility functions for hooks

use super::manager::HookManager;
use super::types::HookTrigger;

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
