//! Grounded Knowledge System v7.17.0
//!
//! Every piece of data has a verifiable source.
//! No invented numbers. No hallucinated descriptions.
//! No guessing config paths - only from pacman/man/Arch Wiki.
//! Rule-based categories from descriptions and metadata.
//! Driver and firmware status from /sys and kernel logs.
//! Hardware health from sensors, SMART, kernel logs.
//! v7.13.0: Dependency graph and network awareness.
//! v7.14.0: Log pattern extraction and config sanity checks.
//! v7.17.0: Network topology, storage mapping, config graph.

pub mod packages;
pub mod commands;
pub mod services;
pub mod errors;
pub mod config;
pub mod category;
pub mod arch_wiki;
pub mod categoriser;
pub mod drivers;
pub mod health;
pub mod deps;
pub mod network;
pub mod log_patterns;
pub mod network_topology;
pub mod storage_topology;
pub mod config_graph;

pub use packages::*;
pub use commands::*;
pub use services::*;
pub use errors::*;
pub use config::*;
pub use category::*;
pub use arch_wiki::*;
pub use categoriser::*;
pub use drivers::*;
pub use health::*;
pub use deps::*;
pub use network::*;
pub use log_patterns::*;
pub use network_topology::*;
pub use storage_topology::*;
pub use config_graph::*;
