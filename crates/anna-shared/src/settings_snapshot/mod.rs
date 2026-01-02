// v0.0.592: Settings Snapshot Module (Phase 168)
// Point-in-time settings snapshots

mod types;
mod snapshot;
mod manager;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{SnapshotType, SnapshotStatus};
pub use snapshot::SettingsSnapshot;
pub use manager::SnapshotManager;
pub use utils::{
    format_snapshot,
    format_snapshots,
    is_snapshot_query,
    settings_snapshot_fun_fact,
};
