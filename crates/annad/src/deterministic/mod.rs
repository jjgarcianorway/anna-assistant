//! Deterministic answerer - produces answers without LLM when data is available (v0.0.176).
//!
//! Answers common queries by parsing hardware snapshots and probe outputs.
//! Used as primary answerer for known query classes, fallback for LLM timeout.
//!
//! v0.0.122: Extended functions moved to det_extended.rs for modularization.
//! v0.0.176: Modularized into domain-focused submodules.

mod audio;
mod cpu;
mod help;
mod memory;
mod packages;
mod router;
mod types;

// Re-export main types and functions
pub use help::answer_help;
pub use router::try_answer;
pub use types::DeterministicResult;

// Re-export handlers for use by router
pub use audio::answer_hardware_audio;
pub use cpu::{answer_cpu_cores, answer_cpu_temp};
pub use memory::{
    answer_disk_usage, answer_memory_free, answer_memory_usage, answer_service_status,
    answer_system_health_summary,
};
pub use packages::{answer_installed_tool_check, answer_package_count};
