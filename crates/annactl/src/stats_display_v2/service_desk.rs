//! Service desk sections: service desk summary, departments, recent activity, quick stats.

use anna_shared::event_log::AggregatedEvents;
use anna_shared::staff_stats::StaffStats;
use anna_shared::ui::{colors, kv, kv_colored, print_section_header};

use crate::stats_display_v2::utils::capitalize;

/// Print the service desk summary section
pub fn print_service_desk_section(staff_stats: &StaffStats, agg: Option<&AggregatedEvents>) {
    println!();
    print_section_header("service desk");

    let total_tickets = staff_stats.total_tickets();
    let resolved = staff_stats.total_resolved();
    let escalated = staff_stats.total_escalated();

    // Get average response time from event log if available
    let avg_response = agg.map(|a| a.avg_duration_ms).unwrap_or(0.0);

    kv("total_tickets", &format!("{}", total_tickets));
    kv_colored("resolved", &format!("{}", resolved), colors::OK);
    kv("escalated", &format!("{}", escalated));
    if avg_response > 0.0 {
        kv("avg_response", &format!("{:.1}s", avg_response / 1000.0));
    }
}

/// Print the departments breakdown section
pub fn print_departments_section(staff_stats: &StaffStats) {
    let by_dept = staff_stats.by_department();
    if by_dept.is_empty() {
        return;
    }

    println!();
    print_section_header("departments");

    // Sort departments by ticket count
    let mut depts: Vec<_> = by_dept.iter().collect();
    depts.sort_by(|a, b| {
        let a_tickets: u32 = a.1.iter().map(|(_, m)| m.tickets_handled).sum();
        let b_tickets: u32 = b.1.iter().map(|(_, m)| m.tickets_handled).sum();
        b_tickets.cmp(&a_tickets)
    });

    for (dept_name, staff) in depts.iter().take(6) {
        let dept_tickets: u32 = staff.iter().map(|(_, m)| m.tickets_handled).sum();
        let dept_resolved: u32 = staff.iter().map(|(_, m)| m.tickets_resolved).sum();
        let dept_time: u64 = staff.iter().map(|(_, m)| m.total_time_ms).sum();
        let avg_time = if dept_tickets > 0 {
            dept_time as f64 / dept_tickets as f64 / 1000.0
        } else {
            0.0
        };

        let dept_display = capitalize(dept_name);
        println!(
            "  {:12}  tickets: {:>3}   resolved: {:>3}   avg: {:.1}s",
            dept_display, dept_tickets, dept_resolved, avg_time
        );
    }
}

/// Print the recent activity section
pub fn print_recent_activity_section(agg: Option<&AggregatedEvents>) {
    let Some(agg) = agg else {
        return;
    };

    if agg.total_requests == 0 {
        return;
    }

    println!();
    print_section_header("recent activity");

    // Show summary of recent work
    let resolved_pct = if agg.total_requests > 0 {
        (agg.verified_count as f32 / agg.total_requests as f32 * 100.0) as u8
    } else {
        0
    };

    println!(
        "  {} total requests, {}% resolved successfully",
        agg.total_requests, resolved_pct
    );

    if agg.escalation_count > 0 {
        println!("  {} escalations to senior staff", agg.escalation_count);
    }

    if agg.current_streak > 0 {
        println!("  {} day streak of activity", agg.current_streak);
    }
}

/// Print the quick stats summary section
pub fn print_quick_stats_section(staff_stats: &StaffStats) {
    let total_tickets = staff_stats.total_tickets();
    if total_tickets == 0 {
        return;
    }

    println!();
    print_section_header("quick stats");

    let resolved = staff_stats.total_resolved();
    let overall_rate = if total_tickets > 0 {
        (resolved as f32 / total_tickets as f32 * 100.0) as u8
    } else {
        0
    };

    let staff_count = staff_stats.by_staff.len();
    let by_dept = staff_stats.by_department();
    let dept_count = by_dept.len();

    println!(
        "  {} staff across {} departments, {}% overall success rate",
        staff_count, dept_count, overall_rate
    );
}
