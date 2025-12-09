//! Model selector with Qwen3-VL preference (v0.0.223).
//!
//! Selects best model from available inventory, prefers Qwen3-VL family.
//!
//! v0.0.74: Initial implementation.
//! v0.0.223: Modularized into domain-focused submodules.

mod benchmark;
mod catalog;
mod config;
mod selection;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use benchmark::{parse_benchmark_response, BENCHMARK_EXPECTED_TOKENS, BENCHMARK_PROMPT};
pub use catalog::model_catalog;
pub use config::{ModelSelectorConfig, ModelSelectorState};
pub use selection::{detect_family, model_matches, select_model};
pub use types::{ModelBenchmark, ModelCandidate, ModelFamily, ModelRole, ModelSelection};
