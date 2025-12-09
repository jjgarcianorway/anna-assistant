//! Stats display module for annactl.
//! v0.0.250: Delegate to stats_display_v2 for RPG-style output.

use anna_shared::stats::GlobalStats;

/// v0.0.250: Use new RPG-style stats display
pub fn print_stats_display(stats: &GlobalStats) {
    crate::stats_display_v2::print_stats_display_v2(stats);
}
