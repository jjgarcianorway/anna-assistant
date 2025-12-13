//! Fun Statistics Display (v0.0.479).
//!
//! Formats and displays fun/interesting statistics about Anna usage
//! as specified in VISION.md's "Fun Statistics" section.
//!
//! Features:
//! - Most consulted team
//! - Repeated questions
//! - Topic most asked about
//! - Longest/shortest reply
//! - Installation date
//! - And more interesting data

use crate::event_log::AggregatedEvents;

/// Fun statistics derived from aggregated events
#[derive(Debug, Clone, Default)]
pub struct FunStats {
    /// Most consulted team
    pub most_consulted_team: Option<String>,
    /// Most consulted team count
    pub most_consulted_count: u64,
    /// Topic most asked about
    pub top_topic: Option<String>,
    /// Top topic count
    pub top_topic_count: u64,
    /// Number of repeated questions
    pub repeated_questions: u64,
    /// Longest reply duration (ms, proxy for length)
    pub longest_reply_ms: u64,
    /// Shortest reply duration (ms, proxy for length)
    pub shortest_reply_ms: u64,
    /// Installation date (first event timestamp)
    pub installation_date: u64,
    /// Days since installation
    pub days_active: u64,
    /// Anna solo successes (no escalation)
    pub anna_solo_count: u64,
    /// Solo percentage
    pub anna_solo_pct: f32,
    /// Lucky team (highest success rate)
    pub lucky_team: Option<String>,
    /// Lucky team rate
    pub lucky_team_rate: f32,
    /// Current streak (days)
    pub current_streak: u32,
    /// Best streak ever
    pub best_streak: u32,
    /// Total requests
    pub total_requests: u64,
    /// Recipes learned
    pub recipes_learned: u64,
}

impl FunStats {
    /// Create fun stats from aggregated events
    pub fn from_aggregated(agg: &AggregatedEvents) -> Self {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let days_active = if agg.first_event_ts > 0 {
            (now_secs - agg.first_event_ts) / 86400
        } else {
            0
        };

        let solo_pct = if agg.total_requests > 0 {
            (agg.anna_solo_count as f32 / agg.total_requests as f32) * 100.0
        } else {
            0.0
        };

        // Find most consulted team from by_team
        let (most_team, most_count) = agg
            .by_team
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(team, count)| (Some(team.clone()), *count))
            .unwrap_or((None, 0));

        Self {
            most_consulted_team: most_team,
            most_consulted_count: most_count,
            top_topic: agg.most_consulted_team.clone(), // reuse as topic proxy
            top_topic_count: most_count,
            repeated_questions: 0, // TODO: track in aggregation
            longest_reply_ms: agg.longest_reply_chars,
            shortest_reply_ms: agg.shortest_reply_chars,
            installation_date: agg.first_event_ts,
            days_active,
            anna_solo_count: agg.anna_solo_count,
            anna_solo_pct: solo_pct,
            lucky_team: agg.lucky_team.clone(),
            lucky_team_rate: agg.lucky_team_rate,
            current_streak: agg.current_streak,
            best_streak: agg.best_streak,
            total_requests: agg.total_requests,
            recipes_learned: agg.recipes_learned,
        }
    }
}

/// Format installation date from timestamp
pub fn format_install_date(timestamp: u64) -> String {
    if timestamp == 0 {
        return "Unknown".to_string();
    }

    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0);
    match dt {
        Some(d) => d.format("%Y-%m-%d").to_string(),
        None => "Unknown".to_string(),
    }
}

/// Format duration as human-readable time
fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{:.1}min", ms as f64 / 60_000.0)
    }
}

