//! Command modules for annactl v7.2.0
//!
//! v7.2.0: sw/hw surface
//!
//! Commands:
//! - status: Anna-only health (daemon, inventory, updates, paths, internal errors)
//! - sw: Software overview (packages, commands, services by category)
//! - sw_detail: Software profiles (package, command, service)
//! - hw: Hardware overview (CPU, memory, GPU, storage, network, audio)
//! - hw_detail: Hardware profiles (cpu, memory, gpu, storage, network, audio, power)
//!
//! Note: The deprecated kdb command (v7.1.x) now routes to sw internally.
//! The old kdb.rs and kdb_detail.rs files have been removed.

pub mod status;
pub mod sw;
pub mod sw_detail;
pub mod hw;
pub mod hw_detail;
