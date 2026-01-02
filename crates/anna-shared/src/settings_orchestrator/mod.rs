// v0.0.574: Settings Orchestrator Module
// Unified coordinator for all settings subsystems

mod core;
mod result;
mod state;
mod status;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the API
pub use core::SettingsOrchestrator;
pub use result::OperationResult;
pub use state::OrchestratorState;
pub use status::{format_orchestrator_status, OrchestratorStatus};
pub use utils::{is_orchestrator_query, orchestrator_fun_fact};
