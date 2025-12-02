//! Command modules for annactl v7.42.1
//!
//! v7.42.1: Status now includes inline diagnostics (no separate doctor command)
//! v7.42.0: Daemon/CLI Contract Fix - separates daemon from snapshot state
//!
//! Commands:
//! - status: Anna health, diagnostics, and runtime (daemon + snapshot + paths)
//! - sw: Software overview (packages, commands, services by category)
//! - sw_detail: Software profiles (package, command, service)
//! - hw: Hardware overview (CPU, memory, GPU, storage, network, audio)
//! - hw_detail: Hardware profiles (cpu, memory, gpu, storage, network, audio, power, sensors, camera, firmware, pci)

pub mod status;
pub mod sw;
pub mod sw_detail;
pub mod hw;
pub mod hw_detail;
