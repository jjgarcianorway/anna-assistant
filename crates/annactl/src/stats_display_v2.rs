//! Stats display v2 - Service Desk Staff Performance Report (v0.0.332).
//!
//! Clean, focused view of the service desk with real staff metrics:
//! - Service desk summary (total tickets, resolved, escalated)
//! - Department breakdown
//! - Staff roster with names, XP, levels
//! - Quick summary
//! - Learning stats (v0.0.330)
//!
//! v0.0.316: Improved formatting to match service desk vision.
//! v0.0.330: Added probe learning stats section.
//! v0.0.331: Added quality trend to learning section.
//! v0.0.332: Added confidence factor and health status.

use anna_shared::event_log::EventLog;
use anna_shared::probe_learning::{LearningHealth, ProbeLearningStore, TrendDirection};
use anna_shared::roster::{person_by_id, Tier};
use anna_shared::staff_stats::{level_title, StaffStats};
use anna_shared::stats::GlobalStats;
use anna_shared::ui::colors;

const HR: &str = "──────────────────────────────────────────────────────────────────────────────";

/// Print the Service Desk staff performance report
pub fn print_stats_display_v2(_stats: &GlobalStats) {
    // Load staff stats (the real source of truth)
    let staff_stats = StaffStats::load();

    // Load event log for recent activity
    let event_log = EventLog::new(EventLog::default_path(), 10000);
    let agg = event_log.aggregate().ok();

    // === HEADER ===
    println!("{}", HR);
    println!("Anna Service Desk  |  Staff Performance Report");
    println!("{}", HR);

    // === [service desk] ===
    println!();
    println!("{}[service desk]{}", colors::HEADER, colors::RESET);

    let total_tickets = staff_stats.total_tickets();
    let resolved = staff_stats.total_resolved();
    let escalated = staff_stats.total_escalated();

    // Get average response time from event log if available
    let avg_response = agg.as_ref().map(|a| a.avg_duration_ms).unwrap_or(0.0);

    kv("total_tickets", &format!("{}", total_tickets));
    kv("resolved", &format!("{}{}{}", colors::OK, resolved, colors::RESET));
    kv("escalated", &format!("{}", escalated));
    if avg_response > 0.0 {
        kv("avg_response", &format!("{:.1}s", avg_response / 1000.0));
    }

    // === [departments] ===
    let by_dept = staff_stats.by_department();
    if !by_dept.is_empty() {
        println!();
        println!("{}[departments]{}", colors::HEADER, colors::RESET);

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

    // === [staff roster] ===
    if !by_dept.is_empty() {
        println!();
        println!("{}[staff roster]{}", colors::HEADER, colors::RESET);

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
                    let tier = if person_id.contains("_sr") { "Sr" } else { "Jr" };
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

    // === [recent activity] ===
    if let Some(ref agg) = agg {
        if agg.total_requests > 0 {
            println!();
            println!("{}[recent activity]{}", colors::HEADER, colors::RESET);

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
    }

    // === [quick stats] === Summary line
    if total_tickets > 0 {
        println!();
        println!("{}[quick stats]{}", colors::HEADER, colors::RESET);

        let overall_rate = if total_tickets > 0 {
            (resolved as f32 / total_tickets as f32 * 100.0) as u8
        } else {
            0
        };

        let staff_count = staff_stats.by_staff.len();
        let dept_count = by_dept.len();

        println!(
            "  {} staff across {} departments, {}% overall success rate",
            staff_count, dept_count, overall_rate
        );
    }

    // === [learning] === v0.0.330: Probe learning stats
    print_learning_section();

    println!("{}", HR);
}

/// v0.0.330: Print probe learning statistics section
fn print_learning_section() {
    let store = ProbeLearningStore::load();
    let stats = store.learning_stats();

    // Only show if there's something to show
    if stats.total_queries == 0 && stats.keywords_learned == 0 {
        return;
    }

    println!();
    println!("{}[learning]{}", colors::HEADER, colors::RESET);

    kv("queries_processed", &format!("{}", stats.total_queries));
    kv("keywords_learned", &format!("{}", stats.keywords_learned));

    if stats.successful_patterns > 0 || stats.negative_patterns > 0 {
        kv(
            "patterns",
            &format!(
                "{}{} success{} / {}{} negative{}",
                colors::OK,
                stats.successful_patterns,
                colors::RESET,
                colors::DIM,
                stats.negative_patterns,
                colors::RESET
            ),
        );
    }

    if stats.avg_quality > 0.0 {
        let quality_color = if stats.avg_quality >= 4.0 {
            colors::OK
        } else if stats.avg_quality >= 3.0 {
            colors::WARN
        } else {
            colors::DIM
        };
        kv(
            "avg_quality",
            &format!("{}{:.1}/5{}", quality_color, stats.avg_quality, colors::RESET),
        );
    }

    // Learning stage indicator
    let stage = if stats.total_queries >= 50 && stats.keywords_learned >= 20 {
        format!("{}Expert{}", colors::OK, colors::RESET)
    } else if stats.total_queries >= 10 {
        format!("{}Growing{}", colors::WARN, colors::RESET)
    } else {
        format!("{}Learning{}", colors::DIM, colors::RESET)
    };
    kv("stage", &stage);

    // v0.0.331: Quality trend
    if let Some(trend) = store.quality_trend() {
        let (trend_icon, trend_color) = match trend.trend {
            TrendDirection::Improving => ("^", colors::OK),
            TrendDirection::Declining => ("v", colors::ERR),
            TrendDirection::Stable => ("=", colors::DIM),
        };
        kv(
            "trend",
            &format!(
                "{}{}{} {} (was {:.1}, now {:.1})",
                trend_color, trend_icon, colors::RESET, trend.trend, trend.previous_avg, trend.current_avg
            ),
        );
    }

    // v0.0.332: Health status and confidence
    let health = store.health_status();
    let health_color = match health {
        LearningHealth::Excellent => colors::OK,
        LearningHealth::Good => colors::OK,
        LearningHealth::Developing => colors::WARN,
        LearningHealth::NeedsAttention => colors::ERR,
        LearningHealth::Insufficient => colors::DIM,
    };
    let confidence = store.confidence_factor();
    kv(
        "health",
        &format!("{}{}{} ({:.0}% confidence)", health_color, health, colors::RESET, confidence * 100.0),
    );
}

fn kv(key: &str, value: &str) {
    println!("  {:22}{}", key, value);
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}
