//! Display helpers for annactl UI.
//! v0.0.249: Delegate to status_display_v2 for rich formatted output.

use anna_shared::rpc::DaemonInfo;
use anna_shared::status::DaemonStatus;
use anna_shared::status_snapshot::StatusSnapshot;

// Re-export from dedicated modules
pub use crate::progress_display::show_bootstrap_progress;
pub use crate::stats_display::print_stats_display;

/// Print status display (v0.0.249: use new rich format)
pub fn print_status_display(
    status: &DaemonStatus,
    snapshot: Option<&StatusSnapshot>,
    daemon_info: Option<&DaemonInfo>,
) {
    crate::status_display_v2::print_status_display_v2(status, snapshot, daemon_info);
}
