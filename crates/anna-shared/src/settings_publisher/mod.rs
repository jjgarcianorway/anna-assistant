// v0.0.634: Settings Publisher Module (Phase 210)
// Publisher for settings change events

mod types;
mod config;
mod event;
mod stats;
mod publisher;
mod registry;
mod utils;

// Re-export all public types
pub use types::{PublisherType, PublicationScope};
pub use config::PublisherConfig;
pub use event::PublicationEvent;
pub use stats::PublisherStats;
pub use publisher::Publisher;
pub use registry::SettingsPublisherRegistry;
pub use utils::{format_publisher_registry, is_publisher_query, publisher_fun_fact};
