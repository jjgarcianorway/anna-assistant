//! Daemon status types.
//! v0.0.924: Added memory health fields
//! v0.1.0: Added update timing and extended status fields
//! v0.2.7: Added RPG stats
//! v0.3.3: Added per-specialist stats and ticket tracking
//! v0.3.20: Added permissions audit, ollama version, escalated tickets count
//! v0.3.21: Full status contract - build info, socket health, config, helpers
//! v0.3.36: Added recovery metrics for self-healing infrastructure

mod config_snapshot;
mod daemon_status;
mod errors;
mod helpers;
mod permissions;
mod recovery;
mod rpg;
mod socket;
#[cfg(test)]
mod tests;
mod tickets;
mod types;

// Re-exports
pub use config_snapshot::{ConfigSnapshot, ModelMapping};
pub use daemon_status::DaemonStatus;
pub use errors::{ErrorEntry, ErrorSummary};
pub use helpers::{BackupStatus, HelperInfo, HelperSource, LearningStatus};
pub use permissions::PermissionsAudit;
pub use recovery::{
    RecoveryEvent, RecoveryOutcome, RecoveryStatus, SubsystemHealth, SubsystemRecoveryMetrics,
};
pub use rpg::RpgStats;
pub use socket::{BuildMetadata, SocketHealth, SocketStatus};
pub use tickets::{
    ActiveTicket, DepartmentTicketStats, SpecialistStats, TeamRoster, TicketTracker,
};
pub use types::{DaemonState, SpecialistStatus, TicketStatus, UpdateCheckState};
