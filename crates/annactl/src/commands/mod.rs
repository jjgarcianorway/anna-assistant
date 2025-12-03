//! Command modules for annactl v0.0.12
//!
//! v0.0.12: Proactive anomaly detection with alert surfacing
//! v0.0.11: Safe auto-update system with channels and rollback
//! v0.0.10: Added uninstall command, updated reset with installer review
//! v7.42.5: Added reset command for factory reset
//! v7.42.4: No truncation - show all data by default
//! v7.42.0: Daemon/CLI Contract Fix - separates daemon from snapshot state
//!
//! Commands:
//! - status: Anna health, diagnostics, and runtime (daemon + snapshot + paths)
//! - reset: Factory reset - delete all data and config (requires root)
//! - uninstall: Complete Anna removal with provenance-aware helper removal
//! - sw: Software overview (packages, commands, services by category)
//! - sw_detail: Software profiles (package, command, service)
//! - hw: Hardware overview (CPU, memory, GPU, storage, network, audio)
//! - hw_detail: Hardware profiles (cpu, memory, gpu, storage, network, audio, power, sensors, camera, firmware, pci)

pub mod status;
pub mod reset;
pub mod uninstall;
pub mod sw;
pub mod sw_detail;
pub mod hw;
pub mod hw_detail;
