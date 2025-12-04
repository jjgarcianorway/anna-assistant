//! Grounded Knowledge System v7.34.0
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
//! v7.19.0: Service topology, signal quality, topology hints.
//! v7.25.0: Peripherals - USB, Bluetooth, Thunderbolt, SD, cameras, audio.
//! v7.32.0: Evidence-based categorization, Steam/platform game detection.

pub mod arch_wiki;
pub mod categoriser;
pub mod category;
pub mod commands;
pub mod config;
pub mod config_graph;
pub mod deps;
pub mod drivers;
pub mod errors;
pub mod health;
pub mod log_patterns;
pub mod network;
pub mod network_topology;
pub mod packages;
pub mod peripherals;
pub mod service_topology;
pub mod services;
pub mod signal_quality;
pub mod storage_topology;
// v7.32.0: Evidence-based categorization and game platform detection
pub mod category_evidence;
pub mod game_platforms;
pub mod steam;

pub use arch_wiki::*;
pub use categoriser::*;
pub use category::*;
pub use commands::*;
pub use config::*;
pub use config_graph::*;
pub use deps::*;
pub use drivers::*;
pub use errors::*;
pub use health::*;
pub use log_patterns::*;
pub use network::*;
pub use network_topology::*;
pub use packages::*;
pub use peripherals::*;
pub use service_topology::*;
pub use services::*;
pub use signal_quality::*;
pub use storage_topology::*;
// v7.32.0: Evidence-based categorization and game platform detection
pub use category_evidence::*;
pub use game_platforms::*;
pub use steam::*;
