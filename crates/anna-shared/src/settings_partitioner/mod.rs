// v0.0.678: Settings Partitioner (Phase 254)
// Partition settings into distinct subsets

mod types;
mod config;
mod partition;
mod stats;
mod partitioner;
mod registry;
mod utils;

// Re-export all public types to preserve API
pub use types::{PartitionStrategy, PredicateType};
pub use config::PartitionerConfig;
pub use partition::{Partition, PartitionResult};
pub use stats::PartitionerStats;
pub use partitioner::SettingsPartitioner;
pub use registry::PartitionerRegistry;
pub use utils::{format_partitioner_registry, is_partitioner_query, partitioner_fun_fact};
