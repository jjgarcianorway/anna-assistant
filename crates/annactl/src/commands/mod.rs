//! Command modules for annactl v6.0.0
//!
//! v6.0.0: Grounded system intelligence
//! Every command now queries real system sources
//!
//! Commands:
//! - status: System status from real sources (pacman, systemctl, journalctl)
//! - stats: Daemon activity statistics
//! - knowledge: Knowledge overview (packages, commands, services)
//! - knowledge_detail: Full profile from pacman/which/systemctl
//! - version: Installation and version info

pub mod status;
pub mod stats;
pub mod knowledge;
pub mod knowledge_detail;
pub mod version;

// Legacy commands (kept for backward compatibility, will be removed)
pub mod knowledge_stats;
pub mod knowledge_category;
