//! Memory recall - finding and suggesting relevant experiences.
//! v0.0.930: Uses keyword index for faster lookups

use std::collections::HashMap;
use std::collections::HashSet;

use super::cluster::{calculate_cluster_similarity, calculate_relevance};
use super::types::{Experience, Memory};
use super::extract_keywords;

impl Memory {
    /// Find relevant experiences for a question
    /// v0.0.930: Uses keyword index for O(k) instead of O(n) lookup where k << n
    pub fn recall(&self, question: &str, limit: usize) -> Vec<&Experience> {
        let keywords = extract_keywords(question);
        let question_lower = question.to_lowercase();

        // v0.0.930: Use index to get candidate experience IDs
        let candidate_ids: HashSet<&str> = self.get_candidates_by_keywords(&keywords)
            .into_iter()
            .collect();

        // If index has candidates, only score those; otherwise fall back to full scan
        let experiences_to_score: Vec<&Experience> = if !candidate_ids.is_empty() {
            self.experiences
                .iter()
                .filter(|exp| candidate_ids.contains(exp.id.as_str()))
                .collect()
        } else {
            // Fallback for empty index or no keyword matches
            self.experiences.iter().collect()
        };

        let mut scored: Vec<(&Experience, f32)> = experiences_to_score
            .into_iter()
            .filter_map(|exp| {
                let score = calculate_relevance(exp, &question_lower, &keywords);
                if score > 0.2 {
                    Some((exp, score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.usefulness_score.cmp(&a.0.usefulness_score))
        });

        scored.into_iter().take(limit).map(|(e, _)| e).collect()
    }

    /// Get suggested commands based on learned patterns
    pub fn suggest_commands(&self, question: &str) -> Vec<String> {
        let keywords = extract_keywords(question);
        let mut suggestions: Vec<(String, u32)> = Vec::new();

        for keyword in &keywords {
            for pattern in &self.patterns {
                if pattern
                    .keywords
                    .iter()
                    .any(|k| k == keyword || keyword.contains(k))
                {
                    for cmd in &pattern.common_commands {
                        if let Some((_, count)) =
                            suggestions.iter_mut().find(|(c, _)| c == &cmd.command)
                        {
                            *count += cmd.success_count;
                        } else {
                            suggestions.push((cmd.command.clone(), cmd.success_count));
                        }
                    }
                }
            }
        }

        suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        suggestions.into_iter().map(|(c, _)| c).collect()
    }

    /// Get commands suggested by clusters (semantic recall)
    pub fn suggest_commands_from_clusters(&self, question: &str) -> Vec<String> {
        let mut suggestions: HashMap<String, u32> = HashMap::new();

        for cluster in &self.clusters {
            let sim = calculate_cluster_similarity(question, cluster);
            if sim > 0.5 {
                for cmd in &cluster.effective_commands {
                    let weight = (sim * cmd.success_count as f32) as u32;
                    *suggestions.entry(cmd.command.clone()).or_insert(0) += weight.max(1);
                }
            }
        }

        let mut sorted: Vec<_> = suggestions.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().map(|(c, _)| c).take(5).collect()
    }

    /// Enhanced recall using clusters
    /// v0.0.930: Uses keyword index for faster lookups
    pub fn recall_with_clusters(&self, question: &str, limit: usize) -> Vec<&Experience> {
        let keywords = extract_keywords(question);
        let question_lower = question.to_lowercase();

        // Get cluster experience IDs
        let mut cluster_exp_ids: HashSet<String> = HashSet::new();
        for cluster in &self.clusters {
            let sim = calculate_cluster_similarity(question, cluster);
            if sim > 0.5 {
                cluster_exp_ids.extend(cluster.experience_ids.clone());
            }
        }

        // v0.0.930: Use index to get candidate experience IDs
        let keyword_candidates: HashSet<&str> = self.get_candidates_by_keywords(&keywords)
            .into_iter()
            .collect();

        // Combine candidates from keywords and clusters
        let all_candidate_ids: HashSet<&str> = keyword_candidates
            .into_iter()
            .chain(cluster_exp_ids.iter().map(|s| s.as_str()))
            .collect();

        // If we have candidates, only score those; otherwise fall back to full scan
        let experiences_to_score: Vec<&Experience> = if !all_candidate_ids.is_empty() {
            self.experiences
                .iter()
                .filter(|exp| all_candidate_ids.contains(exp.id.as_str()))
                .collect()
        } else {
            self.experiences.iter().collect()
        };

        let mut scored: Vec<(&Experience, f32)> = experiences_to_score
            .into_iter()
            .filter_map(|exp| {
                let mut score = calculate_relevance(exp, &question_lower, &keywords);

                if cluster_exp_ids.contains(&exp.id) {
                    score += 0.2;
                }

                if score > 0.2 {
                    Some((exp, score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.usefulness_score.cmp(&a.0.usefulness_score))
        });

        scored.into_iter().take(limit).map(|(e, _)| e).collect()
    }
}
