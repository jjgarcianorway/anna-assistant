//! Command modules for annactl v7.42.0
//!
//! v7.42.0: Daemon/CLI Contract Fix
//! - doctor: Diagnostic command for troubleshooting daemon/snapshot issues
//! - status: Now separates daemon state (socket/systemd) from snapshot state (file)
//!
//! Commands:
//! - status: Anna-only health (daemon, inventory, updates, paths, internal errors)
//! - doctor: Diagnostic tool for daemon/snapshot troubleshooting
//! - sw: Software overview (packages, commands, services by category)
//! - sw_detail: Software profiles (package, command, service)
//! - hw: Hardware overview (CPU, memory, GPU, storage, network, audio)
//! - hw_detail: Hardware profiles (cpu, memory, gpu, storage, network, audio, power, sensors, camera, firmware, pci)

pub mod status;
pub mod doctor;
pub mod sw;
pub mod sw_detail;
pub mod hw;
pub mod hw_detail;
