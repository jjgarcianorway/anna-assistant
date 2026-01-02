//! Fast path detection tests for query scenarios (v0.0.268).
//!
//! Tests fast path detection accuracy and complex query handling.

#[cfg(test)]
mod tests {
    use crate::fastpath::classify_fast_path;
    use crate::query_scenarios::{Difficulty, ScenarioCorpus};

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
}