/// Generate a fun fact about the stats
pub fn generate_fun_fact(stats: &FunStats) -> Option<String> {
    let mut facts = Vec::new();

    // Streak facts
    if stats.current_streak >= 7 {
        facts.push(format!(
            "You're on a {} day streak! Keep it up!",
            stats.current_streak
        ));
    }

    if stats.best_streak >= 30 {
        facts.push(format!(
            "Your record streak was {} days. Impressive dedication!",
            stats.best_streak
        ));
    }

    // Anna solo facts
    if stats.anna_solo_pct >= 50.0 {
        facts.push(format!(
            "Anna handled {:.0}% of requests solo - quite self-sufficient!",
            stats.anna_solo_pct
        ));
    }

    // Lucky team
    if let Some(team) = &stats.lucky_team {
        if stats.lucky_team_rate >= 0.9 {
            facts.push(format!(
                "The {} team has a {:.0}% success rate - your lucky team!",
                team,
                stats.lucky_team_rate * 100.0
            ));
        }
    }

    // Installation milestone
    if stats.days_active >= 365 {
        let years = stats.days_active / 365;
        facts.push(format!(
            "You've been using Anna for over {} year{}. Thank you for your loyalty!",
            years,
            if years > 1 { "s" } else { "" }
        ));
    } else if stats.days_active >= 30 {
        facts.push(format!(
            "Anna has been helping you for {} days now.",
            stats.days_active
        ));
    }

    // Recipes learned
    if stats.recipes_learned >= 50 {
        facts.push(format!(
            "Anna has learned {} recipes from you. She's becoming quite knowledgeable!",
            stats.recipes_learned
        ));
    }

    // Request milestones
    let milestone = match stats.total_requests {
        r if r >= 10000 => Some(("10,000", "power user")),
        r if r >= 1000 => Some(("1,000", "regular")),
        r if r >= 100 => Some(("100", "getting started")),
        _ => None,
    };

    if let Some((count, label)) = milestone {
        facts.push(format!(
            "You've reached {} requests - you're a {}!",
            count, label
        ));
    }

    if facts.is_empty() {
        None
    } else {
        // Return a pseudo-random fact based on current time
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as usize)
            % facts.len();
        Some(facts[idx].clone())
    }
}

