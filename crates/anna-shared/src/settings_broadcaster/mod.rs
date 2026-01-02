// v0.0.635: Settings Broadcaster Module (Phase 211)
// Broadcaster for settings changes to multiple listeners

mod types;
mod config;
mod message;
mod listener;
mod stats;
mod broadcaster;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain the original API
pub use types::{BroadcastChannel, BroadcastMode};
pub use config::BroadcasterConfig;
pub use message::BroadcastMessage;
pub use listener::ListenerInfo;
pub use stats::BroadcasterStats;
pub use broadcaster::SettingsBroadcaster;
pub use utils::{format_broadcaster, is_broadcaster_query, broadcaster_fun_fact};
