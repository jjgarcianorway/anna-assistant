//! Tests for fun statistics (v0.0.479).

#[cfg(test)]
mod tests {
    use crate::event_log::AggregatedEvents;
    use crate::fun_stats_display::*;

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
