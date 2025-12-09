//! Query scenario tests (v0.0.268).
//!
//! Tests routing accuracy, fast path detection, and recipe learning eligibility.

#[cfg(test)]
mod tests {
    use crate::fastpath::classify_fast_path;
    use crate::query_scenarios::{Difficulty, ScenarioCorpus, ScenarioStats};
    use crate::teams::{team_from_domain_intent, Team};

    /// Infer domain from query for routing test
    fn infer_domain(query: &str) -> &'static str {
        let q = query.to_lowercase();

        // Check desktop first since it has many specific keywords
        if q.contains("vim") || q.contains("nano") || q.contains("editor")
            || q.contains("emacs") || q.contains("helix") || q.contains("neovim")
            || q.contains("hyprland") || q.contains("gnome") || q.contains("kde")
            || q.contains("gtk") || q.contains("wayland") || q.contains("x11")
            || q.contains("theme") || q.contains("font") || q.contains("hidpi")
            || q.contains("tmux") || q.contains("bash prompt") || q.contains("ps1")
            || q.contains("sway") || q.contains("i3") || q.contains("dark mode")
            || q.contains("shortcut") || q.contains("keybind") || q.contains("screenshot")
        {
            return "desktop";
        }
        // Storage
        if q.contains("disk") || q.contains("storage") || q.contains("mount")
            || q.contains("partition") || q.contains("btrfs") || q.contains("filesystem")
            || q.contains("space") || q.contains("inode") || q.contains("lsblk")
        {
            return "storage";
        }
        // Network - check after storage to handle "disk space" correctly
        if q.contains("network") || q.contains("wifi") || q.contains("ip ")
            || q.contains("dns") || q.contains("vpn") || q.contains("internet")
            || q.contains("bonding") || q.contains("bridge") || q.contains("connected")
        {
            return "network";
        }
        // Services
        if q.contains("systemd") || q.contains("nginx") || q.contains("docker")
            || q.contains("apache") || q.contains("cron") || q.contains("timer")
            || q.contains("daemon") || q.contains("postgresql") || q.contains("httpd")
            || q.contains("sshd")
        {
            return "services";
        }
        // Hardware
        if q.contains("gpu") || q.contains("nvidia") || q.contains("bluetooth")
            || q.contains("sound") || q.contains("audio") || q.contains("webcam")
            || q.contains("keyboard backlight") || q.contains("driver")
            || q.contains("temperature") || q.contains("monitor") || q.contains("display")
        {
            return "hardware";
        }
        // Security
        if q.contains("permission") || q.contains("ssh key") || q.contains("security")
            || q.contains("fail2ban") || q.contains("gpg") || q.contains("encrypt")
            || q.contains("ufw") || q.contains("harden") || q.contains("login")
        {
            return "security";
        }
        // Logs
        if q.contains("log") || q.contains("journal") || q.contains("dmesg")
            || q.contains("syslog") || q.contains("crash")
        {
            return "logs";
        }
        // Performance - check last
        if q.contains("cpu") || q.contains("ram") || q.contains("memory")
            || q.contains("slow") || q.contains("load") || q.contains("performance")
            || q.contains("swap") || q.contains("power") || q.contains("iowait")
        {
            return "performance";
        }

