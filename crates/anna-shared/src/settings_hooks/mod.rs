// v0.0.571: Settings Hooks Module
// Callbacks and hooks for settings changes

mod types;
mod context;
mod hook;
mod manager;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{HookPriority, HookResult, HookTrigger};
pub use context::{HookContext, HookExecution};
pub use hook::SettingsHook;
pub use manager::HookManager;
pub use utils::{format_hooks, hooks_fun_fact, is_hooks_query};
