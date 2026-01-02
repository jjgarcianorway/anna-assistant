// v0.0.755: Settings Block (Phase 331)
// City block for settings subdivision

mod types;
mod config;
mod plat;
mod stats;
mod block;
mod registry;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{BlockType, BlockStatus};
pub use config::BlockConfig;
pub use plat::{BlockPlat, BlockSurveyor};
pub use stats::BlockStats;
pub use block::SettingsBlock;
pub use registry::{BlockRegistry, format_block_registry, is_block_query, block_fun_fact};
