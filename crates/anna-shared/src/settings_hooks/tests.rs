// v0.0.571: Hook Tests
// Unit tests for settings hooks

#![cfg(test)]

use crate::unified_settings::SettingsCategory;
use super::context::HookContext;
use super::hook::SettingsHook;
use super::manager::HookManager;
use super::types::{HookPriority, HookResult, HookTrigger};
use super::utils::{format_hooks, hooks_fun_fact, is_hooks_query};

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
