//! Corpus structure tests (v0.0.268).
//!
//! Tests corpus size, team coverage, and difficulty distribution.

#[cfg(test)]
mod tests {
    use crate::query_scenarios::{Difficulty, ScenarioCorpus, ScenarioStats};
    use crate::teams::Team;

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

        assert!(
            simple >= 20,
            "Should have 20+ simple scenarios, got {}",
            simple
        );
        assert!(
            medium >= 30,
            "Should have 30+ medium scenarios, got {}",
            medium
        );
        assert!(
            complex >= 10,
            "Should have 10+ complex scenarios, got {}",
            complex
        );
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
