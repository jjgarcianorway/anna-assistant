// v0.0.787: Settings Enclave (Phase 363)
// Exclusive enclave for settings community

mod types;
mod config;
mod member;
mod steward;
mod stats;
mod enclave;
mod registry;
mod utils;

// Re-export types
pub use types::{EnclaveType, EnclaveStatus};
pub use config::EnclaveConfig;
pub use member::EnclaveMember;
pub use steward::EnclaveSteward;
pub use stats::EnclaveStats;
pub use enclave::SettingsEnclave;
pub use registry::EnclaveRegistry;
pub use utils::{format_enclave_registry, is_enclave_query, enclave_fun_fact};
