// v0.0.728: Settings Protocol (Phase 304)
// Formal protocol for settings governance

mod types;
mod config;
mod data;
mod protocol;
mod registry;
mod helpers;
#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain API compatibility
pub use types::{ProtocolType, ProtocolStatus};
pub use config::ProtocolConfig;
pub use data::{ProtocolClause, ProtocolParty, ProtocolStats};
pub use protocol::SettingsProtocol;
pub use registry::ProtocolRegistry;
pub use helpers::{format_protocol_registry, is_protocol_query, protocol_fun_fact};
