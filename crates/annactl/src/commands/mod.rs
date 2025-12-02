//! Command modules for annactl v7.29.0
//!
//! v7.29.0: Bugfix & Performance Release
//! - No ellipsis truncation
//! - hw categories: sensors, camera, firmware, pci
//! - Telemetry window validation (60% sample coverage)
//! - Process identity from /proc (not "Bun Pool N")
//!
//! Commands:
//! - status: Anna-only health (daemon, inventory, updates, paths, internal errors)
//! - sw: Software overview (packages, commands, services by category)
//! - sw_detail: Software profiles (package, command, service)
//! - hw: Hardware overview (CPU, memory, GPU, storage, network, audio)
//! - hw_detail: Hardware profiles (cpu, memory, gpu, storage, network, audio, power, sensors, camera, firmware, pci)

pub mod status;
pub mod sw;
pub mod sw_detail;
pub mod hw;
pub mod hw_detail;
