// v0.0.636: Settings Listener Module (Phase 212)
// Listener for settings change events

mod types;
mod config;
mod event;
mod stats;
mod listener;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{ListenerType, ListenerState};
pub use config::ListenerConfig;
pub use event::ReceivedEvent;
pub use stats::ListenerStats;
pub use listener::SettingsListener;
pub use registry::SettingsListenerRegistry;
pub use utils::{format_listener_registry, is_listener_query, listener_fun_fact};
