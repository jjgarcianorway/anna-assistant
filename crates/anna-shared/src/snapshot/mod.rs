//! System snapshot for "what changed since last time" detection (v0.0.219).
//!
//! Captures minimal system state for delta detection without spamming users.
//! Only surfaces actionable changes that cross meaningful thresholds.
//!
//! v0.0.219: Modularized into domain-focused submodules.

mod capture;
mod delta;
mod persistence;
pub mod types;

// Re-export for backwards compatibility
pub use capture::capture_snapshot;
pub use delta::{diff_snapshots, format_deltas_text, has_actionable_deltas, DeltaItem};
pub use persistence::{
    clear_snapshots, last_snapshot_path, load_last_snapshot, save_snapshot, snapshots_dir,
};
pub use types::{
    SystemSnapshot, DISK_CHANGE_THRESHOLD, DISK_CRITICAL_THRESHOLD, DISK_WARN_THRESHOLD,
    MEMORY_CHANGE_THRESHOLD, MEMORY_HIGH_THRESHOLD,
};
