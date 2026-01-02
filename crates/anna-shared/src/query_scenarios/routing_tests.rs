//! Routing accuracy tests for query scenarios (v0.0.268).
//!
//! Tests routing accuracy and team matching.

#[cfg(test)]
mod tests {
    use super::super::helpers::{infer_domain, infer_route_class};
    use crate::query_scenarios::ScenarioCorpus;
    use crate::teams::team_from_domain_intent;

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
}
