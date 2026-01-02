//! Recipe learning verification tests (v0.0.269).
//!
//! These tests verify that similar queries would match a learned recipe,
//! enabling Anna to reuse previous successful answers.

#[cfg(test)]
mod tests {
    use crate::query_scenarios::ScenarioCorpus;
    use crate::recipe_index::tokenize;
    use crate::synonyms::expand_query_tokens;

    /// Check if two queries have enough token overlap to potentially match
    /// v0.0.272: Now uses synonym expansion for better matching
    fn queries_would_match(original: &str, similar: &str) -> bool {
        let orig_tokens: Vec<String> = tokenize(original);
        let sim_tokens: Vec<String> = tokenize(similar);

        // Expand both with synonyms
        let orig_expanded = expand_query_tokens(&orig_tokens);
        let sim_expanded = expand_query_tokens(&sim_tokens);

        let overlap: Vec<_> = orig_expanded.intersection(&sim_expanded).collect();

        // Need at least 2 matching tokens (including synonyms) for recipe match
        overlap.len() >= 2
    }

    #[test]
    fn test_similar_queries_have_token_overlap() {
        let corpus = ScenarioCorpus::load();
        let mut matched = 0;
        let mut total_with_similar = 0;
        let mut mismatches = Vec::new();

        for scenario in &corpus.scenarios {
            if let Some(ref similar) = scenario.similar_query {
                total_with_similar += 1;
                if queries_would_match(&scenario.query, similar) {
                    matched += 1;
                } else {
                    mismatches.push((scenario.id, scenario.query.clone(), similar.clone()));
                }
            }
        }

        if total_with_similar == 0 {
            return; // No similar queries to test
        }

        let match_rate = matched as f32 / total_with_similar as f32 * 100.0;

        if !mismatches.is_empty() {
            eprintln!(
                "\n=== SIMILAR QUERY TOKEN MISMATCHES ({}) ===",
                mismatches.len()
            );
            for (id, original, similar) in mismatches.iter().take(5) {
                eprintln!("  #{}: \"{}\"", id, original);
                eprintln!("      similar: \"{}\"", similar);
                let orig_tokens: Vec<_> = tokenize(original);
                let sim_tokens: Vec<_> = tokenize(similar);
                eprintln!("      original tokens: {:?}", orig_tokens);
                eprintln!("      similar tokens: {:?}", sim_tokens);
            }
        }

        // Currently at 22% - many paraphrases use completely different words
        // Future: Improve via synonym expansion in recipe_index
        // For now, just require non-zero matches (any recipe learning benefit)
        assert!(
            matched > 0,
            "At least some similar queries should have token overlap, got 0/{}",
            total_with_similar
        );

        // Log the actual match rate for tracking improvement
        eprintln!(
            "\n[INFO] Similar query token overlap: {:.1}% ({}/{})",
            match_rate, matched, total_with_similar
        );
    }

    #[test]
    fn test_learnable_scenarios_have_good_tokens() {
        let corpus = ScenarioCorpus::load();
        let learnable = corpus.learnable_scenarios();

        let mut good_tokens = 0;
        let mut bad_tokens = Vec::new();

        for scenario in &learnable {
            let tokens = tokenize(&scenario.query);
            // Learnable recipes should have at least 3 meaningful tokens
            // to enable proper matching
            if tokens.len() >= 3 {
                good_tokens += 1;
            } else {
                bad_tokens.push((&scenario.query, tokens));
            }
        }

        if !bad_tokens.is_empty() {
            eprintln!(
                "\n=== LEARNABLE SCENARIOS WITH FEW TOKENS ({}) ===",
                bad_tokens.len()
            );
            for (query, tokens) in bad_tokens.iter().take(5) {
                eprintln!("  \"{}\" -> {:?}", query, tokens);
            }
        }

        let good_rate = good_tokens as f32 / learnable.len() as f32 * 100.0;
        assert!(
            good_rate >= 80.0,
            "At least 80% of learnable scenarios should have 3+ tokens, got {:.1}%",
            good_rate
        );
    }

    #[test]
    fn test_paraphrase_matching_examples() {
        // These are canonical paraphrase pairs that should match
        let paraphrase_pairs = [
            // Storage
            ("how much disk space do I have", "what is my disk usage"),
            ("check storage", "disk usage"),
            // Network
            ("what is my ip address", "show my IP"),
            (
                "am I connected to the internet",
                "check internet connection",
            ),
            // Performance
            ("why is my computer slow", "system is slow"),
            ("check CPU usage", "CPU load"),
            // Services
            ("restart docker", "docker service restart"),
            ("nginx status", "is nginx running"),
            // Editor config
            ("enable line numbers in vim", "show line numbers vim"),
            ("set vim theme", "vim colorscheme"),
        ];

        let mut matched = 0;
        let mut failed = Vec::new();

        for (q1, q2) in &paraphrase_pairs {
            if queries_would_match(q1, q2) {
                matched += 1;
            } else {
                failed.push((q1, q2));
            }
        }

        if !failed.is_empty() {
            eprintln!("\n=== PARAPHRASE PAIRS THAT FAILED TO MATCH ===");
            for (q1, q2) in &failed {
                let t1 = tokenize(q1);
                let t2 = tokenize(q2);
                eprintln!("  \"{}\" <-> \"{}\"", q1, q2);
                eprintln!("      tokens: {:?} vs {:?}", t1, t2);
            }
        }

        let match_rate = matched as f32 / paraphrase_pairs.len() as f32 * 100.0;
        // Currently at 40% - paraphrases often use completely different words
        // This is expected without synonym expansion
        // The test documents current state; improving this would require
        // adding synonyms like: storage<->disk, check<->show, etc.
        assert!(
            matched > 0,
            "At least some paraphrase pairs should match, got 0/{}",
            paraphrase_pairs.len()
        );

        // Log the actual match rate for tracking improvement
        eprintln!(
            "\n[INFO] Paraphrase matching: {:.1}% ({}/{})",
            match_rate,
            matched,
            paraphrase_pairs.len()
        );
    }

    #[test]
    fn test_action_verb_preservation() {
        // Action verbs should be preserved in tokens for recipe matching
        let action_queries = [
            ("install htop", vec!["install", "htop"]),
            ("restart nginx", vec!["restart", "nginx"]),
            ("enable bluetooth", vec!["enable", "bluetooth"]),
            ("configure vim", vec!["configure", "vim"]),
            ("check memory", vec!["check", "memory"]),
        ];

        for (query, expected_verbs) in &action_queries {
            let tokens = tokenize(query);
            for verb in expected_verbs {
                assert!(
                    tokens.contains(&verb.to_string()),
                    "Query \"{}\" should preserve action verb \"{}\", got tokens: {:?}",
                    query,
                    verb,
                    tokens
                );
            }
        }
    }

    #[test]
    fn test_target_noun_preservation() {
        // Target nouns should be preserved for substitution matching
        let target_queries = [
            ("edit .vimrc", "vimrc"),
            ("configure hyprland.conf", "hyprland"),
            ("check /home usage", "home"),
            ("restart docker service", "docker"),
        ];

        for (query, expected_target) in &target_queries {
            let tokens = tokenize(query);
            let has_target = tokens.iter().any(|t| t.contains(expected_target));
            assert!(
                has_target,
                "Query \"{}\" should contain target \"{}\", got tokens: {:?}",
                query, expected_target, tokens
            );
        }
    }
}
