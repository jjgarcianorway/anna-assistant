// v0.0.639: Settings Notifier (Phase 215)
// Notifier for settings change alerts

mod channel;
mod priority;
mod config;
mod notification;
mod stats;
mod notifier;
mod registry;

#[cfg(test)]
mod tests;

// Re-export public API
pub use channel::NotifyChannel;
pub use priority::NotifyPriority;
pub use config::NotifierConfig;
pub use notification::Notification;
pub use stats::NotifierStats;
pub use notifier::SettingsNotifier;
pub use registry::{
    SettingsNotifierRegistry,
    format_notifier_registry,
    is_notifier_query,
    notifier_fun_fact,
};
