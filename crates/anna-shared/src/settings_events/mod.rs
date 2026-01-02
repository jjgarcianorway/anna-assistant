// v0.0.581: Settings Events (Phase 157)
// Event system for settings changes with pub/sub pattern

mod types;
mod event;
mod filter;
mod bus;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve API
pub use types::{EventPriority, SettingsEventType};
pub use event::SettingsEvent;
pub use filter::{EventFilter, Subscriber};
pub use bus::SettingsEventBus;
pub use utils::{format_events, is_events_query, settings_events_fun_fact};
