//! RPG stats section (v0.0.450).

use anna_shared::event_log::AggregatedEvents;
use anna_shared::ui::{colors, kv, print_section_header};

/// Print the RPG stats section
pub fn print_rpg_section(agg: Option<&AggregatedEvents>) {
    let Some(agg) = agg else {
        return;
    };

    if agg.total_requests == 0 {
        return;
    }

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
