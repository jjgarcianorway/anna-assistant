//! Staff roster display section.

use anna_shared::roster::{person_by_id, Tier};
use anna_shared::staff_stats::{level_title, StaffStats};
use anna_shared::ui::{colors, print_section_header};

use crate::stats_display_v2::utils::capitalize;

/// Print the staff roster section
pub fn print_staff_roster_section(staff_stats: &StaffStats) {
    let by_dept = staff_stats.by_department();
    if by_dept.is_empty() {
        return;
    }

    println!();
    print_section_header("staff roster");

    // Sort departments alphabetically for roster display
    let mut depts: Vec<_> = by_dept.iter().collect();
    depts.sort_by(|a, b| a.0.cmp(b.0));

    for (dept_name, staff) in depts {
        if staff.is_empty() {
            continue;
        }

        // Department header
        println!("  {}", dept_name.to_uppercase());

        // Sort staff by tickets handled (descending)
        let mut sorted_staff = staff.clone();
        sorted_staff.sort_by(|a, b| b.1.tickets_handled.cmp(&a.1.tickets_handled));

        for (person_id, metrics) in sorted_staff {
            // Look up real name from roster
            let (name, tier_label) = if let Some(profile) = person_by_id(person_id) {
                let tier = match profile.tier {
                    Tier::Junior => "Jr",
                    Tier::Senior => "Sr",
                };
                (profile.display_name.to_string(), tier)
            } else {
                // Fallback: extract from person_id
                let parts: Vec<&str> = person_id.split('_').collect();
                let name = if parts.len() >= 3 {
                    capitalize(parts[2])
                } else {
                    capitalize(parts.last().unwrap_or(&"Unknown"))
                };
                let tier = if person_id.contains("_sr") {
                    "Sr"
                } else {
                    "Jr"
                };
                (name, tier)
            };

            let success_rate = metrics.success_rate();
            let rate_color = if success_rate >= 80.0 {
                colors::OK
            } else if success_rate >= 50.0 {
                colors::WARN
            } else {
                colors::DIM
            };

            // Get level title based on tier
            let is_senior = tier_label == "Sr";
            let level_name = level_title(metrics.level, is_senior);

            // v0.0.316: Better aligned columns
            // Name (Tier) padded to 14 chars, then fixed-width columns
            let name_col = format!("{} ({})", name, tier_label);
            println!(
                "    {:<14} tickets: {:>3}   xp: {:>4}   rate: {}{:>3.0}%{}   {}",
                name_col,
                metrics.tickets_handled,
                metrics.xp,
                rate_color,
                success_rate,
                colors::RESET,
                level_name
            );
        }
    }
}
