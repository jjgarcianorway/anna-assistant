// v0.0.731: Settings Pact (Phase 307)
// Formal pact for settings governance

mod types;
mod structs;
mod pact;
mod registry;
mod utils;

// Re-export types
pub use types::{PactStatus, PactType};

// Re-export structs
pub use structs::{PactClause, PactConfig, PactParty, PactStats};

// Re-export main pact
pub use pact::SettingsPact;

// Re-export registry
pub use registry::{format_pact_registry, PactRegistry};

// Re-export utils
pub use utils::{is_pact_query, pact_fun_fact};