/// Format fun stats for display (full view)
pub fn format_fun_stats(stats: &FunStats) -> String {
    let mut output = String::new();

    output.push_str("Fun Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    // Installation & Usage
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
    output.push('\n');

    // Team stats
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
    output.push('\n');

    // Anna's independence
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
    output.push('\n');

    // Response times
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
    output.push('\n');

    // Streaks
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

    // Add a fun fact at the end
    if let Some(fact) = generate_fun_fact(stats) {
        output.push('\n');
        output.push_str("💡 Fun Fact\n");
        output.push_str("──────────────────────────────────────\n");
        output.push_str(&format!("  {}\n", fact));
    }

    output
}

/// Format fun stats for compact display (greeting integration)
pub fn format_fun_stats_compact(stats: &FunStats) -> String {
    let mut parts = Vec::new();

    if stats.days_active > 0 {
        parts.push(format!("{} days together", stats.days_active));
    }

    if stats.total_requests > 0 {
        parts.push(format!("{} requests", stats.total_requests));
    }

    if stats.current_streak > 0 {
        parts.push(format!("{} day streak", stats.current_streak));
    }

    if parts.is_empty() {
        "No statistics yet".to_string()
    } else {
        parts.join(" | ")
    }
}

/// Detect if a query is asking for fun stats
pub fn is_fun_stats_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    // Direct patterns
    let patterns = [
        "fun stat",
        "fun fact",
        "interesting stat",
        "interesting fact",
        "show me something interesting",
        "tell me something fun",
        "any fun stat",
        "anna trivia",
        "usage trivia",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    // Question patterns
    if lower.contains("how long") && lower.contains("using anna") {
        return true;
    }

    if lower.contains("when") && lower.contains("install") && lower.contains("anna") {
        return true;
    }

    if lower.contains("how many") && (lower.contains("request") || lower.contains("question")) {
        return true;
    }

    false
}

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
        FunStatsCategory::All => format_fun_stats(stats),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_aggregated() -> AggregatedEvents {
        let mut agg = AggregatedEvents::default();
        agg.total_requests = 150;
        agg.first_event_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (45 * 86400); // 45 days ago
        agg.anna_solo_count = 75;
        agg.recipes_learned = 25;
        agg.longest_reply_chars = 5000;
        agg.shortest_reply_chars = 100;
        agg.current_streak = 7;
        agg.best_streak = 14;
        agg.lucky_team = Some("Storage".to_string());
        agg.lucky_team_rate = 0.95;
        agg.by_team.insert("Storage".to_string(), 50);
        agg.by_team.insert("Network".to_string(), 40);
        agg.by_team.insert("Desktop".to_string(), 30);
        agg
    }

    #[test]
    fn test_fun_stats_from_aggregated() {
        let agg = make_test_aggregated();
        let stats = FunStats::from_aggregated(&agg);

        assert_eq!(stats.total_requests, 150);
        assert_eq!(stats.anna_solo_count, 75);
        assert!(stats.anna_solo_pct > 49.0 && stats.anna_solo_pct < 51.0);
        assert_eq!(stats.most_consulted_team, Some("Storage".to_string()));
        assert_eq!(stats.most_consulted_count, 50);
        assert_eq!(stats.current_streak, 7);
        assert_eq!(stats.best_streak, 14);
    }

    #[test]
    fn test_format_install_date() {
        // Test valid timestamp
        let ts = 1700000000; // Nov 14, 2023
        let formatted = format_install_date(ts);
        assert!(formatted.contains("2023"));

        // Test zero timestamp
        assert_eq!(format_install_date(0), "Unknown");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(90000), "1.5min");
    }

    #[test]
    fn test_generate_fun_fact() {
        let agg = make_test_aggregated();
        let stats = FunStats::from_aggregated(&agg);

        let fact = generate_fun_fact(&stats);
        assert!(fact.is_some());

        let fact_text = fact.unwrap();
        // Should be one of our expected facts (including team for lucky team fact)
        assert!(
            fact_text.contains("streak")
                || fact_text.contains("solo")
                || fact_text.contains("days")
                || fact_text.contains("requests")
                || fact_text.contains("team")
        );
    }

    #[test]
    fn test_format_fun_stats_display() {
        let agg = make_test_aggregated();
        let stats = FunStats::from_aggregated(&agg);

        let display = format_fun_stats(&stats);

        assert!(display.contains("Fun Statistics"));
        assert!(display.contains("History"));
        assert!(display.contains("Teams"));
        assert!(display.contains("Independence"));
        assert!(display.contains("Response Times"));
        assert!(display.contains("Streaks"));
        assert!(display.contains("Storage")); // Most consulted team
    }

    #[test]
    fn test_format_fun_stats_compact() {
        let agg = make_test_aggregated();
        let stats = FunStats::from_aggregated(&agg);

        let compact = format_fun_stats_compact(&stats);

        assert!(compact.contains("days together"));
        assert!(compact.contains("150 requests"));
        assert!(compact.contains("7 day streak"));
    }

    #[test]
    fn test_is_fun_stats_query() {
        assert!(is_fun_stats_query("show me fun stats"));
        assert!(is_fun_stats_query("any interesting facts?"));
        assert!(is_fun_stats_query("tell me something fun about my usage"));
        assert!(is_fun_stats_query("how long have I been using anna?"));
        assert!(is_fun_stats_query("when did I install anna?"));
        assert!(is_fun_stats_query("how many requests have I made?"));

        assert!(!is_fun_stats_query("check disk space"));
        assert!(!is_fun_stats_query("restart docker"));
    }

    #[test]
    fn test_fun_stats_category_parse() {
        assert_eq!(
            FunStatsCategory::parse("all"),
            Some(FunStatsCategory::All)
        );
        assert_eq!(
            FunStatsCategory::parse("history"),
            Some(FunStatsCategory::History)
        );
        assert_eq!(
            FunStatsCategory::parse("teams"),
            Some(FunStatsCategory::Teams)
        );
        assert_eq!(
            FunStatsCategory::parse("independence"),
            Some(FunStatsCategory::Independence)
        );
        assert_eq!(
            FunStatsCategory::parse("times"),
            Some(FunStatsCategory::Times)
        );
        assert_eq!(
            FunStatsCategory::parse("streaks"),
            Some(FunStatsCategory::Streaks)
        );
        assert_eq!(FunStatsCategory::parse("unknown"), None);
    }

    #[test]
    fn test_format_category() {
        let agg = make_test_aggregated();
        let stats = FunStats::from_aggregated(&agg);

        let history = format_fun_stats_category(&stats, FunStatsCategory::History);
        assert!(history.contains("History"));
        assert!(history.contains("Installed"));
        assert!(!history.contains("Teams"));

        let teams = format_fun_stats_category(&stats, FunStatsCategory::Teams);
        assert!(teams.contains("Teams"));
        assert!(teams.contains("Storage"));
        assert!(!teams.contains("Streaks"));
    }

    #[test]
    fn test_empty_stats() {
        let agg = AggregatedEvents::default();
        let stats = FunStats::from_aggregated(&agg);

        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.days_active, 0);
        assert!(stats.most_consulted_team.is_none());

        let compact = format_fun_stats_compact(&stats);
        assert_eq!(compact, "No statistics yet");
    }
}
