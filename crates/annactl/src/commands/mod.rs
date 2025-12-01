//! Command modules for annactl v7.0.0
//!
//! v7.0.0: Minimal surface - only 4 commands
//!
//! Commands:
//! - status: Anna-only health (daemon, inventory, updates, paths, internal errors)
//! - kdb: Knowledge database overview (packages, commands, services by category)
//! - kdb_detail: Object profiles and category overviews

pub mod status;
pub mod kdb;
pub mod kdb_detail;
