//! Category filtering for fun statistics (v0.0.479).

use super::formatters::{format_duration, format_install_date};
use super::types::FunStats;

/// Fun stats category for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunStatsCategory {
    /// All stats
    All,
    /// History (installation, days active)
    History,
    /// Team statistics
    Teams,
    /// Anna's independence
    Independence,
    /// Response times
    Times,
    /// Streaks
    Streaks,
}

impl FunStatsCategory {
    /// Parse category from string
    pub fn parse(s: &str) -> Option<Self> {
        let lower = s.to_lowercase();
        match lower.as_str() {
            "all" | "everything" => Some(Self::All),
            "history" | "install" | "installation" => Some(Self::History),
            "team" | "teams" => Some(Self::Teams),
            "independence" | "solo" | "anna" => Some(Self::Independence),
            "time" | "times" | "response" | "speed" => Some(Self::Times),
            "streak" | "streaks" => Some(Self::Streaks),
            _ => None,
        }
    }
}

/// Format specific category of fun stats
pub fn format_fun_stats_category(stats: &FunStats, category: FunStatsCategory) -> String {
    match category {
        FunStatsCategory::All => super::formatters::format_fun_stats(stats),
        FunStatsCategory::History => {
            let mut output = String::new();
            output.push_str("📅 History\n");
            output.push_str("──────────────────────────────────────\n");
            output.push_str(&format!(
                "  Installed:         {}\n",
                format_install_date(stats.installation_date)
            ));
            output.push_str(&format!("  Days active:       {}\n", stats.days_active));
            output.push_str(&format!(
                "  Total requests:    {}\n",
                stats.total_requests
            ));
            output
        }
        FunStatsCategory::Teams => {
            let mut output = String::new();
            output.push_str("👥 Teams\n");
            output.push_str("──────────────────────────────────────\n");
            if let Some(team) = &stats.most_consulted_team {
                output.push_str(&format!(
                    "  Most consulted:    {} ({} times)\n",
                    team, stats.most_consulted_count
                ));
            }
            if let Some(team) = &stats.lucky_team {
                output.push_str(&format!(
                    "  Lucky team:        {} ({:.0}% success)\n",
                    team,
                    stats.lucky_team_rate * 100.0
                ));
            }
            output
        }
        FunStatsCategory::Independence => {
            let mut output = String::new();
            output.push_str("🤖 Anna's Independence\n");
            output.push_str("──────────────────────────────────────\n");
            output.push_str(&format!(
                "  Solo answers:      {} ({:.1}%)\n",
                stats.anna_solo_count, stats.anna_solo_pct
            ));
            output.push_str(&format!(
                "  Recipes learned:   {}\n",
                stats.recipes_learned
            ));
            output
        }
        FunStatsCategory::Times => {
            let mut output = String::new();
            output.push_str("⏱ Response Times\n");
            output.push_str("──────────────────────────────────────\n");
            output.push_str(&format!(
                "  Longest reply:     {}\n",
                format_duration(stats.longest_reply_ms)
            ));
            output.push_str(&format!(
                "  Shortest reply:    {}\n",
                format_duration(stats.shortest_reply_ms)
            ));
            output
        }
        FunStatsCategory::Streaks => {
            let mut output = String::new();
            output.push_str("🔥 Streaks\n");
            output.push_str("──────────────────────────────────────\n");
            output.push_str(&format!(
                "  Current streak:    {} days\n",
                stats.current_streak
            ));
            output.push_str(&format!(
                "  Best streak:       {} days\n",
                stats.best_streak
            ));
            output
        }
    }
}