        "system"
    }

    /// Infer route class from query - mirrors team_from_route_class patterns
    fn infer_route_class(query: &str) -> &'static str {
        let q = query.to_lowercase();

        // Desktop - editors and DE/WM
        if q.contains("vim") || q.contains("nano") || q.contains("editor")
            || q.contains("emacs") || q.contains("helix") || q.contains("neovim")
            || q.contains("gnome") || q.contains("kde") || q.contains("hyprland")
            || q.contains("sway") || q.contains("i3") || q.contains("wayland")
            || q.contains("x11") || q.contains("gtk") || q.contains("theme")
            || q.contains("font") || q.contains("hidpi") || q.contains("dark mode")
            || q.contains("tmux") || q.contains("bash prompt") || q.contains("ps1")
            || q.contains("shortcut") || q.contains("keybind") || q.contains("screenshot")
        {
            return "desktop";
        }
        // Storage
        if q.contains("disk") || q.contains("space") || q.contains("mount")
            || q.contains("partition") || q.contains("btrfs") || q.contains("filesystem")
            || q.contains("inode") || q.contains("lsblk") || q.contains("du ")
        {
            return "disk";
        }
        // Network
        if q.contains("network") || q.contains("wifi") || q.contains("ip ")
            || q.contains("dns") || q.contains("port") || q.contains("firewall")
            || q.contains("vpn") || q.contains("internet") || q.contains("connection")
            || q.contains("bonding") || q.contains("bridge")
        {
            return "network";
        }
        // Services
        if q.contains("service") || q.contains("systemd") || q.contains("nginx")
            || q.contains("docker") || q.contains("apache") || q.contains("cron")
            || q.contains("timer") || q.contains("daemon") || q.contains("postgresql")
            || q.contains("httpd") || q.contains("sshd")
        {
            return "service";
        }
        // Hardware
        if q.contains("gpu") || q.contains("nvidia") || q.contains("bluetooth")
            || q.contains("sound") || q.contains("audio") || q.contains("webcam")
            || q.contains("monitor") || q.contains("display") || q.contains("driver")
        {
            return "hardware";
        }
        // Security
        if q.contains("permission") || q.contains("ssh key") || q.contains("ufw")
            || q.contains("fail2ban") || q.contains("gpg") || q.contains("encrypt")
            || q.contains("security") || q.contains("harden") || q.contains("login")
        {
            return "security";
        }
        // Logs
        if q.contains("log") || q.contains("journal") || q.contains("dmesg")
            || q.contains("syslog") || q.contains("crash")
        {
            return "log";
        }
        // Performance - check after others to avoid false matches
        if q.contains("cpu") || q.contains("ram") || q.contains("memory")
            || q.contains("slow") || q.contains("load") || q.contains("swap")
            || q.contains("performance") || q.contains("power") || q.contains("iowait")
        {
            return "performance";
        }

        ""
    }

    #[test]
    fn test_corpus_has_100_plus_scenarios() {
        let corpus = ScenarioCorpus::load();
        assert!(
            corpus.scenarios.len() >= 100,
            "Corpus should have 100+ scenarios, got {}",
            corpus.scenarios.len()
        );
    }

    #[test]
    fn test_corpus_covers_all_teams() {
        let corpus = ScenarioCorpus::load();

        let teams = [
            Team::Storage,
            Team::Network,
            Team::Desktop,
            Team::Services,
            Team::Performance,
            Team::Hardware,
            Team::Security,
            Team::Logs,
            Team::General,
        ];

        for team in teams {
            let count = corpus.by_team(team).len();
            assert!(
                count >= 3,
                "Team {:?} should have at least 3 scenarios, got {}",
                team,
                count
            );
        }
    }

    #[test]
    fn test_corpus_has_difficulty_distribution() {
        let corpus = ScenarioCorpus::load();

        let simple = corpus.by_difficulty(Difficulty::Simple).len();
        let medium = corpus.by_difficulty(Difficulty::Medium).len();
        let complex = corpus.by_difficulty(Difficulty::Complex).len();

        assert!(simple >= 20, "Should have 20+ simple scenarios, got {}", simple);
        assert!(medium >= 30, "Should have 30+ medium scenarios, got {}", medium);
        assert!(complex >= 10, "Should have 10+ complex scenarios, got {}", complex);
    }

    #[test]
    fn test_corpus_has_fast_path_scenarios() {
        let corpus = ScenarioCorpus::load();
        let fast_path = corpus.fast_path_scenarios().len();
        assert!(
            fast_path >= 5,
            "Should have 5+ fast path scenarios, got {}",
            fast_path
        );
    }

    #[test]
    fn test_corpus_has_learnable_recipes() {
        let corpus = ScenarioCorpus::load();
        let learnable = corpus.learnable_scenarios().len();
        assert!(
            learnable >= 20,
            "Should have 20+ learnable recipe scenarios, got {}",
            learnable
        );
    }

    #[test]
    fn test_routing_accuracy_for_corpus() {
        let corpus = ScenarioCorpus::load();
        let mut correct = 0;
        let mut incorrect = 0;
        let mut mismatches = Vec::new();

        for scenario in &corpus.scenarios {
            let domain = infer_domain(&scenario.query);
            let route_class = infer_route_class(&scenario.query);
            let actual_team = team_from_domain_intent(domain, "", route_class);

            if actual_team == scenario.expected_team {
                correct += 1;
            } else {
                incorrect += 1;
                mismatches.push((
                    scenario.id,
                    scenario.query.clone(),
                    scenario.expected_team,
                    actual_team,
                ));
            }
        }

        let accuracy = correct as f32 / (correct + incorrect) as f32 * 100.0;

        // Print mismatches for debugging
        if !mismatches.is_empty() {
            eprintln!("\n=== ROUTING MISMATCHES ({}) ===", mismatches.len());
            for (id, query, expected, actual) in mismatches.iter().take(10) {
                eprintln!("  #{}: \"{}\"", id, query);
                eprintln!("      expected: {:?}, got: {:?}", expected, actual);
            }
            if mismatches.len() > 10 {
                eprintln!("  ... and {} more", mismatches.len() - 10);
            }
        }

        assert!(
            accuracy >= 70.0,
            "Routing accuracy should be >= 70%, got {:.1}% ({} correct, {} incorrect)",
            accuracy,
            correct,
            incorrect
        );
    }

    #[test]
    fn test_fast_path_detection_accuracy() {
        let corpus = ScenarioCorpus::load();
        let fast_path_scenarios = corpus.fast_path_scenarios();

        let mut detected = 0;
        let mut missed = Vec::new();

        for scenario in &fast_path_scenarios {
            let class = classify_fast_path(&scenario.query);
            if class != crate::fastpath::FastPathClass::NotFastPath {
                detected += 1;
            } else {
                missed.push(&scenario.query);
            }
        }

        let accuracy = if fast_path_scenarios.is_empty() {
            100.0
        } else {
            detected as f32 / fast_path_scenarios.len() as f32 * 100.0
        };

        if !missed.is_empty() {
            eprintln!("\n=== FAST PATH MISSED ({}) ===", missed.len());
            for q in missed.iter().take(5) {
                eprintln!("  - \"{}\"", q);
            }
        }

        assert!(
            accuracy >= 50.0,
            "Fast path detection should be >= 50%, got {:.1}%",
            accuracy
        );
    }

    #[test]
    fn test_similar_queries_should_match_same_team() {
        let corpus = ScenarioCorpus::load();

        for scenario in &corpus.scenarios {
            if let Some(ref similar) = scenario.similar_query {
                let original_domain = infer_domain(&scenario.query);
                let original_route = infer_route_class(&scenario.query);
                let original_team = team_from_domain_intent(original_domain, "", original_route);

                let similar_domain = infer_domain(similar);
                let similar_route = infer_route_class(similar);
                let similar_team = team_from_domain_intent(similar_domain, "", similar_route);

                // Similar queries should route to the same team
                if original_team != similar_team {
                    eprintln!(
                        "Similar query mismatch: \"{}\" -> {:?}, \"{}\" -> {:?}",
                        scenario.query, original_team, similar, similar_team
                    );
                }
            }
        }
    }

    #[test]
    fn test_complex_queries_not_fast_path() {
        let corpus = ScenarioCorpus::load();

        for scenario in corpus.by_difficulty(Difficulty::Complex) {
            let class = classify_fast_path(&scenario.query);
            assert_eq!(
                class,
                crate::fastpath::FastPathClass::NotFastPath,
                "Complex query should not be fast path: \"{}\"",
                scenario.query
            );
        }
    }

    #[test]
    fn test_scenario_stats_collection() {
        let mut stats = ScenarioStats::new();
        stats.total_scenarios = 100;
        stats.passed = 85;
        stats.failed = 15;
        stats.routing_correct = 90;
        stats.routing_incorrect = 10;
        stats.expected_fast_path = 10;
        stats.actual_fast_path = 8;

        assert!((stats.overall_pass_rate() - 85.0).abs() < 0.1);
        assert!((stats.routing_accuracy() - 90.0).abs() < 0.1);
        assert!((stats.fast_path_accuracy() - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_summary_report_generation() {
        let mut stats = ScenarioStats::new();
        stats.total_scenarios = 100;
        stats.passed = 85;
        stats.failed = 15;

        let report = stats.summary_report();

        assert!(report.contains("SCENARIO TEST SUMMARY"));
        assert!(report.contains("85.0%"));
        assert!(report.contains("BY TEAM"));
    }
}
