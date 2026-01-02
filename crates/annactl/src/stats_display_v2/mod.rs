//! Stats display v2 - Service Desk Staff Performance Report (v0.0.344).
//!
//! Clean, focused view of the service desk with real staff metrics:
//! - Service desk summary (total tickets, resolved, escalated)
//! - Department breakdown
//! - Staff roster with names, XP, levels
//! - Quick summary
//! - Learning stats (v0.0.330)
//! - Recipe vs LLM stats (v0.0.406)
//! - RPG stats (v0.0.450)
//! - Category filtering (v0.0.464)
//!
//! v0.0.316: Improved formatting to match service desk vision.
//! v0.0.330: Added probe learning stats section.
//! v0.0.331: Added quality trend to learning section.
//! v0.0.332: Added confidence factor and health status.
//! v0.0.338: Use centralized UI helpers for consistency.
//! v0.0.344: Use print_title() and print_footer() for consistency.
//! v0.0.406: Added recipe vs LLM efficiency section.
//! v0.0.450: Added RPG stats section per VISION.md.
//! v0.0.464: Added category filter (annactl stats <category>).

mod service_desk;
mod staff_roster;
mod rpg_section;
mod learning_section;
mod efficiency_section;
mod utils;

use anna_shared::event_log::EventLog;
use anna_shared::staff_stats::StaffStats;
use anna_shared::stats::GlobalStats;
use anna_shared::ui::{print_footer, print_section_header, print_title};

// Re-export public functions
pub use learning_section::print_learning_section;
pub use efficiency_section::print_efficiency_section;

/// Print the Service Desk staff performance report
pub fn print_stats_display_v2(_stats: &GlobalStats) {
    // Load staff stats (the real source of truth)
    let staff_stats = StaffStats::load();

    // Load event log for recent activity
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    let agg = event_log.aggregate().ok();

    // === HEADER ===
    print_title("Anna Service Desk | Staff Performance Report");

    // === [service desk] ===
    service_desk::print_service_desk_section(&staff_stats, agg.as_ref());

    // === [departments] ===
    service_desk::print_departments_section(&staff_stats);

    // === [staff roster] ===
    staff_roster::print_staff_roster_section(&staff_stats);

    // === [recent activity] ===
    service_desk::print_recent_activity_section(agg.as_ref());

    // === [rpg] === v0.0.450: RPG stats per VISION.md
    rpg_section::print_rpg_section(agg.as_ref());

    // === [quick stats] === Summary line
    service_desk::print_quick_stats_section(&staff_stats);

    // === [learning] === v0.0.330: Probe learning stats
    print_learning_section();

    // === [efficiency] === v0.0.406: Recipe vs LLM stats
    print_efficiency_section();

    print_footer();
}
