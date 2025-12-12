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

use anna_shared::event_log::EventLog;
use anna_shared::probe_learning::{LearningHealth, ProbeLearningStore, TrendDirection};
use anna_shared::roster::{person_by_id, Tier};
use anna_shared::staff_stats::{level_title, StaffStats};
use anna_shared::stats::GlobalStats;
use anna_shared::ticket_log::{calculate_stats, load_recent_tickets};
use anna_shared::ui::{colors, kv, kv_colored, print_footer, print_section_header, print_title};

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
    println!();
    print_section_header("service desk");

    let total_tickets = staff_stats.total_tickets();
    let resolved = staff_stats.total_resolved();
    let escalated = staff_stats.total_escalated();

    // Get average response time from event log if available
    let avg_response = agg.as_ref().map(|a| a.avg_duration_ms).unwrap_or(0.0);

    kv("total_tickets", &format!("{}", total_tickets));
    kv_colored("resolved", &format!("{}", resolved), colors::OK);
    kv("escalated", &format!("{}", escalated));
    if avg_response > 0.0 {
        kv("avg_response", &format!("{:.1}s", avg_response / 1000.0));
    }

    // === [departments] ===
    let by_dept = staff_stats.by_department();
    if !by_dept.is_empty() {
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

    // === [staff roster] ===
    if !by_dept.is_empty() {
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

    // === [recent activity] ===
    if let Some(ref agg) = agg {
        if agg.total_requests > 0 {
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
    }

    // === [rpg] === v0.0.450: RPG stats per VISION.md
    if let Some(ref agg) = agg {
        if agg.total_requests > 0 {
            println!();
            print_section_header("rpg");

            // XP bar (0-100)
            let xp_bar = anna_shared::ui::progress_bar(agg.xp as f32 / 100.0, 20);
            kv(
                "xp",
                &format!(
                    "{}{}/100{}  {}",
                    colors::CYAN,
                    agg.xp,
                    colors::RESET,
                    xp_bar
                ),
            );
            kv(
                "level",
                &format!(
                    "{} - {}{}{}",
                    agg.level,
                    colors::BOLD,
                    agg.title,
                    colors::RESET
                ),
            );

            // Installation date
            if agg.first_event_ts > 0 {
                let install_date = chrono::DateTime::from_timestamp(agg.first_event_ts as i64, 0)
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "-".to_string());
                kv("installed", &install_date);
            }

            // Anna solo count
            if agg.anna_solo_count > 0 {
                let solo_pct = if agg.total_requests > 0 {
                    agg.anna_solo_count as f32 / agg.total_requests as f32 * 100.0
                } else {
                    0.0
                };
                kv(
                    "anna_solo",
                    &format!(
                        "{} ({:.0}% without specialists)",
                        agg.anna_solo_count, solo_pct
                    ),
                );
            }

            // Recipes learned
            if agg.recipes_learned > 0 {
                kv("recipes_learned", &format!("{}", agg.recipes_learned));
            }

            // Most consulted team
            if let Some(ref team) = agg.most_consulted_team {
                kv("most_consulted", team);
            }

            // Response times
            if agg.min_duration_ms > 0 || agg.max_duration_ms > 0 {
                kv(
                    "response_times",
                    &format!(
                        "fastest: {:.1}s, longest: {:.1}s",
                        agg.min_duration_ms as f64 / 1000.0,
                        agg.max_duration_ms as f64 / 1000.0
                    ),
                );
            }

            // Streaks
            if agg.best_streak > 0 {
                kv(
                    "streaks",
                    &format!(
                        "current: {} days, best: {} days",
                        agg.current_streak, agg.best_streak
                    ),
                );
            }

            // Average interactions
            if agg.avg_interactions > 0.0 {
                kv(
                    "avg_interactions",
                    &format!(
                        "{:.1} (max: {})",
                        agg.avg_interactions, agg.max_interactions
                    ),
                );
            }
        }
    }

    // === [quick stats] === Summary line
    if total_tickets > 0 {
        println!();
        print_section_header("quick stats");

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

    // === [efficiency] === v0.0.406: Recipe vs LLM stats
    print_efficiency_section();

    print_footer();
}

/// v0.0.330: Print probe learning statistics section
pub fn print_learning_section() {
    let store = ProbeLearningStore::load();
    let stats = store.learning_stats();

    // Only show if there's something to show
    if stats.total_queries == 0 && stats.keywords_learned == 0 {
        return;
    }

    println!();
    print_section_header("learning");

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
            &format!(
                "{}{:.1}/5{}",
                quality_color,
                stats.avg_quality,
                colors::RESET
            ),
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
                trend_color,
                trend_icon,
                colors::RESET,
                trend.trend,
                trend.previous_avg,
                trend.current_avg
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
        &format!(
            "{}{}{} ({:.0}% confidence)",
            health_color,
            health,
            colors::RESET,
            confidence * 100.0
        ),
    );
}

/// v0.0.407: Print truthful stats section
pub fn print_efficiency_section() {
    let tickets = load_recent_tickets(100);

    // Only show if there's data
    if tickets.is_empty() {
        return;
    }

    let stats = calculate_stats(&tickets);

    println!();
    print_section_header("outcomes");

    // v0.0.407: Truthful outcome counts
    let total = stats.total;
    if total > 0 {
        kv("total_tickets", &format!("{}", total));

        // Success rate with color coding
        let rate_color = if stats.success_rate >= 80.0 {
            colors::OK
        } else if stats.success_rate >= 50.0 {
            colors::WARN
        } else {
            colors::ERR
        };
        kv(
            "resolved",
            &format!(
                "{}{}{} ({}{:.0}%{})",
                colors::OK,
                stats.success,
                colors::RESET,
                rate_color,
                stats.success_rate,
                colors::RESET
            ),
        );

        if stats.failed > 0 {
            kv(
                "failed",
                &format!("{}{}{}", colors::ERR, stats.failed, colors::RESET),
            );
        }

        // v0.0.407: Show LLM failures separately
        if stats.llm_failed > 0 {
            kv(
                "llm_failed",
                &format!(
                    "{}{}{} (timeout/parse errors)",
                    colors::WARN,
                    stats.llm_failed,
                    colors::RESET
                ),
            );
        }

        if stats.escalated > 0 {
            kv("escalated", &format!("{}", stats.escalated));
        }
    }

    println!();
    print_section_header("handlers");

    // Handler breakdown
    let recipe_count = stats.by_handler.get("recipe").copied().unwrap_or(0);
    let llm_count = stats.by_handler.get("llm").copied().unwrap_or(0);
    let det_count = stats.by_handler.get("deterministic").copied().unwrap_or(0);

    if total > 0 {
        if recipe_count > 0 {
            let recipe_pct = recipe_count as f64 / total as f64 * 100.0;
            kv(
                "recipes",
                &format!(
                    "{}{}{} ({:.0}%)",
                    colors::OK,
                    recipe_count,
                    colors::RESET,
                    recipe_pct
                ),
            );
        }
        if llm_count > 0 {
            let llm_pct = llm_count as f64 / total as f64 * 100.0;
            kv(
                "llm",
                &format!(
                    "{}{}{} ({:.0}%)",
                    colors::WARN,
                    llm_count,
                    colors::RESET,
                    llm_pct
                ),
            );
        }
        if det_count > 0 {
            let det_pct = det_count as f64 / total as f64 * 100.0;
            kv(
                "deterministic",
                &format!(
                    "{}{}{} ({:.0}%)",
                    colors::DIM,
                    det_count,
                    colors::RESET,
                    det_pct
                ),
            );
        }

        // LLM savings indicator
        let savings = recipe_count + det_count;
        if savings > 0 {
            let savings_pct = savings as f64 / total as f64 * 100.0;
            kv(
                "llm_avoided",
                &format!(
                    "{}{:.0}%{} handled without LLM",
                    colors::OK,
                    savings_pct,
                    colors::RESET
                ),
            );
        }
    }

    // Average metrics (only from answered tickets)
    if stats.avg_duration_ms > 0 || stats.avg_reliability > 0 {
        println!();
        print_section_header("metrics");

        if stats.avg_duration_ms > 0 {
            kv(
                "avg_response",
                &format!(
                    "{:.1}s (answered only)",
                    stats.avg_duration_ms as f64 / 1000.0
                ),
            );
        }
        if stats.avg_reliability > 0 {
            let rel_color = if stats.avg_reliability >= 80 {
                colors::OK
            } else if stats.avg_reliability >= 60 {
                colors::WARN
            } else {
                colors::DIM
            };
            kv(
                "avg_reliability",
                &format!("{}{}%{}", rel_color, stats.avg_reliability, colors::RESET),
            );
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}
